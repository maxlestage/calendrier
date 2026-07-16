import Foundation
import SwiftUI

@MainActor
final class CalendarViewModel: ObservableObject {
    /// First day of the displayed month
    @Published var monthAnchor: Date
    @Published var selectedDay: Date
    @Published var events: [CalendarEvent] = []
    @Published var errorMessage: String?

    @AppStorage("serverURL") var serverURL: String = "https://calendrier-89594ce603e6.herokuapp.com"

    var calendar: Calendar {
        var cal = Calendar(identifier: .gregorian)
        cal.locale = Locale(identifier: "fr_FR")
        cal.firstWeekday = 2 // lundi
        return cal
    }

    private var api: APIClient { APIClient(baseURL: serverURL) }

    init() {
        let now = Date()
        let cal = Calendar(identifier: .gregorian)
        monthAnchor = cal.date(from: cal.dateComponents([.year, .month], from: now)) ?? now
        selectedDay = cal.startOfDay(for: now)
    }

    /// The 42 days (6 weeks, Monday-first) of the month grid.
    var gridDays: [Date] {
        let cal = calendar
        let weekday = cal.component(.weekday, from: monthAnchor)
        let offset = (weekday - cal.firstWeekday + 7) % 7
        guard let start = cal.date(byAdding: .day, value: -offset, to: monthAnchor) else {
            return []
        }
        return (0..<42).compactMap { cal.date(byAdding: .day, value: $0, to: start) }
    }

    var monthTitle: String {
        let formatter = DateFormatter()
        formatter.locale = Locale(identifier: "fr_FR")
        formatter.dateFormat = "LLLL yyyy"
        return formatter.string(from: monthAnchor).capitalized
    }

    func isInDisplayedMonth(_ day: Date) -> Bool {
        calendar.isDate(day, equalTo: monthAnchor, toGranularity: .month)
    }

    func eventsOn(_ day: Date) -> [CalendarEvent] {
        let cal = calendar
        let dayStart = cal.startOfDay(for: day)
        guard let dayEnd = cal.date(byAdding: .day, value: 1, to: dayStart) else { return [] }
        return events
            .filter { $0.startDate < dayEnd && $0.endDate >= dayStart }
            .sorted { $0.start < $1.start }
    }

    func shiftMonth(_ delta: Int) {
        if let next = calendar.date(byAdding: .month, value: delta, to: monthAnchor) {
            monthAnchor = next
            Task { await reload() }
        }
    }

    func goToday() {
        let cal = calendar
        let now = Date()
        monthAnchor = cal.date(from: cal.dateComponents([.year, .month], from: now)) ?? now
        selectedDay = cal.startOfDay(for: now)
        Task { await reload() }
    }

    func select(_ day: Date) {
        selectedDay = day
        if !isInDisplayedMonth(day) {
            let cal = calendar
            monthAnchor = cal.date(from: cal.dateComponents([.year, .month], from: day)) ?? day
            Task { await reload() }
        }
    }

    func reload() async {
        guard let first = gridDays.first, let last = gridDays.last,
              let afterLast = calendar.date(byAdding: .day, value: 1, to: last)
        else { return }
        let formatter = ISO8601DateFormatter()
        do {
            events = try await api.events(
                from: formatter.string(from: first),
                to: formatter.string(from: afterLast)
            )
            errorMessage = nil
        } catch {
            errorMessage = error.localizedDescription
        }
    }

    func save(event existing: CalendarEvent?, payload: EventPayload) async throws {
        if let existing {
            _ = try await api.update(id: existing.id, payload)
        } else {
            _ = try await api.create(payload)
        }
        await reload()
    }

    func delete(event: CalendarEvent) async throws {
        try await api.delete(id: event.id)
        await reload()
    }
}
