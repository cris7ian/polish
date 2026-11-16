import AppKit
import ApplicationServices

@main
struct SelectionActionTests {
    static func main() throws {
        var sources = SelectionSourceState(hostPID: 99)
        guard let opaqueSource = sources.pin(frontmostPID: 42, element: nil, window: nil) else {
            preconditionFailure("An app with no focused AX element must still be a capture candidate")
        }
        precondition(opaqueSource.pid == 42)
        precondition(opaqueSource.matches(element: nil, window: nil))
        let knownElement = AXUIElementCreateApplication(42)
        let knownWindow = AXUIElementCreateApplication(43)
        let knownSource = SelectionSourcePin(pid: 42, generation: 1, element: knownElement, window: knownWindow)
        precondition(knownSource.matches(element: AXUIElementCreateApplication(42), window: AXUIElementCreateApplication(43)))
        precondition(!knownSource.matches(element: nil, window: knownWindow))
        precondition(!knownSource.matches(element: AXUIElementCreateApplication(55), window: knownWindow))
        precondition(!knownSource.matches(element: knownElement, window: nil))
        let hostFrontmost = sources.pin(frontmostPID: 99, element: nil, window: nil)
        precondition(hostFrontmost?.pid == 42)
        precondition(sources.isCurrent(opaqueSource, frontmostPID: 99, allowHost: true))
        precondition(!sources.isCurrent(opaqueSource, frontmostPID: 99))
        precondition(sources.isCurrent(opaqueSource, frontmostPID: 42))
        sources.recordActivation(55)
        precondition(!sources.isCurrent(opaqueSource, frontmostPID: 42))
        var noHistory = SelectionSourceState(hostPID: 99)
        precondition(noHistory.pin(frontmostPID: 99, element: nil, window: nil) == nil)

        let selectedOnly = try SelectionCaptureContext.resolve(document: nil, range: nil, selectedText: "selected café")
        precondition(selectedOnly.text.document == "selected café")
        precondition(selectedOnly.text.selection_start_utf16 == 0)
        precondition(selectedOnly.text.selection_length_utf16 == "selected café".utf16.count)
        precondition(!selectedOnly.usesWholeDocument)

        let sourceOffset = try SelectionCaptureContext.resolve(document: nil, range: CFRange(location: 42, length: 4), selectedText: "café")
        precondition(sourceOffset.text.selection_start_utf16 == 0)
        precondition(sourceOffset.sourceRange?.location == 42)
        precondition(sourceOffset.resultRange(for: "🙂")?.location == 42)
        precondition(sourceOffset.resultRange(for: "🙂")?.length == 2)
        precondition(!sourceOffset.usesWholeDocument)
        precondition(sourceOffset.matches(document: nil, range: CFRange(location: 42, length: 4), selectedText: "café") == .confirmed)
        precondition(sourceOffset.matches(document: nil, range: CFRange(location: 12, length: 4), selectedText: "café") == .changed)
        precondition(sourceOffset.matches(document: nil, range: CFRange(location: 42, length: 4), selectedText: "cafe") == .changed)
        precondition(sourceOffset.matches(document: nil, range: CFRange(location: 42, length: 4), selectedText: nil) == .needsNativeCopy)
        let missingRange = try SelectionCaptureContext.resolve(document: "all context", range: nil, selectedText: "context")
        precondition(!missingRange.hasFullContext && !missingRange.usesWholeDocument)
        precondition(missingRange.text.document == "context")
        precondition(missingRange.matches(document: "changed context", range: nil, selectedText: "context") == .changed)
        let invalidOffset = try SelectionCaptureContext.resolve(document: nil, range: CFRange(location: Int.max, length: 4), selectedText: "café")
        precondition(invalidOffset.sourceRange == nil)
        do {
            _ = try SelectionCaptureContext.resolve(document: nil, range: nil, selectedText: nil)
            preconditionFailure("Missing selection accepted")
        } catch { }
        try testClipboardFallback()

        let document = "First café. 🙂 Last café."
        let range = (document as NSString).range(of: "café", options: .backwards)
        let full = try SelectionCaptureContext.resolve(document: document,
                                                        range: CFRange(location: range.location, length: range.length),
                                                        selectedText: "café")
        precondition(full.hasFullContext && !full.usesWholeDocument)
        precondition(full.text.document == document)
        precondition(full.text.selection_start_utf16 == range.location)
        let whole = try SelectionCaptureContext.resolve(document: document, range: CFRange(location: 3, length: 0), selectedText: "")
        precondition(whole.usesWholeDocument)
        precondition(whole.resultRange(for: "new")?.location == 0)
        let selection = try SelectionTextSnapshot(document: document,
                                                  range: CFRange(location: range.location, length: range.length),
                                                  selectedText: "café")
        precondition(selection.replacingTarget(with: "coffee") == "First café. 🙂 Last coffee.")
        let replacementRange = selection.resultRange(for: "🙂 café")
        precondition(replacementRange.location == range.location)
        precondition(replacementRange.length == 7)
        precondition(selection.shouldReselectResult("🙂 café", currentRange: CFRange(location: range.location + 7, length: 0)))
        precondition(!selection.shouldReselectResult("🙂 café", currentRange: replacementRange))
        precondition(!selection.shouldReselectResult("🙂 café", currentRange: CFRange(location: 0, length: 0)))
        precondition(!selection.shouldReselectResult("🙂 café", currentRange: CFRange(location: range.location, length: 1)))
        precondition(!selection.shouldReselectResult("", currentRange: CFRange(location: range.location, length: 0)))
        precondition(selection.matches(document: document, range: CFRange(location: range.location, length: range.length)))
        precondition(!selection.matches(document: document + "!", range: CFRange(location: range.location, length: range.length)))
        precondition(!selection.matches(document: document, range: CFRange(location: 6, length: range.length)))
        precondition(!selection.matches(document: document, range: CFRange(location: range.location, length: 0)))

        let canonical = try SelectionTextSnapshot(document: "x é", range: CFRange(location: 0, length: 1), selectedText: "x")
        precondition(!canonical.matches(document: "x e\u{301}", range: CFRange(location: 0, length: 1)))
        precondition(!SelectionTextSnapshot.exactlyEqual("é", "e\u{301}"))

        let collapsed = try SelectionTextSnapshot(document: document, range: CFRange(location: 0, length: 0), selectedText: "")
        precondition(collapsed.replacingTarget(with: "Whole document") == "Whole document")
        let wholeDocumentRange = collapsed.resultRange(for: "🙂")
        precondition(wholeDocumentRange.location == 0 && wholeDocumentRange.length == 2)
        let collapsedAtEnd = try SelectionTextSnapshot(document: document,
                                                        range: CFRange(location: document.utf16.count, length: 0),
                                                        selectedText: "")
        precondition(collapsedAtEnd.resultRange(for: "new").location == 0)
        precondition(SelectionAction.allCases.count == 2)
        for action in SelectionAction.allCases {
            let request = SelectionRequest(snapshot: selection, action: action)
            let payload = try JSONSerialization.jsonObject(with: JSONEncoder().encode(request)) as! [String: Any]
            precondition(payload["document"] as? String == document)
            precondition(payload["selection_start_utf16"] as? Int == range.location)
            precondition(payload["selection_length_utf16"] as? Int == range.length)
            precondition(payload["action"] as? String == (action == .polish ? "polish" : "reroll"))
            precondition(payload.count == 4)
        }
        precondition(SelectionAction.polish.soundResource == "ready")
        precondition(SelectionAction.reroll.soundResource == "rework")
        precondition(SelectionAction.polish.workingStatus == "Polishing selection…")
        precondition(SelectionAction.reroll.workingStatus == "Rerolling selection…")

        func reject(_ document: String, _ location: Int, _ length: Int, _ selected: String) {
            do {
                _ = try SelectionTextSnapshot(document: document, range: CFRange(location: location, length: length), selectedText: selected)
                preconditionFailure("Invalid selection accepted at \(location), length \(length), document length \(document.utf16.count)")
            } catch { }
        }
        reject("é", 0, 1, "e\u{301}")
        reject("🙂", 1, 1, "") // Splits a UTF16 surrogate pair.
        reject("🙂", 0, 1, "")
        reject(document, -1, 1, "")
        reject(document, 0, -1, "")
        reject(document, Int.max, 1, "")
        reject(document, 0, Int.max, "")
        reject(document, range.location, range.length, "different")
        reject(document, document.utf16.count + 1, 0, "")
        print("Selection capture, Unicode, stale-document, replacement, and CLI-schema checks passed.")
    }

    private static func testClipboardFallback() throws {
        let board = NSPasteboard.withUniqueName()
        defer { board.releaseGlobally() }
        board.clearContents()
        board.setString("old clipboard", forType: .string)
        var copied: Result<String, Error>?
        SelectionClipboard.copy(pasteboard: board, timeout: 0, sourceIsCurrent: { true }, sendCopy: {
            board.clearContents()
            return board.setString("selected text", forType: .string)
        }) { copied = $0 }
        let selection = try copied!.get()
        precondition(selection == "selected text")
        precondition(board.string(forType: .string) == "selected text")

        copied = nil
        SelectionClipboard.copy(pasteboard: board, timeout: 0, sourceIsCurrent: { true }, sendCopy: { true }) { copied = $0 }
        if case .success = copied! { preconditionFailure("Old clipboard accepted as a new selection") }
        precondition(board.string(forType: .string) == nil)

        copied = nil
        SelectionClipboard.copy(pasteboard: board, timeout: 0, sourceIsCurrent: { false }, sendCopy: {
            preconditionFailure("Copy sent after source changed")
        }) { copied = $0 }
        if case .success = copied! { preconditionFailure("Changed source accepted") }
    }
}
