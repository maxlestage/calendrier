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
            errorMessage = error.localizedDescription
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
    }

    func delete(_ id: Int) async throws {
        try await API.delete(id)
        await load()
        await syncNotifications()
    }

    /// Full refresh used at launch and after settings changes.
    func refreshAll() async {
        await load()
        await loadWeather()
        await loadPrefs()
        await syncNotifications()
    }
}
