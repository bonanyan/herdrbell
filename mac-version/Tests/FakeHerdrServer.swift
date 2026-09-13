import Foundation

@testable import HerdrBell

final class FakeHerdrServer: @unchecked Sendable {
    let path: String

    private let lock = NSLock()
    private var listenerFD: Int32 = -1
    private var clientFDs: [Int32] = []
    private var subscriberFDs: [Int32] = []
    private var running = false

    init(path: String) {
        self.path = path
    }

    func start() throws {
        unlink(path)
        let fd = socket(AF_UNIX, SOCK_STREAM, 0)
        guard fd >= 0 else {
            throw HerdrSocketError.connectFailed(String(cString: strerror(errno)))
        }
        var addr = sockaddr_un()
        addr.sun_family = sa_family_t(AF_UNIX)
        let pathBytes = path.utf8CString
        guard pathBytes.count <= MemoryLayout.size(ofValue: addr.sun_path) else {
            close(fd)
            throw HerdrSocketError.connectFailed("socket path too long")
        }
        pathBytes.withUnsafeBytes { (rawBuffer: UnsafeRawBufferPointer) in
            withUnsafeMutablePointer(to: &addr.sun_path) { pointer in
                UnsafeMutableRawPointer(pointer).copyMemory(from: rawBuffer.baseAddress!, byteCount: rawBuffer.count)
            }
        }
        let bindResult = withUnsafePointer(to: &addr) { pointer in
            pointer.withMemoryRebound(to: sockaddr.self, capacity: 1) { sockaddrPointer in
                bind(fd, sockaddrPointer, socklen_t(MemoryLayout<sockaddr_un>.size))
            }
        }
        guard bindResult == 0 else {
            let detail = String(cString: strerror(errno))
            close(fd)
            throw HerdrSocketError.connectFailed(detail)
        }
        listen(fd, 8)
        lock.lock()
        listenerFD = fd
        running = true
        lock.unlock()
        Thread.detachNewThread { [weak self] in
            self?.acceptLoop()
        }
    }

    func stop() {
        lock.lock()
        running = false
        let listener = listenerFD
        listenerFD = -1
        let fds = clientFDs
        clientFDs = []
        lock.unlock()
        if listener >= 0 {
            shutdown(listener, SHUT_RDWR)
            close(listener)
        }
        for fd in fds {
            shutdown(fd, SHUT_RDWR)
            close(fd)
        }
        unlink(path)
    }

    /// Pushes a `pane_agent_status_changed` event to every open subscription.
    func pushStatusChange(paneId: String, status: String) {
        let frame = #"{"event":"pane_agent_status_changed","data":{"type":"pane_agent_status_changed","pane_id":"\#(paneId)","workspace_id":"w1","agent_status":"\#(status)","agent":"fake","title":"fake agent title"}}"#
        lock.lock()
        let fds = subscriberFDs
        lock.unlock()
        for fd in fds {
            send(fd, frame)
        }
    }

    private func acceptLoop() {
        while true {
            lock.lock()
            let alive = running
            let listener = listenerFD
            lock.unlock()
            guard alive, listener >= 0 else { return }
            let fd = accept(listener, nil, nil)
            guard fd >= 0 else { return }
            lock.lock()
            clientFDs.append(fd)
            lock.unlock()
            Thread.detachNewThread { [weak self] in
                self?.serve(fd)
            }
        }
    }

    private func serve(_ fd: Int32) {
        var buffer = Data()
        while isRunning() {
            guard let line = readLine(fd: fd, buffer: &buffer) else {
                lock.lock()
                subscriberFDs.removeAll { $0 == fd }
                lock.unlock()
                return
            }
            guard let text = String(data: line, encoding: .utf8) else { return }
            let id = Self.requestID(text) ?? "req_fake"
            if text.contains("\"ping\"") {
                send(fd, #"{"id":"\#(id)","result":{"type":"pong","version":"0.0.1","protocol":20,"capabilities":{}}}"#)
            } else if text.contains("\"session.snapshot\"") {
                send(fd, Self.snapshotJSON(id: id))
            } else if text.contains("\"agent.list\"") {
                send(fd, Self.agentListJSON(id: id))
            } else if text.contains("\"events.subscribe\"") {
                send(fd, #"{"id":"\#(id)","result":{"type":"subscription_started"}}"#)
                lock.lock()
                subscriberFDs.append(fd)
                lock.unlock()
            } else {
                send(fd, #"{"id":"\#(id)","result":{}}"#)
            }
        }
    }

    private func isRunning() -> Bool {
        lock.lock()
        defer { lock.unlock() }
        return running
    }

    private func send(_ fd: Int32, _ json: String) {
        var data = Data(json.utf8)
        data.append(0x0A)
        data.withUnsafeBytes { (rawBuffer: UnsafeRawBufferPointer) in
            var written = 0
            while written < rawBuffer.count {
                let result = write(fd, rawBuffer.baseAddress!.advanced(by: written), rawBuffer.count - written)
                guard result > 0 else { return }
                written += result
            }
        }
    }

    private func readLine(fd: Int32, buffer: inout Data) -> Data? {
        while true {
            if let index = buffer.firstIndex(of: 0x0A) {
                let line = buffer.subdata(in: buffer.startIndex..<index)
                buffer.removeSubrange(buffer.startIndex...index)
                return line
            }
            var chunk = [UInt8](repeating: 0, count: 8192)
            let count = read(fd, &chunk, chunk.count)
            guard count > 0 else { return nil }
            buffer.append(contentsOf: chunk[0..<count])
        }
    }

    private static func requestID(_ text: String) -> String? {
        guard let data = text.data(using: .utf8),
              let object = try? JSONSerialization.jsonObject(with: data) as? [String: Any]
        else { return nil }
        return object["id"] as? String
    }

    private static let agentJSON = #"{"agent":"fake","agent_status":"working","pane_id":"w1:p1","workspace_id":"w1","tab_id":"w1:t1","terminal_id":"term_fake","terminal_title":"fake agent title","focused":true}"#

    private static func snapshotJSON(id: String) -> String {
        #"{"id":"\#(id)","result":{"type":"session_snapshot","snapshot":{"version":"0.0.1","protocol":20,"workspaces":[],"tabs":[],"panes":[],"agents":[\#(agentJSON)]}}}"#
    }

    private static func agentListJSON(id: String) -> String {
        #"{"id":"\#(id)","result":{"type":"agent_list","agents":[\#(agentJSON)]}}"#
    }
}
