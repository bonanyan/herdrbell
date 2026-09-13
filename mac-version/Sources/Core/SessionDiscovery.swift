import Foundation

struct DiscoveredSession: Hashable, Sendable {
    let name: String
    let socketPath: String
}

final class SessionDiscovery: @unchecked Sendable {
    var onChange: (@Sendable ([DiscoveredSession]) -> Void)?

    private let queue = DispatchQueue(label: "dev.herdr.bell.discovery")
    private let fileManager = FileManager.default
    private var sources: [String: DispatchSourceFileSystemObject] = [:]
    private lazy var rescanTimer = DispatchSource.makeTimerSource(queue: queue)
    private var scanPending = false
    private var lastReported: [DiscoveredSession] = []
    private var started = false
    private var stopped = false

    private var configDir: URL {
        fileManager.homeDirectoryForCurrentUser
            .appendingPathComponent(".config/herdr")
    }

    private var sessionsDir: URL {
        configDir.appendingPathComponent("sessions")
    }

    func start() {
        queue.async { [self] in
            guard !started else { return }
            started = true
            watch(configDir.path)
            watch(sessionsDir.path)
            rescanTimer.schedule(deadline: .now() + 60, repeating: 60)
            rescanTimer.setEventHandler { [weak self] in self?.scheduleScan() }
            rescanTimer.resume()
            scanLocked()
        }
    }

    func stop() {
        queue.async { [self] in
            stopped = true
            for source in sources.values {
                source.cancel()
            }
            sources.removeAll()
            rescanTimer.cancel()
        }
    }

    private func scheduleScan() {
        queue.async { [self] in
            scheduleScanLocked()
        }
    }

    private func scheduleScanLocked() {
        guard !scanPending, !stopped else { return }
        scanPending = true
        queue.asyncAfter(deadline: .now() + 0.2) { [weak self] in
            guard let self else { return }
            self.scanPending = false
            self.scanLocked()
        }
    }

    private func scanLocked() {
        guard !stopped else { return }
        watch(sessionsDir.path)
        var found: [DiscoveredSession] = []
        let defaultSocket = configDir.appendingPathComponent("herdr.sock")
        if fileManager.fileExists(atPath: defaultSocket.path) {
            found.append(DiscoveredSession(name: "default", socketPath: defaultSocket.path))
        }
        if let entries = try? fileManager.contentsOfDirectory(atPath: sessionsDir.path) {
            for entry in entries.sorted() {
                let socketPath = sessionsDir
                    .appendingPathComponent(entry)
                    .appendingPathComponent("herdr.sock")
                if fileManager.fileExists(atPath: socketPath.path) {
                    found.append(DiscoveredSession(name: entry, socketPath: socketPath.path))
                }
            }
        }
        if found != lastReported {
            lastReported = found
            onChange?(found)
        }
    }

    private func watch(_ path: String) {
        guard sources[path] == nil else { return }
        let fd = open(path, O_EVTONLY)
        guard fd >= 0 else { return }
        let source = DispatchSource.makeFileSystemObjectSource(
            fileDescriptor: fd,
            eventMask: [.write, .rename, .delete, .link, .attrib],
            queue: queue
        )
        source.setEventHandler { [weak self] in self?.scheduleScan() }
        source.setCancelHandler { close(fd) }
        source.resume()
        sources[path] = source
    }
}
