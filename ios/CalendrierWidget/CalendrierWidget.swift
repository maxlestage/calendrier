import SwiftUI
import WidgetKit

// The widget has no App Group (kept out to keep automatic code-signing free
// of capabilities that need manual portal registration), so it reads the
// server URL that the app last mirrored into standard defaults, falling back
// to the deployed backend.
private let defaultServer = "https://calendrier-89594ce603e6.herokuapp.com"

// The widget target is self-contained on purpose: a minimal copy of the
// event model and fetch logic avoids sharing files across targets.
struct WidgetEvent: Decodable, Identifiable {
    let id: Int
    let title: String
    let start: String
    let end: String
    let allDay: Bool
    let color: String?

    enum CodingKeys: String, CodingKey {
        case id, title, start, end, color
        case allDay = "all_day"
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

    var swiftUIColor: Color {
        guard let color, color.hasPrefix("#"), color.count == 7,
              let value = UInt32(color.dropFirst(), radix: 16)
        else {
            return Color(red: 79 / 255.0, green: 107 / 255.0, blue: 237 / 255.0)
        }
        return Color(
            red: Double((value >> 16) & 0xFF) / 255.0,
            green: Double((value >> 8) & 0xFF) / 255.0,
            blue: Double(value & 0xFF) / 255.0
        )
    }
}

struct NextEventsEntry: TimelineEntry {
    let date: Date
    let events: [WidgetEvent]
}

struct NextEventsProvider: TimelineProvider {
    func placeholder(in context: Context) -> NextEventsEntry {
        NextEventsEntry(date: Date(), events: [])
    }

    func getSnapshot(in context: Context, completion: @escaping (NextEventsEntry) -> Void) {
        Task { completion(await load()) }
    }

    func getTimeline(in context: Context, completion: @escaping (Timeline<NextEventsEntry>) -> Void) {
        Task {
            let entry = await load()
            let refresh = Date().addingTimeInterval(30 * 60)
            completion(Timeline(entries: [entry], policy: .after(refresh)))
        }
    }

    private func load() async -> NextEventsEntry {
        let base = UserDefaults.standard.string(forKey: "serverURL") ?? defaultServer
        let trimmed = base.hasSuffix("/") ? String(base.dropLast()) : base
        let formatter = ISO8601DateFormatter()
        let now = Date()
        var events: [WidgetEvent] = []
        if var components = URLComponents(string: trimmed + "/api/events") {
            components.queryItems = [
                URLQueryItem(name: "from", value: formatter.string(from: now)),
                URLQueryItem(name: "to", value: formatter.string(from: now.addingTimeInterval(30 * 86400))),
            ]
            if let url = components.url,
               let (data, _) = try? await URLSession.shared.data(from: url) {
                events = (try? JSONDecoder().decode([WidgetEvent].self, from: data)) ?? []
            }
        }
        let upcoming = events
            .filter { $0.endDate >= now }
            .sorted { $0.start < $1.start }
        return NextEventsEntry(date: now, events: Array(upcoming.prefix(4)))
    }
}

struct NextEventsView: View {
    var entry: NextEventsEntry
    @Environment(\.widgetFamily) private var family

    private func when(_ event: WidgetEvent) -> String {
        let formatter = DateFormatter()
        formatter.locale = Locale(identifier: "fr_FR")
        formatter.dateFormat = event.allDay ? "EEE d MMM" : "EEE d MMM HH:mm"
        return formatter.string(from: event.startDate).capitalized
    }

    var body: some View {
        if entry.events.isEmpty {
            VStack(spacing: 4) {
                Image(systemName: "calendar")
                    .font(.title2)
                    .foregroundStyle(.secondary)
                Text("Aucun événement à venir")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
                    .multilineTextAlignment(.center)
            }
        } else if family == .systemSmall {
            let event = entry.events[0]
            VStack(alignment: .leading, spacing: 4) {
                Text("Prochain")
                    .font(.caption2.weight(.semibold))
                    .foregroundStyle(.secondary)
                RoundedRectangle(cornerRadius: 2)
                    .fill(event.swiftUIColor)
                    .frame(width: 24, height: 4)
                Text(event.title)
                    .font(.footnote.weight(.bold))
                    .lineLimit(3)
                Spacer(minLength: 0)
                Text(when(event))
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
        } else {
            VStack(alignment: .leading, spacing: 6) {
                ForEach(entry.events.prefix(3)) { event in
                    HStack(spacing: 8) {
                        RoundedRectangle(cornerRadius: 2)
                            .fill(event.swiftUIColor)
                            .frame(width: 4, height: 28)
                        VStack(alignment: .leading, spacing: 1) {
                            Text(event.title)
                                .font(.caption.weight(.semibold))
                                .lineLimit(1)
                            Text(when(event))
                                .font(.caption2)
                                .foregroundStyle(.secondary)
                        }
                        Spacer(minLength: 0)
                    }
                }
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        }
    }
}

struct CalendrierWidget: Widget {
    let kind: String = "CalendrierWidget"

    var body: some WidgetConfiguration {
        StaticConfiguration(kind: kind, provider: NextEventsProvider()) { entry in
            NextEventsView(entry: entry)
                .containerBackground(.fill.tertiary, for: .widget)
        }
        .configurationDisplayName("Prochains événements")
        .description("Les prochains événements de ton calendrier : F1, ciné, astronomie…")
        .supportedFamilies([.systemSmall, .systemMedium])
    }
}

@main
struct CalendrierWidgetBundle: WidgetBundle {
    var body: some Widget {
        CalendrierWidget()
    }
}
