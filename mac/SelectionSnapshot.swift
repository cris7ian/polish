import Foundation

enum SelectionAction: String, CaseIterable, Encodable {
    case polish
    case reroll

    var workingStatus: String { self == .polish ? "Polishing selection…" : "Rerolling selection…" }
    var resultDescription: String { self == .polish ? "Polished text" : "Rerolled text" }
    var soundResource: String { self == .polish ? "ready" : "rework" }
}

struct SelectionRequest: Encodable {
    let document: String
    let selection_start_utf16: Int
    let selection_length_utf16: Int
    let action: SelectionAction

    init(snapshot: SelectionTextSnapshot, action: SelectionAction) {
        document = snapshot.document
        selection_start_utf16 = snapshot.selection_start_utf16
        selection_length_utf16 = snapshot.selection_length_utf16
        self.action = action
    }
}

struct SelectionTextSnapshot {
    let document: String
    let selection_start_utf16: Int
    let selection_length_utf16: Int

    init(document: String, range: CFRange, selectedText: String) throws {
        let count = document.utf16.count
        guard range.location >= 0, range.length >= 0,
              range.location <= count, range.length <= count - range.location,
              Self.isUTF16Boundary(document, range.location),
              Self.isUTF16Boundary(document, range.location + range.length),
              Self.exactlyEqual((document as NSString).substring(with: NSRange(location: range.location, length: range.length)), selectedText) else {
            throw SelectionActionError.message("The app did not provide a valid text selection. The original text is unchanged.")
        }
        self.document = document
        selection_start_utf16 = range.location
        selection_length_utf16 = range.length
    }

    static func exactlyEqual(_ lhs: String, _ rhs: String) -> Bool {
        lhs.utf16.elementsEqual(rhs.utf16)
    }

    private static func isUTF16Boundary(_ text: String, _ offset: Int) -> Bool {
        let units = text.utf16
        guard offset > 0, offset < units.count else { return true }
        let index = units.index(units.startIndex, offsetBy: offset)
        return !(0xDC00...0xDFFF).contains(units[index])
            || !(0xD800...0xDBFF).contains(units[units.index(before: index)])
    }

    func resultRange(for result: String) -> CFRange {
        CFRange(location: selection_length_utf16 == 0 ? 0 : selection_start_utf16,
                length: result.utf16.count)
    }

    func shouldReselectResult(_ result: String, currentRange: CFRange) -> Bool {
        let inserted = resultRange(for: result)
        return inserted.length > 0 && currentRange.length == 0
            && currentRange.location == inserted.location + inserted.length
    }

    func replacingTarget(with result: String) -> String {
        if selection_length_utf16 == 0 { return result }
        let range = NSRange(location: selection_start_utf16, length: selection_length_utf16)
        return (document as NSString).replacingCharacters(in: range, with: result)
    }

    func matches(document: String, range: CFRange) -> Bool {
        Self.exactlyEqual(self.document, document) && selection_start_utf16 == range.location && selection_length_utf16 == range.length
    }
}

enum SelectionActionError: Error {
    case message(String)
    var message: String {
        switch self { case .message(let text): return text }
    }
}


// Keep source observations separate from the normalized model payload.
enum SelectionObservationMatch { case confirmed, needsNativeCopy, changed }

struct SelectionCaptureContext {
    let text: SelectionTextSnapshot
    let sourceDocument: String?
    let sourceRange: CFRange?
    let selectedText: String
    let hasFullContext: Bool
    var usesWholeDocument: Bool { hasFullContext && text.selection_length_utf16 == 0 }

    static func resolve(document: String?, range: CFRange?, selectedText: String?) throws -> SelectionCaptureContext {
        if let document = document, !document.isEmpty, let range = range {
            let selected = selectedText ?? (range.length == 0 ? "" : nil)
            if let selected = selected,
               let text = try? SelectionTextSnapshot(document: document, range: range, selectedText: selected) {
                return SelectionCaptureContext(text: text, sourceDocument: document, sourceRange: range,
                                               selectedText: selected, hasFullContext: true)
            }
        }
        guard let selected = selectedText, !selected.isEmpty else {
            throw SelectionActionError.message("Select some text first.")
        }
        let text = try SelectionTextSnapshot(document: selected,
                                              range: CFRange(location: 0, length: selected.utf16.count),
                                              selectedText: selected)
        let reliableRange = range.flatMap { $0.location >= 0 && $0.length == selected.utf16.count && $0.location <= Int.max - $0.length ? $0 : nil }
        return SelectionCaptureContext(text: text, sourceDocument: document, sourceRange: reliableRange,
                                       selectedText: selected, hasFullContext: false)
    }

    func matches(document: String?, range: CFRange?, selectedText: String?, wholeDocumentSelected: Bool = false) -> SelectionObservationMatch {
        if let original = sourceDocument {
            guard let document = document, SelectionTextSnapshot.exactlyEqual(original, document) else { return .changed }
        }
        if wholeDocumentSelected {
            guard usesWholeDocument, let range = range, range.location == 0,
                  range.length == text.document.utf16.count else { return .changed }
            return .confirmed
        }
        if let original = sourceRange {
            guard let range = range, original.location == range.location, original.length == range.length else { return .changed }
        }
        if hasFullContext { return .confirmed }
        guard let selected = selectedText else { return .needsNativeCopy }
        return SelectionTextSnapshot.exactlyEqual(self.selectedText, selected) ? .confirmed : .changed
    }

    func expectedDocument(after result: String) -> String? {
        hasFullContext ? text.replacingTarget(with: result) : nil
    }

    func resultRange(for result: String) -> CFRange? {
        guard let range = sourceRange else { return nil }
        return CFRange(location: usesWholeDocument ? 0 : range.location, length: result.utf16.count)
    }
}

