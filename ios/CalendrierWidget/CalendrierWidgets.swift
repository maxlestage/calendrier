import SwiftUI
import WidgetKit

// MARK: - Config & helpers

/// Same default backend as the app. The widget runs in its own process and
/// (no App Group, on purpose) can't read the app's edited server URL, so it
/// uses the default Heroku deployment.
private let backendBase = "https://calendrier-89594ce603e6.herokuapp.com"
private let tideHex = "#0277bd"
private let accentHex = "#4f6bed"

private func isoDate(_ s: String) -> Date? {
    let f = ISO8601DateFormatter()
    f.formatOptions = [.withInternetDateTime]
    return f.date(from: s)
}

private func hm(_ d: Date) -> String {
    let f = DateFormatter()
    f.locale = Locale(identifier: "fr_FR")
    f.dateFormat = "HH:mm"
    return f.string(from: d)
}

private func wxEmoji(_ c: Int?) -> String {
    guard let c else { return "🌡️" }
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

// MARK: - Models & API

struct WEvent: Decodable {
    let title: String
    let start: String
    let end: String
    let all_day: Bool
    let color: String?
    var startDate: Date { isoDate(start) ?? .distantFuture }
    var isTide: Bool { color == tideHex }
    /// Title without a leading emoji, for compact display.
    var cleanTitle: String {
        title.replacingOccurrences(
            of: #"^[\x{1F000}-\x{1FAFF}\x{2600}-\x{27BF}\x{2648}-\x{2653}\x{2B00}-\x{2BFF}\x{FE00}-\x{FE0F}\x{1F1E6}-\x{1F1FF}\s]+"#,
            with: "", options: .regularExpression
        ).trimmingCharacters(in: .whitespaces)
    }
}

struct WDay: Decodable {
    let date: String
    let code: Int?
    let tmax: Double?
    let tmin: Double?
    let water: Double?
}

struct WSpot: Decodable {
    let name: String
    let group: String
    let days: [WDay]
}

private struct WWeatherResponse: Decodable { let spots: [WSpot] }

enum WAPI {
    static func events(from: Date, to: Date) async -> [WEvent] {
        let f = ISO8601DateFormatter()
        f.formatOptions = [.withInternetDateTime]
        guard var c = URLComponents(string: backendBase + "/api/events") else { return [] }
        c.queryItems = [
            URLQueryItem(name: "from", value: f.string(from: from)),
            URLQueryItem(name: "to", value: f.string(from: to)),
        ]
        guard let url = c.url,
              let (d, _) = try? await URLSession.shared.data(from: url),
              let evs = try? JSONDecoder().decode([WEvent].self, from: d)
        else { return [] }
        return evs
    }

    static func weather() async -> [WSpot] {
        guard let url = URL(string: backendBase + "/api/beach-weather"),
              let (d, _) = try? await URLSession.shared.data(from: url),
              let w = try? JSONDecoder().decode(WWeatherResponse.self, from: d)
        else { return [] }
        return w.spots
    }
}

/// Refresh policy shared by all widgets: about once an hour.
private func nextRefresh() -> Date {
    Calendar.current.date(byAdding: .minute, value: 60, to: Date()) ?? Date().addingTimeInterval(3600)
}

private func todayKey() -> String {
    let f = DateFormatter()
    f.dateFormat = "yyyy-MM-dd"
    return f.string(from: Date())
}

// MARK: - 🌊 Prochaines marées

struct TideItem: Hashable {
    let beach: String
    let time: String
    let high: Bool
}

struct TidesEntry: TimelineEntry {
    let date: Date
    let items: [TideItem]
}

struct TidesProvider: TimelineProvider {
    func placeholder(in context: Context) -> TidesEntry {
        TidesEntry(date: Date(), items: [TideItem(beach: "Anglet", time: "06:46", high: true)])
    }
    func getSnapshot(in context: Context, completion: @escaping (TidesEntry) -> Void) {
        Task { completion(await entry()) }
    }
    func getTimeline(in context: Context, completion: @escaping (Timeline<TidesEntry>) -> Void) {
        Task { completion(Timeline(entries: [await entry()], policy: .after(nextRefresh()))) }
    }
    private func entry() async -> TidesEntry {
        let now = Date()
        let evs = await WAPI.events(from: now, to: now.addingTimeInterval(2 * 86400))
        let items = evs
            .filter { $0.isTide && $0.startDate > now }
            .sorted { $0.startDate < $1.startDate }
            .prefix(4)
            .map { ev -> TideItem in
                let beach = ev.title.components(separatedBy: " — ").first?
                    .replacingOccurrences(of: "🌊", with: "").trimmingCharacters(in: .whitespaces) ?? ""
                return TideItem(beach: beach, time: hm(ev.startDate), high: ev.title.contains("Pleine mer"))
            }
        return TidesEntry(date: now, items: Array(items))
    }
}

struct TidesWidgetView: View {
    var entry: TidesEntry
    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            Label("Marées", systemImage: "water.waves").font(.caption).bold()
                .foregroundStyle(Color(hex: tideHex))
            if entry.items.isEmpty {
                Text("Aucune marée").font(.caption2).foregroundStyle(.secondary)
            } else {
                ForEach(entry.items.prefix(3), id: \.self) { it in
                    HStack(spacing: 4) {
                        Image(systemName: it.high ? "arrow.up" : "arrow.down")
                            .font(.caption2).foregroundStyle(Color(hex: tideHex))
                        Text(it.time).font(.callout).monospacedDigit().bold()
                        Text(it.high ? "pleine" : "basse").font(.caption2).foregroundStyle(.secondary)
                    }
                }
                if let beach = entry.items.first?.beach {
                    Text(beach).font(.caption2).foregroundStyle(.secondary).lineLimit(1)
                }
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .containerBackground(for: .widget) { Color(hex: "#e7f4fd") }
    }
}

struct TidesWidget: Widget {
    var body: some WidgetConfiguration {
        StaticConfiguration(kind: "CalendrierTides", provider: TidesProvider()) { entry in
            TidesWidgetView(entry: entry)
        }
        .configurationDisplayName("Marées")
        .description("Les prochaines pleines et basses mers de ta plage.")
        .supportedFamilies([.systemSmall, .systemMedium])
    }
}

// MARK: - ☀️ Météo du jour

struct WeatherEntry: TimelineEntry {
    let date: Date
    let name: String
    let emoji: String
    let tmax: Int?
    let tmin: Int?
    let water: Int?
    let hasData: Bool
}

struct WeatherProvider: TimelineProvider {
    func placeholder(in context: Context) -> WeatherEntry {
        WeatherEntry(date: Date(), name: "Anglet", emoji: "☀️", tmax: 27, tmin: 19, water: 24, hasData: true)
    }
    func getSnapshot(in context: Context, completion: @escaping (WeatherEntry) -> Void) {
        Task { completion(await entry()) }
    }
    func getTimeline(in context: Context, completion: @escaping (Timeline<WeatherEntry>) -> Void) {
        Task { completion(Timeline(entries: [await entry()], policy: .after(nextRefresh()))) }
    }
    private func entry() async -> WeatherEntry {
        let spots = await WAPI.weather()
        guard let spot = spots.first, let d = spot.days.first(where: { $0.date == todayKey() }) ?? spot.days.first else {
            return WeatherEntry(date: Date(), name: "", emoji: "🌡️", tmax: nil, tmin: nil, water: nil, hasData: false)
        }
        return WeatherEntry(
            date: Date(), name: spot.name, emoji: wxEmoji(d.code),
            tmax: d.tmax.map { Int($0.rounded()) },
            tmin: d.tmin.map { Int($0.rounded()) },
            water: d.water.map { Int($0.rounded()) },
            hasData: true
        )
    }
}

struct WeatherWidgetView: View {
    var entry: WeatherEntry
    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(entry.name.isEmpty ? "Météo" : entry.name).font(.caption).bold().lineLimit(1)
            Text(entry.emoji).font(.system(size: 40))
            if let mx = entry.tmax {
                HStack(spacing: 4) {
                    Text("\(mx)°").font(.title2).bold()
                    if let mn = entry.tmin { Text("/ \(mn)°").font(.callout).foregroundStyle(.secondary) }
                }
            }
            if let w = entry.water {
                Text("💧 eau \(w)°").font(.caption2).foregroundStyle(.secondary)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .containerBackground(for: .widget) {
            LinearGradient(colors: [Color(hex: "#eaf3ff"), Color(hex: "#f7fbff")],
                           startPoint: .top, endPoint: .bottom)
        }
    }
}

struct WeatherWidget: Widget {
    var body: some WidgetConfiguration {
        StaticConfiguration(kind: "CalendrierWeather", provider: WeatherProvider()) { entry in
            WeatherWidgetView(entry: entry)
        }
        .configurationDisplayName("Météo du jour")
        .description("La météo de ta plage ou ville.")
        .supportedFamilies([.systemSmall, .systemMedium])
    }
}

// MARK: - 📅 Agenda du jour

struct AgendaItem: Hashable {
    let time: String
    let title: String
}

struct AgendaEntry: TimelineEntry {
    let date: Date
    let items: [AgendaItem]
}

struct AgendaProvider: TimelineProvider {
    func placeholder(in context: Context) -> AgendaEntry {
        AgendaEntry(date: Date(), items: [AgendaItem(time: "09:00", title: "Sport")])
    }
    func getSnapshot(in context: Context, completion: @escaping (AgendaEntry) -> Void) {
        Task { completion(await entry()) }
    }
    func getTimeline(in context: Context, completion: @escaping (Timeline<AgendaEntry>) -> Void) {
        Task { completion(Timeline(entries: [await entry()], policy: .after(nextRefresh()))) }
    }
    private func entry() async -> AgendaEntry {
        let cal = Calendar.current
        let start = cal.startOfDay(for: Date())
        let end = cal.date(byAdding: .day, value: 1, to: start) ?? start
        let evs = await WAPI.events(from: start, to: end)
        let items = evs
            .filter { !$0.isTide }
            .sorted { $0.start < $1.start }
            .prefix(4)
            .map { AgendaItem(time: $0.all_day ? "Jour" : hm($0.startDate), title: $0.cleanTitle) }
        return AgendaEntry(date: Date(), items: Array(items))
    }
}

struct AgendaWidgetView: View {
    var entry: AgendaEntry
    var body: some View {
        VStack(alignment: .leading, spacing: 5) {
            Label("Aujourd'hui", systemImage: "calendar").font(.caption).bold()
                .foregroundStyle(Color(hex: accentHex))
            if entry.items.isEmpty {
                Text("Rien de prévu").font(.caption2).foregroundStyle(.secondary)
            } else {
                ForEach(entry.items.prefix(4), id: \.self) { it in
                    HStack(spacing: 6) {
                        Text(it.time).font(.caption2).monospacedDigit()
                            .foregroundStyle(.secondary).frame(width: 34, alignment: .leading)
                        Text(it.title).font(.caption).lineLimit(1)
                    }
                }
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .containerBackground(for: .widget) { Color(.systemBackground) }
    }
}

struct AgendaWidget: Widget {
    var body: some WidgetConfiguration {
        StaticConfiguration(kind: "CalendrierAgenda", provider: AgendaProvider()) { entry in
            AgendaWidgetView(entry: entry)
        }
        .configurationDisplayName("Agenda du jour")
        .description("Les événements de ta journée.")
        .supportedFamilies([.systemSmall, .systemMedium])
    }
}

// MARK: - ⏭️ Prochain événement

struct NextEventEntry: TimelineEntry {
    let date: Date
    let title: String?
    let when: String?
}

struct NextEventProvider: TimelineProvider {
    func placeholder(in context: Context) -> NextEventEntry {
        NextEventEntry(date: Date(), title: "Sport", when: "Aujourd'hui 09:00")
    }
    func getSnapshot(in context: Context, completion: @escaping (NextEventEntry) -> Void) {
        Task { completion(await entry()) }
    }
    func getTimeline(in context: Context, completion: @escaping (Timeline<NextEventEntry>) -> Void) {
        Task { completion(Timeline(entries: [await entry()], policy: .after(nextRefresh()))) }
    }
    private func entry() async -> NextEventEntry {
        let now = Date()
        let evs = await WAPI.events(from: now, to: now.addingTimeInterval(30 * 86400))
        guard let ev = evs
            .filter({ !$0.isTide && !$0.all_day && $0.startDate > now })
            .min(by: { $0.startDate < $1.startDate })
        else {
            return NextEventEntry(date: now, title: nil, when: nil)
        }
        let f = DateFormatter()
        f.locale = Locale(identifier: "fr_FR")
        f.dateFormat = Calendar.current.isDateInToday(ev.startDate) ? "'Aujourd''hui' HH:mm" : "EEE d MMM · HH:mm"
        return NextEventEntry(date: now, title: ev.cleanTitle, when: f.string(from: ev.startDate))
    }
}

struct NextEventWidgetView: View {
    var entry: NextEventEntry
    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            Label("Prochain", systemImage: "arrowshape.forward.fill").font(.caption).bold()
                .foregroundStyle(Color(hex: accentHex))
            Spacer(minLength: 0)
            if let title = entry.title {
                Text(title).font(.headline).lineLimit(2)
                if let when = entry.when {
                    Text(when).font(.caption).foregroundStyle(.secondary)
                }
            } else {
                Text("Aucun événement à venir").font(.caption).foregroundStyle(.secondary)
            }
            Spacer(minLength: 0)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .containerBackground(for: .widget) { Color(.systemBackground) }
    }
}

struct NextEventWidget: Widget {
    var body: some WidgetConfiguration {
        StaticConfiguration(kind: "CalendrierNextEvent", provider: NextEventProvider()) { entry in
            NextEventWidgetView(entry: entry)
        }
        .configurationDisplayName("Prochain événement")
        .description("Ton prochain rendez-vous.")
        .supportedFamilies([.systemSmall, .systemMedium])
    }
}

// MARK: - Bundle

@main
struct CalendrierWidgets: WidgetBundle {
    var body: some Widget {
        TidesWidget()
        WeatherWidget()
        AgendaWidget()
        NextEventWidget()
    }
}
