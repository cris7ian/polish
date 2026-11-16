import ApplicationServices
import Foundation

struct SelectionSourcePin {
    let pid: pid_t
    let generation: UInt64
    let element: AXUIElement?
    let window: AXUIElement?

    func matches(element currentElement: AXUIElement?, window currentWindow: AXUIElement?) -> Bool {
        if let element = element {
            guard let current = currentElement, CFEqual(element, current) else { return false }
        }
        if let window = window {
            guard let current = currentWindow, CFEqual(window, current) else { return false }
        }
        return true
    }
}

// The menu host can become active; keep the last external app independent of AX availability.
struct SelectionSourceState {
    let hostPID: pid_t
    private(set) var lastExternalPID: pid_t?
    private var generation: UInt64 = 0

    init(hostPID: pid_t) { self.hostPID = hostPID }

    mutating func recordActivation(_ pid: pid_t) {
        guard pid != hostPID, pid != lastExternalPID else { return }
        lastExternalPID = pid
        generation &+= 1
    }

    mutating func candidatePID(frontmostPID: pid_t?) -> pid_t? {
        if let pid = frontmostPID { recordActivation(pid) }
        return lastExternalPID
    }

    mutating func pin(frontmostPID: pid_t?, element: AXUIElement?, window: AXUIElement?) -> SelectionSourcePin? {
        guard let pid = candidatePID(frontmostPID: frontmostPID) else { return nil }
        return SelectionSourcePin(pid: pid, generation: generation, element: element, window: window)
    }

    mutating func isCurrent(_ pin: SelectionSourcePin, frontmostPID: pid_t?, allowHost: Bool = false) -> Bool {
        if let pid = frontmostPID { recordActivation(pid) }
        return pin.pid == lastExternalPID && pin.generation == generation
            && (frontmostPID == pin.pid || (allowHost && frontmostPID == hostPID))
    }
}
