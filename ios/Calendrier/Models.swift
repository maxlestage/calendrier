import Foundation

struct CalendarEvent: Codable, Identifiable, Hashable {
    var id: Int
    var title: String
    var description: String?
    var start: String
    var end: String
    var allDay: Bool
    var color: String?

    enum CodingKeys: String, CodingKey {
        case id, title, description, start, end
        case allDay = "all_day"
        case color
    }

    var startDate: Date { Self.parseISO(start) }
    var endDate: Date { Self.parseISO(end) }

    static func parseISO(_ iso: String) -> Date {
        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        if let date = formatter.date(from: iso) { return date }
        formatter.formatOptions = [.withInternetDateTime]
        return formatter.date(from: iso) ?? .distantPast
    }
}

struct EventPayload: Codable {
    var title: String
    var description: String?
    var start: String
    var end: String
    var allDay: Bool
    var color: String?

    enum CodingKeys: String, CodingKey {
        case title, description, start, end
        case allDay = "all_day"
        case color
    }
}

enum EventColors {
    static let palette: [String] = [
        "#4f6bed", "#0f9d58", "#d93025", "#f4a300", "#8e44ad", "#0aa3a3",
    ]
}
