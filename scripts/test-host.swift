import Foundation
import Darwin

@main
struct HostTests {
    static func main() throws {
        let arguments = CommandLine.arguments
        if arguments.count > 2, arguments[1] == "--fixture" {
            runFixture(arguments[2])
            return
        }
        let binary = URL(fileURLWithPath: arguments[0])
        let largeInput = Data(repeating: 0x63, count: 1024 * 1024)
        let duplex = try CoreProcess.run(arguments: ["--fixture", "duplex"], input: largeInput,
                                         timeout: 5, executableURL: binary)
        precondition(duplex.status == 0)
        precondition(duplex.output == Data(repeating: 0x61, count: 256 * 1024) + largeInput)
        precondition(duplex.failure == Data(repeating: 0x62, count: 256 * 1024))

        let failed = try CoreProcess.run(arguments: ["--fixture", "failure"], input: Data(),
                                         timeout: 5, executableURL: binary)
        precondition(failed.status == 7)
        precondition(failed.failureText == "fixture failed")
        precondition(failed.text == nil)

        let earlyExit = try CoreProcess.run(arguments: ["--fixture", "failure"], input: largeInput,
                                            timeout: 5, executableURL: binary)
        precondition(earlyExit.status != 0)
        precondition(earlyExit.failureText == "fixture failed")

        let started = Date()
        do {
            _ = try CoreProcess.run(arguments: ["--fixture", "hang"], input: largeInput,
                                    timeout: 0.1, executableURL: binary)
            preconditionFailure("Hung helper did not time out")
        } catch CoreProcessError.timedOut { }
        precondition(Date().timeIntervalSince(started) < 5)

        do {
            _ = try CoreProcess.run(arguments: [], input: Data(), executableURL: URL(fileURLWithPath: "/nonexistent-polish-fixture"))
            preconditionFailure("Missing helper did not fail")
        } catch { }

        let config = URL(fileURLWithPath: arguments[1]).appendingPathComponent("config.json")
        let preferences = HostPreferences(url: config)
        precondition(preferences.load() == HostPreferences.Values())
        try Data("{\"sound\":false,\"menu_bar_icon\":false,\"model\":\"fixture\"}".utf8).write(to: config)
        precondition(preferences.load() == HostPreferences.Values(sound: false, menuBarIcon: false))
        try Data("{\"sound\":false}".utf8).write(to: config)
        precondition(preferences.load() == HostPreferences.Values(sound: false, menuBarIcon: true))
        try Data("{\"sound\":\"invalid\",\"menu_bar_icon\":false}".utf8).write(to: config)
        precondition(preferences.load() == HostPreferences.Values(sound: true, menuBarIcon: false))
        try Data("invalid JSON".utf8).write(to: config)
        precondition(preferences.load() == HostPreferences.Values())
        print("Host process draining, large input, early exit, timeout, and preference checks passed.")
    }

    private static func runFixture(_ mode: String) {
        switch mode {
        case "duplex":
            FileHandle.standardOutput.write(Data(repeating: 0x61, count: 256 * 1024))
            FileHandle.standardError.write(Data(repeating: 0x62, count: 256 * 1024))
            FileHandle.standardOutput.write(FileHandle.standardInput.readDataToEndOfFile())
        case "failure":
            FileHandle.standardError.write(Data("fixture failed".utf8))
            exit(7)
        case "hang":
            signal(SIGTERM, SIG_IGN)
            while true { pause() }
        default: preconditionFailure("Unknown fixture")
        }
    }
}
