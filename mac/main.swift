import AppKit
import Foundation

final class Provider: NSObject {
    var receivedRequest = false
    private var completionSound: NSSound?

    private func playCompletionSound() {
        struct SoundSettings: Decodable { let sound: Bool? }
        let config = FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent("Library/Application Support/Polish/config.json")
        let settings = (try? Data(contentsOf: config)).flatMap {
            try? JSONDecoder().decode(SoundSettings.self, from: $0)
        }
        guard settings?.sound ?? true,
              let url = Bundle.main.url(forResource: "ready", withExtension: "wav") else { return }
        completionSound?.stop()
        completionSound = NSSound(contentsOf: url, byReference: true)
        completionSound?.play()
    }

    @objc func polish(_ pasteboard: NSPasteboard, userData: String?, error: AutoreleasingUnsafeMutablePointer<NSString?>) {
        receivedRequest = true
        guard let input = pasteboard.string(forType: .string), !input.isEmpty else {
            error.pointee = "Select text to polish." as NSString
            return
        }
        let task = Process()
        task.executableURL = Bundle.main.bundleURL.appendingPathComponent("Contents/MacOS/polish-core")
        task.arguments = ["--polish"]
        let stdin = Pipe(), stdout = Pipe(), stderr = Pipe()
        task.standardInput = stdin; task.standardOutput = stdout; task.standardError = stderr
        do {
            try task.run()
            let outputGroup = DispatchGroup()
            var result = Data(), failure = Data()
            outputGroup.enter()
            DispatchQueue.global().async { result = stdout.fileHandleForReading.readDataToEndOfFile(); outputGroup.leave() }
            outputGroup.enter()
            DispatchQueue.global().async { failure = stderr.fileHandleForReading.readDataToEndOfFile(); outputGroup.leave() }
            stdin.fileHandleForWriting.write(Data(input.utf8))
            try stdin.fileHandleForWriting.close()
            task.waitUntilExit()
            outputGroup.wait()
            guard task.terminationStatus == 0, let output = String(data: result, encoding: .utf8), !output.isEmpty else {
                error.pointee = (String(data: failure, encoding: .utf8) ?? "Polish failed.") as NSString
                return
            }
            pasteboard.clearContents()
            guard pasteboard.setString(output, forType: .string) else {
                error.pointee = "Could not return the polished text." as NSString
                return
            }
            playCompletionSound()
        } catch let caught {
            error.pointee = "Could not start Polish: \(caught)" as NSString
        }
    }
}
final class Delegate: NSObject, NSApplicationDelegate {
    let provider = Provider()
    func applicationDidFinishLaunching(_ notification: Notification) {
        NSApp.servicesProvider = provider
        DispatchQueue.main.asyncAfter(deadline: .now() + 1.0) { [weak self] in
            guard let self = self, !self.provider.receivedRequest else { return }
            self.launchSettings()
        }
    }
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
