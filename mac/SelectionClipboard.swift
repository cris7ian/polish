import AppKit

// Read a fresh native Copy result; prior clipboard contents are intentionally replaced.
enum SelectionClipboard {
    static func copy(pasteboard: NSPasteboard = .general, timeout: TimeInterval = 0.8,
                     sourceIsCurrent: @escaping () -> Bool, sendCopy: () -> Bool,
                     completion: @escaping (Result<String, Error>) -> Void) {
        guard sourceIsCurrent() else {
            completion(.failure(SelectionActionError.message("The source app or selection changed.")))
            return
        }
        let generation = pasteboard.clearContents()
        guard sendCopy() else {
            completion(.failure(SelectionActionError.message("Could not copy the selection.")))
            return
        }
        poll(pasteboard, generation: generation, deadline: Date().addingTimeInterval(timeout),
             sourceIsCurrent: sourceIsCurrent, completion: completion)
    }

    private static func poll(_ pasteboard: NSPasteboard, generation: Int, deadline: Date,
                             sourceIsCurrent: @escaping () -> Bool,
                             completion: @escaping (Result<String, Error>) -> Void) {
        guard sourceIsCurrent() else {
            completion(.failure(SelectionActionError.message("The source app or selection changed.")))
            return
        }
        if pasteboard.changeCount != generation, let text = pasteboard.string(forType: .string), !text.isEmpty {
            completion(.success(text))
        } else if Date() >= deadline {
            completion(.failure(SelectionActionError.message("Select some text first. Copy did not return a text selection.")))
        } else {
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.04) {
                poll(pasteboard, generation: generation, deadline: deadline,
                     sourceIsCurrent: sourceIsCurrent, completion: completion)
            }
        }
    }
}
