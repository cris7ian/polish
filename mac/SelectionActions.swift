import AppKit
import ApplicationServices

final class SelectionActions {
    private struct Source {
        let application: NSRunningApplication
        let pin: SelectionSourcePin
        var element: AXUIElement? { pin.element }
    }
    private struct Capture {
        let source: Source
        let context: SelectionCaptureContext
    }
    private var source: Source?
    private var sourceState = SelectionSourceState(hostPID: ProcessInfo.processInfo.processIdentifier)
    private var activationObserver: NSObjectProtocol?
    private var activeAction = SelectionAction.polish
    private(set) var isWorking = false
    var onStatus: ((String, Bool) -> Void)?
    var onCopied: ((SelectionAction) -> Void)?
    var onRestoreSource: (() -> Void)?
#if POLISH_INTEGRATION_TEST
    var integrationError: ((String) -> Void)?
    var integrationWithoutAX = false
#endif

    init() {
        AXUIElementSetMessagingTimeout(AXUIElementCreateSystemWide(), 0.4)
        if let application = NSWorkspace.shared.frontmostApplication {
            sourceState.recordActivation(application.processIdentifier)
        }
        activationObserver = NSWorkspace.shared.notificationCenter.addObserver(
            forName: NSWorkspace.didActivateApplicationNotification, object: nil, queue: .main
        ) { [weak self] notification in
            guard let application = notification.userInfo?[NSWorkspace.applicationUserInfoKey] as? NSRunningApplication else { return }
            self?.sourceState.recordActivation(application.processIdentifier)
        }
    }

    deinit {
        if let observer = activationObserver { NSWorkspace.shared.notificationCenter.removeObserver(observer) }
    }

    // Remember the app even when it exposes no focused Accessibility element.
    func rememberSource() {
        source = nil
        let frontmostPID = NSWorkspace.shared.frontmostApplication?.processIdentifier
        guard let pid = sourceState.candidatePID(frontmostPID: frontmostPID),
              let application = NSRunningApplication(processIdentifier: pid), !application.isTerminated else { return }
#if POLISH_INTEGRATION_TEST
        let useAX = AXIsProcessTrusted() && !integrationWithoutAX
#else
        let useAX = AXIsProcessTrusted()
#endif
        let element = useAX ? focusedElement(application) : nil
        let window = useAX ? focusedWindow(application) : nil
        guard let pin = sourceState.pin(frontmostPID: frontmostPID, element: element, window: window) else { return }
        source = Source(application: application, pin: pin)
    }

