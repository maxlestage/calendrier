import Foundation
import SwiftUI

// MARK: - ISO 8601 (UTC) parsing / formatting

private let isoFormatter: ISO8601DateFormatter = {
    let f = ISO8601DateFormatter()
    f.formatOptions = [.withInternetDateTime]
    return f
}()

extension String {
    /// Parse a backend "YYYY-MM-DDTHH:MM:SSZ" instant.
    var isoDate: Date? { isoFormatter.date(from: self) }
}

extension Date {
    /// Serialise to the backend's ISO UTC form.
    var isoString: String { isoFormatter.string(from: self) }
}

// MARK: - Local (device) calendar helpers

/// The device calendar (Europe/Paris for the user) — all grid/day logic is local.
let appCalendar = Calendar.current

extension Date {
    var localDayStart: Date { appCalendar.startOfDay(for: self) }

    func sameDay(as other: Date) -> Bool {
        appCalendar.isDate(self, inSameDayAs: other)
    }

    /// "HH:mm" in the device time zone.
    var clock: String {
        let f = DateFormatter()
        f.locale = Locale(identifier: "fr_FR")
        f.dateFormat = "HH:mm"
        return f.string(from: self)
    }
}

/// Whether an event [start, end] overlaps the given local day.
func eventCoversDay(_ ev: CalendarEvent, _ day: Date) -> Bool {
    let dayStart = day.localDayStart
    let dayEnd = appCalendar.date(byAdding: .day, value: 1, to: dayStart)!.addingTimeInterval(-1)
    return ev.startDate <= dayEnd && ev.endDate >= dayStart
}

/// 42 days (6 weeks) of the month grid, Monday-first, like the web.
func monthGridDays(year: Int, month: Int) -> [Date] {
    var comps = DateComponents()
    comps.year = year
    comps.month = month
    comps.day = 1
    guard let first = appCalendar.date(from: comps) else { return [] }
    // weekday: 1 = Sunday … 7 = Saturday → Monday-first offset
    let weekday = appCalendar.component(.weekday, from: first)
    let offset = (weekday + 5) % 7
    guard let start = appCalendar.date(byAdding: .day, value: -offset, to: first) else { return [] }
    return (0..<42).compactMap { appCalendar.date(byAdding: .day, value: $0, to: start) }
}

let frMonthNames = ["Janvier", "Février", "Mars", "Avril", "Mai", "Juin",
                    "Juillet", "Août", "Septembre", "Octobre", "Novembre", "Décembre"]
let frWeekdayShort = ["Lun", "Mar", "Mer", "Jeu", "Ven", "Sam", "Dim"]
let frWeekdayFull = ["Dimanche", "Lundi", "Mardi", "Mercredi", "Jeudi", "Vendredi", "Samedi"]

// MARK: - Colour from hex

extension Color {
    init(hex: String) {
        let s = hex.trimmingCharacters(in: CharacterSet(charactersIn: "#"))
        var v: UInt64 = 0
        Scanner(string: s).scanHexInt64(&v)
        self.init(
            red: Double((v >> 16) & 0xFF) / 255,
            green: Double((v >> 8) & 0xFF) / 255,
            blue: Double(v & 0xFF) / 255
        )
    }
}

// MARK: - Weather (WMO code → emoji)

func weatherEmoji(_ code: Int?) -> String {
    guard let c = code else { return "🌡️" }
    switch c {
    case 0: return "☀️"
    case 1: return "🌤️"
    case 2: return "⛅"
    case 3: return "☁️"
    case 45, 48: return "🌫️"
    case 51...57: return "🌦️"
    case 61...67: return "🌧️"
    case 71...77: return "🌨️"
    case 80...82: return "🌦️"
    case 85, 86: return "🌨️"
    case 95...99: return "⛈️"
    default: return "🌡️"
    }
}
