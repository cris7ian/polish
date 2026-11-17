import AppKit
#if canImport(Sparkle)
import Sparkle
#endif

final class Delegate: NSObject, NSApplicationDelegate, NSMenuDelegate {
    private let preferences = HostPreferences()
    private lazy var completionSound = CompletionSound(preferences: preferences)
    private(set) lazy var provider = Provider(completionSound: completionSound)
    private var statusItem: NSStatusItem?
    private let selectionActions = SelectionActions()
#if canImport(Sparkle)
    private lazy var updater = SPUStandardUpdaterController(startingUpdater: true, updaterDelegate: nil, userDriverDelegate: nil)
#endif
    private var selectionMenuItems: [NSMenuItem] = []
    private let selectionStatus = NSMenuItem(title: "Select text in another app", action: nil, keyEquivalent: "")
    private func refreshMenuBarItem(_ preferences: HostPreferences.Values) {
        if preferences.menuBarIcon {
            if statusItem == nil { installMenuBarItem() }
        } else if let item = statusItem {
            NSStatusBar.system.removeStatusItem(item)
            statusItem = nil
        }
    }

    private func installMenuBarItem() {
        let item = NSStatusBar.system.statusItem(withLength: NSStatusItem.squareLength)
        statusItem = item
        item.button?.image = Self.menuBarFlag()
        item.button?.toolTip = "Polish"
        item.button?.setAccessibilityLabel("Polish")

        let menu = NSMenu()
        menu.delegate = self
        menu.autoenablesItems = false
        selectionMenuItems = []
        for (title, action) in [("Polish", #selector(polishSelection)),
                                ("Reroll", #selector(rerollSelection))] {
            let entry = menu.addItem(withTitle: title, action: action, keyEquivalent: "")
            entry.target = self
            entry.isEnabled = !selectionActions.isWorking
            selectionMenuItems.append(entry)
        }
        selectionStatus.isEnabled = false
        menu.addItem(selectionStatus)
        menu.addItem(.separator())
        let open = menu.addItem(withTitle: "Open Polish", action: #selector(openFromMenu), keyEquivalent: "")
        open.target = self
#if canImport(Sparkle)
        let update = menu.addItem(withTitle: "Check for Updates…", action: #selector(checkForUpdates), keyEquivalent: "")
        update.target = self
#endif
        menu.addItem(.separator())
        let quit = menu.addItem(withTitle: "Quit Polish", action: #selector(quitFromMenu), keyEquivalent: "q")
        quit.target = self
        item.menu = menu
    }

    private static func menuBarFlag() -> NSImage {
        let image = NSImage(size: NSSize(width: 18, height: 18), flipped: false) { rect in
            NSGraphicsContext.saveGraphicsState()
            defer { NSGraphicsContext.restoreGraphicsState() }
            let flag = NSBezierPath(roundedRect: rect.insetBy(dx: 1, dy: 1), xRadius: 3.5, yRadius: 3.5)
            flag.addClip()
            NSColor.black.withAlphaComponent(0.4).setFill()
            flag.fill()

            // Match the app flag's wave, with a solid lower stripe and a translucent upper stripe.
            let lowerStripe = NSBezierPath()
            lowerStripe.move(to: NSPoint(x: 1, y: 1))
            lowerStripe.line(to: NSPoint(x: 17, y: 1))
            lowerStripe.line(to: NSPoint(x: 17, y: 9))
            lowerStripe.curve(to: NSPoint(x: 1, y: 9),
                              controlPoint1: NSPoint(x: 11.7, y: 11.2),
                              controlPoint2: NSPoint(x: 6.9, y: 6.8))
            lowerStripe.close()
            // Copy replaces the translucent base so the lower stripe remains fully opaque.
            NSGraphicsContext.current?.compositingOperation = .copy
            NSColor.black.setFill()
            lowerStripe.fill()
            return true
        }
        // AppKit renders the alpha mask white on dark menu bars and dark on light menu bars.
        image.isTemplate = true
        image.accessibilityDescription = "Polish"
        return image
    }

    func menuWillOpen(_ menu: NSMenu) { selectionActions.rememberSource() }

    @objc private func polishSelection(_ sender: Any?) { selectionActions.run(action: .polish) }
    @objc private func rerollSelection(_ sender: Any?) { selectionActions.run(action: .reroll) }

    @objc private func openFromMenu(_ sender: Any?) { launchSettings() }

#if canImport(Sparkle)
    @objc private func checkForUpdates(_ sender: Any?) { updater.checkForUpdates(sender) }

    func applicationShouldTerminate(_ sender: NSApplication) -> NSApplication.TerminateReply {
        if !NSRunningApplication.runningApplications(withBundleIdentifier: "com.cristian.polish.settings").isEmpty || selectionActions.isWorking {
            let alert = NSAlert()
            alert.messageText = "Close the Polish editor before updating"
            alert.informativeText = "Copy any text you want to keep, close the editor, and wait for any menu action to finish. Then try again."
            alert.addButton(withTitle: "OK")
            alert.runModal()
            return .terminateCancel
        }
        return .terminateNow
    }
#endif

    @objc private func quitFromMenu(_ sender: Any?) {
        for application in NSRunningApplication.runningApplications(withBundleIdentifier: "com.cristian.polish.settings") {
            application.terminate()
        }
        NSApp.terminate(sender)
    }

    func applicationDidFinishLaunching(_ notification: Notification) {
#if canImport(Sparkle)
        _ = updater
#endif
        NSApp.servicesProvider = provider
        selectionActions.onRestoreSource = { [weak self] in self?.statusItem?.menu?.cancelTracking() }
        selectionActions.onCopied = { [weak self] action in
            self?.completionSound.play(named: action.soundResource)
        }
        selectionActions.onStatus = { [weak self] status, working in
            self?.selectionStatus.title = status
            self?.selectionMenuItems.forEach { $0.isEnabled = !working }
            self?.statusItem?.button?.toolTip = working ? status : "Polish"
        }
#if POLISH_INTEGRATION_TEST
        if runIntegrationIfRequested() { return }
#endif
        preferences.observe { [weak self] values in self?.refreshMenuBarItem(values) }
        DispatchQueue.main.asyncAfter(deadline: .now() + 1.0) { [weak self] in
            guard let self = self, !self.provider.receivedRequest else { return }
            self.launchSettings()
        }
    }
#if POLISH_INTEGRATION_TEST
    private func runIntegrationIfRequested() -> Bool {
        let arguments = CommandLine.arguments
        guard let index = arguments.firstIndex(of: "--integration-selection") else { return false }
        guard arguments.indices.contains(index + 1), let action = SelectionAction(rawValue: arguments[index + 1]) else {
            print("integration: invalid action")
            exit(2)
        }
        guard let expectedIndex = arguments.firstIndex(of: "--expected-source-bundle"),
              arguments.indices.contains(expectedIndex + 1) else {
            print("integration: expected source bundle required")
            exit(2)
        }
        let expectedBundle = arguments[expectedIndex + 1]
        guard expectedBundle != "com.openai.codex",
              NSWorkspace.shared.frontmostApplication?.bundleIdentifier == expectedBundle else {
            print("integration: source app is outside the permitted test scope")
            exit(2)
        }
        selectionActions.integrationError = { message in
            // SelectionActionError messages contain fixed UI text, never source or response content.
            print("integration error: \(message)")
            fflush(stdout)
            exit(1)
        }
        selectionActions.onStatus = { status, working in
            print("integration status: \(status)")
            fflush(stdout)
            if !working {
                DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) { exit(0) }
            }
        }
        selectionActions.onCopied = { [weak self] action in
            self?.completionSound.play(named: action.soundResource)
            print("integration: result copied")
            fflush(stdout)
        }
        selectionActions.integrationWithoutAX = arguments.contains("--without-ax")
        selectionActions.rememberSource()
        if arguments.contains("--activate-host") { NSApp.activate(ignoringOtherApps: true) }
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.2) { [weak self] in
            self?.selectionActions.run(action: action)
        }
        DispatchQueue.main.asyncAfter(deadline: .now() + 85) {
            print("integration: timed out")
            fflush(stdout)
            exit(124)
        }
        return true
    }
#endif

    func applicationShouldHandleReopen(_ sender: NSApplication, hasVisibleWindows flag: Bool) -> Bool { launchSettings(); return false }
    func launchSettings() {
        let url = Bundle.main.bundleURL.appendingPathComponent("Contents/Helpers/Polish Settings.app")
        NSWorkspace.shared.openApplication(at: url, configuration: NSWorkspace.OpenConfiguration()) { _, error in
            if let error = error { NSLog("Could not open Polish settings: %@", error.localizedDescription) }
        }
    }
}
let app = NSApplication.shared
let delegate = Delegate()
app.delegate = delegate
app.setActivationPolicy(.accessory)
app.run()
