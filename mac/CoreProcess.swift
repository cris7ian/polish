import Foundation
import Darwin

struct CoreProcessResult {
    let status: Int32
    let output: Data
    let failure: Data

    var text: String? { String(data: output, encoding: .utf8).flatMap { $0.isEmpty ? nil : $0 } }
    var failureText: String? { String(data: failure, encoding: .utf8) }
}

enum CoreProcessError: Error {
    case timedOut
}

// Both the Services provider and menu actions use the same helper and pipe lifecycle.
enum CoreProcess {
    static var executableURL: URL {
        Bundle.main.bundleURL.appendingPathComponent("Contents/Helpers/Polish Settings.app/Contents/MacOS/Polish")
    }

    static func run(arguments: [String], input: Data, timeout: TimeInterval = 70,
                    executableURL: URL = executableURL) throws -> CoreProcessResult {
        let task = Process()
        task.executableURL = executableURL
        task.arguments = arguments
        let stdin = Pipe(), stdout = Pipe(), stderr = Pipe()
        task.standardInput = stdin
        task.standardOutput = stdout
        task.standardError = stderr
        // An early-exiting helper must produce an error instead of terminating the host with SIGPIPE.
        _ = fcntl(stdin.fileHandleForWriting.fileDescriptor, F_SETNOSIGPIPE, 1)
        try task.run()

        let state = ProcessState()
        let readers = DispatchGroup()
        readers.enter()
        DispatchQueue.global().async {
            state.output = stdout.fileHandleForReading.readDataToEndOfFile()
            readers.leave()
        }
        readers.enter()
        DispatchQueue.global().async {
            state.failure = stderr.fileHandleForReading.readDataToEndOfFile()
            readers.leave()
        }
        let timer = DispatchSource.makeTimerSource(queue: .global())
        timer.schedule(deadline: .now() + timeout)
        timer.setEventHandler {
            guard task.isRunning else { return }
            state.didTimeout = true
            task.terminate()
            DispatchQueue.global().asyncAfter(deadline: .now() + 2) {
                if task.isRunning { Darwin.kill(task.processIdentifier, SIGKILL) }
            }
        }
        timer.resume()
        defer { timer.cancel() }

        var inputError: Error?
        do { try stdin.fileHandleForWriting.write(contentsOf: input) }
        catch {
            inputError = error
            if task.isRunning { task.terminate() }
        }
        try? stdin.fileHandleForWriting.close()
        task.waitUntilExit()
        timer.cancel()
        readers.wait()
        if state.didTimeout { throw CoreProcessError.timedOut }
        // Preserve a helper's useful stderr/nonzero status even if it closed stdin early.
        if let inputError = inputError, task.terminationStatus == 0 { throw inputError }
        return CoreProcessResult(status: task.terminationStatus, output: state.output, failure: state.failure)
    }
}

private final class ProcessState {
    private let lock = NSLock()
    private var storedOutput = Data()
    private var storedFailure = Data()
    private var storedTimeout = false
    var output: Data {
        get { lock.lock(); defer { lock.unlock() }; return storedOutput }
        set { lock.lock(); defer { lock.unlock() }; storedOutput = newValue }
    }
    var failure: Data {
        get { lock.lock(); defer { lock.unlock() }; return storedFailure }
        set { lock.lock(); defer { lock.unlock() }; storedFailure = newValue }
    }
    var didTimeout: Bool {
        get { lock.lock(); defer { lock.unlock() }; return storedTimeout }
        set { lock.lock(); defer { lock.unlock() }; storedTimeout = newValue }
    }
}
