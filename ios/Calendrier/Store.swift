import Foundation
import SwiftUI

/// App-wide state: the visible month's events, beach weather, selection, and
/// notification preferences. Talks to the Rust backend through `API`.
@MainActor
final class CalendarStore: ObservableObject {
    @Published var year: Int
    @Published var month: Int          // 1–12
    @Published var selectedDay: Date
    @Published var events: [CalendarEvent] = []
    @Published var weather: [BeachWeather] = []
    @Published var prefs: NotifPrefs = .fallback
    @Published var errorMessage: String?
    @Published var loading = false

    init() {
        let now = Date()
        year = appCalendar.component(.year, from: now)
        month = appCalendar.component(.month, from: now)
        selectedDay = now
    }

    /// [from, to] covering the visible 6-week grid.
    private var gridBounds: (Date, Date) {
        let days = monthGridDays(year: year, month: month)
        let from = days.first ?? Date()
        let to = appCalendar.date(byAdding: .day, value: 1, to: days.last ?? Date()) ?? Date()
        return (from, to)
    }

    var eventsForSelectedDay: [CalendarEvent] {
        events.filter { eventCoversDay($0, selectedDay) }
            .sorted { $0.start < $1.start }
    }

    func shiftMonth(_ delta: Int) {
        var comps = DateComponents(); comps.year = year; comps.month = month + delta; comps.day = 1
        if let d = appCalendar.date(from: comps) {
            year = appCalendar.component(.year, from: d)
            month = appCalendar.component(.month, from: d)
        }
        Task { await load() }
    }

    func goToday() {
        let now = Date()
        year = appCalendar.component(.year, from: now)
        month = appCalendar.component(.month, from: now)
        selectedDay = now
        Task { await load() }
    }

    func select(_ day: Date) {
        selectedDay = day
        let m = appCalendar.component(.month, from: day)
        let y = appCalendar.component(.year, from: day)
        if m != month || y != year { year = y; month = m; Task { await load() } }
    }

    func load() async {
        loading = true
        defer { loading = false }
        let (from, to) = gridBounds
        do {
            events = try await API.events(from: from, to: to)
            errorMessage = nil
        } catch {
            // Offline / server down: fall back to the on-device copy so the
            // app still shows the calendar. The views filter by day, so the
            // whole cached set is safe to hand over.
            if let cached = DeviceBackup.load()?.events {
                events = cached
                errorMessage = "Hors ligne — données locales"
            } else {
                errorMessage = error.localizedDescription
            }
        }
    }

    func loadWeather() async {
        weather = (try? await API.beachWeather()) ?? []
    }

    func loadPrefs() async {
        if let p = try? await API.prefs() { prefs = p }
    }

    /// Reschedule local notifications from a wide (14-day) fetch + weather.
    func syncNotifications() async {
        let now = Date()
        let to = now.addingTimeInterval(14 * 86400)
        guard let wide = try? await API.events(from: now, to: to) else { return }
        await Notifications.reschedule(events: wide, weather: weather, prefs: prefs)
    }

    func save(_ payload: EventPayload, editing id: Int?) async throws {
        if let id { _ = try await API.update(id, payload) } else { _ = try await API.create(payload) }
        await load()
        await syncNotifications()
        await backupLocally()
    }

    func delete(_ id: Int) async throws {
        try await API.delete(id)
        await load()
        await syncNotifications()
        await backupLocally()
    }

    /// Full refresh used after settings changes.
    func refreshAll() async {
        await load()
        await loadWeather()
        await loadPrefs()
        await syncNotifications()
    }

    // MARK: - Persistence across dyno resets

    /// Boot sequence: restore the server from the device if it was wiped,
    /// then load, then refresh the local copy.
    func launch() async {
        await restoreIfServerReset()
        await refreshAll()
        await backupLocally()
    }

    /// Returning to the foreground: re-check for a server reset (the dyno may
    /// have been wiped while the app was backgrounded), reload, and refresh
    /// the local copy — so simply reopening the app restores everything.
    func onForeground() async {
        await restoreIfServerReset()
        await load()
        await loadWeather()
        await backupLocally()
    }

    /// Compare markers: if the server's is gone/changed, it lost its
    /// database — push the device's copy back. Ensure a marker exists.
    private func restoreIfServerReset() async {
        guard let server = try? await API.state() else { return }
        let local = DeviceBackup.load()
        let localMarker = local.flatMap { DeviceBackup.marker(in: $0) }
        let serverMarker = DeviceBackup.marker(in: server)
        if let local, let localMarker, serverMarker != localMarker {
            _ = try? await API.importState(local)
        }
        if serverMarker == nil {
            let seed = ServerState(
                events: [],
                settings: [SettingKV(key: DeviceBackup.markerKey, value: DeviceBackup.newMarker())]
            )
            _ = try? await API.importState(seed)
        }
    }

    /// Refresh the on-device backup — but only from a marked server state, so
    /// a freshly reset (marker-less) server never overwrites a good copy.
    func backupLocally() async {
        if let server = try? await API.state(), DeviceBackup.marker(in: server) != nil {
            DeviceBackup.save(server)
        }
    }
}
