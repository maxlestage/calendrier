import Foundation

/// On-device safety net: the phone keeps a full copy of the server state
/// (events + settings) on disk and pushes it back when the server (an
/// ephemeral Heroku dyno) has been wiped. Works with **no config** — no
/// Heroku API key, no external service — and survives even a brutal crash.
///
/// Loss is detected with a `backup_marker` setting: written server-side on
/// first launch and mirrored locally. If the server's marker no longer
/// matches the local one, the database was reset and we restore it.
enum DeviceBackup {
    static let markerKey = "backup_marker"

    private static var fileURL: URL {
        let dir = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
        try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        return dir.appendingPathComponent("calendrier-backup.json")
    }

    static func save(_ state: ServerState) {
        guard let data = try? JSONEncoder().encode(state) else { return }
        try? data.write(to: fileURL, options: .atomic)
    }

    static func load() -> ServerState? {
        guard let data = try? Data(contentsOf: fileURL) else { return nil }
        return try? JSONDecoder().decode(ServerState.self, from: data)
    }

    static func marker(in state: ServerState) -> String? {
        state.settings.first { $0.key == markerKey }?.value
    }

    static func newMarker() -> String { UUID().uuidString }
}
