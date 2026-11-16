import AppKit

final class Provider: NSObject {
    var receivedRequest = false
    private let completionSound: CompletionSound

    init(completionSound: CompletionSound) { self.completionSound = completionSound }

    @objc func polish(_ pasteboard: NSPasteboard, userData: String?, error: AutoreleasingUnsafeMutablePointer<NSString?>) {
        runService(pasteboard, argument: "--polish", error: error)
    }

    @objc func polishTweet(_ pasteboard: NSPasteboard, userData: String?, error: AutoreleasingUnsafeMutablePointer<NSString?>) {
        runService(pasteboard, argument: "--tweet", error: error)
    }

    private func runService(_ pasteboard: NSPasteboard, argument: String, error: AutoreleasingUnsafeMutablePointer<NSString?>) {
        receivedRequest = true
        guard let input = pasteboard.string(forType: .string), !input.isEmpty else {
            error.pointee = "Select text to polish." as NSString
            return
        }
        do {
            let result = try CoreProcess.run(arguments: [argument], input: Data(input.utf8))
            guard result.status == 0, let output = result.text else {
                error.pointee = (result.failureText ?? "Polish failed.") as NSString
                return
            }
            pasteboard.clearContents()
            guard pasteboard.setString(output, forType: .string) else {
                error.pointee = "Could not return the polished text." as NSString
                return
            }
            completionSound.play()
        } catch CoreProcessError.timedOut {
            error.pointee = "Polish timed out. The original text is unchanged." as NSString
        } catch let caught {
            error.pointee = "Could not start Polish: \(caught)" as NSString
        }
    }
}