    func run(action: SelectionAction) {
        guard !isWorking else { return }
        guard AXIsProcessTrusted() else {
            let options = [kAXTrustedCheckOptionPrompt.takeUnretainedValue() as String: true] as CFDictionary
            _ = AXIsProcessTrustedWithOptions(options)
            let alert = NSAlert()
            alert.messageText = "Allow Accessibility access"
            alert.informativeText = "Enable Polish in System Settings → Privacy & Security → Accessibility, then try again."
            alert.addButton(withTitle: "Open System Settings")
            alert.addButton(withTitle: "Cancel")
            if alert.runModal() == .alertFirstButtonReturn,
               let url = URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility") {
                NSWorkspace.shared.open(url)
            }
            return
        }
        guard let source = source else {
            show(SelectionActionError.message("Activate the app whose selection you want to edit, then open the Polish menu again."))
            return
        }
        activeAction = action
        isWorking = true
        onStatus?(action.workingStatus, true)
        restoreSource(source) { [weak self] ready in
            guard let self = self else { return }
            guard ready else { self.finish("Source app changed"); return }
            self.capture(source) { captured in
                switch captured {
                case .failure(let error):
                    self.finish("Could not read selection")
                    self.show(error)
                case .success(let capture):
                    DispatchQueue.global(qos: .userInitiated).async { [weak self] in
                        let result = Result { try Self.request(capture.context.text, action: action) }
                        DispatchQueue.main.async { self?.complete(result, capture: capture, action: action) }
                    }
                }
            }
        }
    }

    private func capture(_ source: Source, completion: @escaping (Result<Capture, Error>) -> Void) {
        let document = string(source.element, kAXValueAttribute)
        let range = selectionRange(source.element)
        let selected = string(source.element, kAXSelectedTextAttribute)
        if let context = try? SelectionCaptureContext.resolve(document: document, range: range, selectedText: selected) {
            guard sourceIsCurrent(source), observationMatch(context, source: source) != .changed else {
                completion(.failure(SelectionActionError.message("The source app or selection changed.")))
                return
            }
            completion(.success(Capture(source: source, context: context)))
        } else {
            nativeCopy(source, document: document, range: range) { copied in
                completion(copied.flatMap { selected in Result {
                    let context = try SelectionCaptureContext.resolve(document: document, range: range, selectedText: selected)
                    guard self.observationMatch(context, source: source) != .changed else {
                        throw SelectionActionError.message("The selection changed while copying.")
                    }
                    return Capture(source: source, context: context)
                } })
            }
        }
    }

    private func complete(_ result: Result<String, Error>, capture: Capture, action: SelectionAction) {
        switch result {
        case .failure(let error):
            finish(action == .polish ? "Polish failed" : "Reroll failed")
            show(error)
        case .success(let text):
            guard copyResult(text) else { finish("Could not copy result"); return }
            onCopied?(action)
            restoreSource(capture.source) { [weak self] ready in
                guard let self = self else { return }
                guard ready else { self.finishCopied(); return }
                self.paste(capture, result: text)
            }
        }
    }

    private func paste(_ capture: Capture, result: String, wholeDocumentSelected: Bool = false) {
        guard sourceIsCurrent(capture.source) else { finishCopied(); return }
        switch observationMatch(capture.context, source: capture.source, wholeDocumentSelected: wholeDocumentSelected) {
        case .changed: finishCopied()
        case .confirmed:
            if capture.context.usesWholeDocument && !wholeDocumentSelected {
                guard sendCommand(key: 0, to: capture.source.application.processIdentifier) else { finishCopied(); return }
                DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
                    self.paste(capture, result: result, wholeDocumentSelected: true)
                }
            } else { dispatchPaste(capture, result: result) }
        case .needsNativeCopy:
            nativeCopy(capture.source, document: capture.context.sourceDocument, range: capture.context.sourceRange) { copied in
                // Native Copy replaced the clipboard. Put the model result back on every path.
                guard self.copyResult(result) else { self.finish("Could not copy result"); return }
                guard case .success(let current) = copied,
                      SelectionTextSnapshot.exactlyEqual(current, capture.context.selectedText),
                      self.sourceIsCurrent(capture.source), self.observationMatch(capture.context, source: capture.source) != .changed else {
                    self.finishCopied(); return
                }
                self.dispatchPaste(capture, result: result)
            }
        }
    }

    private func dispatchPaste(_ capture: Capture, result: String) {
        guard copyResult(result), sendCommand(key: 9, to: capture.source.application.processIdentifier) else { finishCopied(); return }
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.3) {
            if let expected = capture.context.expectedDocument(after: result),
               let document = self.string(capture.source.element, kAXValueAttribute),
               SelectionTextSnapshot.exactlyEqual(document, expected) {
                self.reselectResult(capture, result: result)
                self.finish("\(self.activeAction.resultDescription) pasted")
            } else {
                self.reselectResult(capture, result: result)
                self.finish("Result copied; paste attempted")
            }
        }
    }

    private func nativeCopy(_ source: Source, document: String?, range: CFRange?,
                            completion: @escaping (Result<String, Error>) -> Void) {
        SelectionClipboard.copy(sourceIsCurrent: {
            self.sourceIsCurrent(source) && self.sameObservedDocument(document, in: source.element)
                && self.sameObservedRange(range, in: source.element)
        }, sendCopy: { self.sendCommand(key: 8, to: source.application.processIdentifier) }, completion: completion)
    }

    private func restoreSource(_ source: Source, completion: @escaping (Bool) -> Void) {
        guard !source.application.isTerminated,
              sourceState.isCurrent(source.pin, frontmostPID: NSWorkspace.shared.frontmostApplication?.processIdentifier, allowHost: true),
              source.pin.matches(element: source.element == nil ? nil : focusedElement(source.application),
                                 window: source.pin.window == nil ? nil : focusedWindow(source.application)) else {
            completion(false); return
        }
        onRestoreSource?()
        guard source.application.activate(options: []) else { completion(false); return }
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.15) { completion(self.sourceIsCurrent(source)) }
    }

    private func reselectResult(_ capture: Capture, result: String) {
        guard sourceIsCurrent(capture.source), var range = capture.context.resultRange(for: result),
              let element = capture.source.element, let current = selectionRange(element), current.length == 0,
              range.location <= Int.max - range.length, current.location == range.location + range.length else { return }
        if let expected = capture.context.expectedDocument(after: result) {
            guard let document = string(element, kAXValueAttribute), SelectionTextSnapshot.exactlyEqual(document, expected) else { return }
        }
        var writable = DarwinBoolean(false)
        guard AXUIElementIsAttributeSettable(element, kAXSelectedTextRangeAttribute as CFString, &writable) == .success,
              writable.boolValue, let value = AXValueCreate(.cfRange, &range) else { return }
        _ = AXUIElementSetAttributeValue(element, kAXSelectedTextRangeAttribute as CFString, value)
    }

    private func observationMatch(_ context: SelectionCaptureContext, source: Source,
                                  wholeDocumentSelected: Bool = false) -> SelectionObservationMatch {
        context.matches(document: string(source.element, kAXValueAttribute), range: selectionRange(source.element),
                        selectedText: string(source.element, kAXSelectedTextAttribute), wholeDocumentSelected: wholeDocumentSelected)
    }

    private func sourceIsCurrent(_ source: Source) -> Bool {
        !source.application.isTerminated
            && sourceState.isCurrent(source.pin, frontmostPID: NSWorkspace.shared.frontmostApplication?.processIdentifier)
            && source.pin.matches(element: source.element == nil ? nil : focusedElement(source.application),
                                  window: source.pin.window == nil ? nil : focusedWindow(source.application))
    }

    private func sameObservedDocument(_ original: String?, in element: AXUIElement?) -> Bool {
        guard let original = original else { return true }
        return string(element, kAXValueAttribute).map { SelectionTextSnapshot.exactlyEqual(original, $0) } == true
    }

    private func sameObservedRange(_ original: CFRange?, in element: AXUIElement?) -> Bool {
        guard let original = original else { return true }
        return selectionRange(element).map { $0.location == original.location && $0.length == original.length } == true
    }

    private func copyResult(_ text: String) -> Bool {
        NSPasteboard.general.clearContents()
        return NSPasteboard.general.setString(text, forType: .string)
    }

    private func finishCopied() { finish("\(activeAction.resultDescription) copied") }
    private func finish(_ status: String) { isWorking = false; onStatus?(status, false) }

    private func sendCommand(key: CGKeyCode, to application: pid_t) -> Bool {
        guard let source = CGEventSource(stateID: .combinedSessionState),
              let down = CGEvent(keyboardEventSource: source, virtualKey: key, keyDown: true),
              let up = CGEvent(keyboardEventSource: source, virtualKey: key, keyDown: false) else { return false }
        down.flags = .maskCommand
        up.flags = .maskCommand
        down.postToPid(application)
        up.postToPid(application)
        return true
    }

    private func focusedElement(_ app: NSRunningApplication) -> AXUIElement? { appElement(app, attribute: kAXFocusedUIElementAttribute) }
    private func focusedWindow(_ app: NSRunningApplication) -> AXUIElement? { appElement(app, attribute: kAXFocusedWindowAttribute) }
    private func appElement(_ app: NSRunningApplication, attribute: String) -> AXUIElement? {
        var value: CFTypeRef?
        guard AXUIElementCopyAttributeValue(AXUIElementCreateApplication(app.processIdentifier), attribute as CFString, &value) == .success,
              let value = value, CFGetTypeID(value) == AXUIElementGetTypeID() else { return nil }
        return (value as! AXUIElement)
    }

    private func string(_ element: AXUIElement?, _ attribute: String) -> String? {
        guard let element = element else { return nil }
        var value: CFTypeRef?
        guard AXUIElementCopyAttributeValue(element, attribute as CFString, &value) == .success else { return nil }
        return value as? String
    }

    private func selectionRange(_ element: AXUIElement?) -> CFRange? {
        guard let element = element else { return nil }
        var value: CFTypeRef?
        guard AXUIElementCopyAttributeValue(element, kAXSelectedTextRangeAttribute as CFString, &value) == .success,
              let value = value, CFGetTypeID(value) == AXValueGetTypeID() else { return nil }
        let axValue = value as! AXValue
        guard AXValueGetType(axValue) == .cfRange else { return nil }
        var range = CFRange()
        return AXValueGetValue(axValue, .cfRange, &range) ? range : nil
    }

    private func show(_ error: Error) {
        let message = (error as? SelectionActionError)?.message ?? "Could not edit the selection. The original text is unchanged."
#if POLISH_INTEGRATION_TEST
        if let integrationError = integrationError { integrationError(message); return }
#endif
        let alert = NSAlert()
        alert.messageText = "Polish"
        alert.informativeText = message
        alert.addButton(withTitle: "OK")
        alert.runModal()
    }

    private static func request(_ snapshot: SelectionTextSnapshot, action: SelectionAction) throws -> String {
        let body = try JSONEncoder().encode(SelectionRequest(snapshot: snapshot, action: action))
        let result: CoreProcessResult
        do { result = try CoreProcess.run(arguments: ["--polish-selection"], input: body) }
        catch CoreProcessError.timedOut {
            throw SelectionActionError.message("Polish timed out. The original text is unchanged.")
        } catch { throw SelectionActionError.message("Could not start Polish. The original text is unchanged.") }
        guard result.status == 0, let text = result.text else {
            throw SelectionActionError.message("Could not edit the selection. Check your API key and connection in Polish.")
        }
        return text
    }
}
