import Foundation
import Darwin

final class HostPreferences {
    struct Values: Decodable, Equatable {
        let sound: Bool
        let menuBarIcon: Bool

        private enum CodingKeys: String, CodingKey { case sound; case menuBarIcon = "menu_bar_icon" }

        init(sound: Bool = true, menuBarIcon: Bool = true) {
            self.sound = sound
            self.menuBarIcon = menuBarIcon
        }

        init(from decoder: Decoder) throws {
            let container = try decoder.container(keyedBy: CodingKeys.self)
            sound = (try? container.decodeIfPresent(Bool.self, forKey: .sound)) ?? true
            menuBarIcon = (try? container.decodeIfPresent(Bool.self, forKey: .menuBarIcon)) ?? true
        }
    }

    let url: URL
    private var fileWatcher: DispatchSourceFileSystemObject?
    private var directoryWatcher: DispatchSourceFileSystemObject?
    private var pendingRefresh: DispatchWorkItem?
    private var onChange: ((Values) -> Void)?

    init(url: URL = FileManager.default.homeDirectoryForCurrentUser
        .appendingPathComponent("Library/Application Support/Polish/config.json")) {
        self.url = url
    }

    func load() -> Values {
        guard let data = try? Data(contentsOf: url),
              let values = try? JSONDecoder().decode(Values.self, from: data) else { return Values() }
        return values
    }

    func observe(_ onChange: @escaping (Values) -> Void) {
        self.onChange = onChange
        let directory = url.deletingLastPathComponent()
        try? FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        // The directory watch handles first saves and replacement files; the file watch handles writes.
        directoryWatcher = watch(directory, events: [.write]) { [weak self] in
            self?.watchFile()
            self?.scheduleRefresh()
        }
        watchFile()
        onChange(load())
    }

    private func scheduleRefresh() {
        pendingRefresh?.cancel()
        let refresh = DispatchWorkItem { [weak self] in
            guard let self = self else { return }
            self.onChange?(self.load())
        }
        pendingRefresh = refresh
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.1, execute: refresh)
    }

    private func watchFile() {
        fileWatcher?.cancel()
        fileWatcher = watch(url, events: [.write, .rename, .delete]) { [weak self] in self?.scheduleRefresh() }
    }

    private func watch(_ url: URL, events: DispatchSource.FileSystemEvent,
                       onChange: @escaping () -> Void) -> DispatchSourceFileSystemObject? {
        let descriptor = Darwin.open(url.path, O_EVTONLY)
        guard descriptor >= 0 else { return nil }
        let source = DispatchSource.makeFileSystemObjectSource(fileDescriptor: descriptor,
                                                               eventMask: events, queue: .main)
        source.setEventHandler(handler: onChange)
        source.setCancelHandler { Darwin.close(descriptor) }
        source.resume()
        return source
    }

    deinit {
        pendingRefresh?.cancel()
        fileWatcher?.cancel()
        directoryWatcher?.cancel()
    }
}
