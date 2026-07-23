import Foundation

/// Sea blue the backend uses for tide events.
let tideColorHex = "#0277bd"
/// Green the backend uses for holidays / school vacations.
let holidayColorHex = "#2e7d32"

/// User-palette colours offered in the event editor.
let eventColors = ["#4f6bed", "#0f9d58", "#d93025", "#f4a300", "#8e44ad", "#0aa3a3"]

struct CalendarEvent: Codable, Identifiable, Hashable {
    let id: Int
    var title: String
    var description: String?
    /// ISO 8601 UTC ("2026-07-22T15:03:55Z")
    var start: String
    var end: String
    var allDay: Bool
    var color: String?
    var recurrence: String?

    enum CodingKeys: String, CodingKey {
        case id, title, description, start, end
        case allDay = "all_day"
        case color, recurrence
    }

    var startDate: Date { start.isoDate ?? .distantPast }
    var endDate: Date { end.isoDate ?? startDate }
    var isTide: Bool { color == tideColorHex }
}

/// Body for POST/PUT (no id).
struct EventPayload: Codable {
    var title: String
    var description: String?
    var start: String
    var end: String
    var allDay: Bool
    var color: String?
    var recurrence: String?

    enum CodingKeys: String, CodingKey {
        case title, description, start, end
        case allDay = "all_day"
        case color, recurrence
    }
}

struct TideSpot: Codable, Identifiable, Hashable {
    let key: String
    let name: String
    let group: String
    let selected: Bool
    var id: String { key }
}

struct WeatherCity: Codable, Identifiable, Hashable {
    let key: String
    let name: String
    let selected: Bool
    var id: String { key }
}

struct BeachWeatherDay: Codable, Hashable {
    let date: String
    let code: Int?
    let tmax: Double?
    let tmin: Double?
    let wind: Double?
    let uv: Double?
    let precip: Double?
    let wave: Double?
    let water: Double?
    let sunrise: String?
    let sunset: String?
    let pollen: Double?
}

struct BeachWeather: Codable, Identifiable, Hashable {
    let key: String
    let name: String
    let group: String
    let days: [BeachWeatherDay]
    var id: String { key }
}

struct BeachWeatherResponse: Codable { let spots: [BeachWeather] }

/// A backend setting (key/value) as returned by /api/state.
struct SettingKV: Codable, Hashable {
    let key: String
    let value: String
}

/// The unit the device stores locally and can push back after a dyno reset.
struct ServerState: Codable {
    var events: [CalendarEvent]
    var settings: [SettingKV]
}

struct NotifPrefs: Codable, Equatable {
    var morningHour: Int
    var leadMin: Int
    var morningBriefing: Bool
    var eventReminders: Bool

    enum CodingKeys: String, CodingKey {
        case morningHour = "morning_hour"
        case leadMin = "lead_min"
        case morningBriefing = "morning_briefing"
        case eventReminders = "event_reminders"
    }

    static let fallback = NotifPrefs(morningHour: 7, leadMin: 15, morningBriefing: true, eventReminders: true)
}
