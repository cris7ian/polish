import AppKit

final class CompletionSound {
    private let preferences: HostPreferences
    private var playing: NSSound?

    init(preferences: HostPreferences) { self.preferences = preferences }

    func play(named resource: String = "ready") {
        guard preferences.load().sound,
              let url = Bundle.main.url(forResource: resource, withExtension: "wav") else { return }
        playing?.stop()
        playing = NSSound(contentsOf: url, byReference: true)
        playing?.play()
    }
}
