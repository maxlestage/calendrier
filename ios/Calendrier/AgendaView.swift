import SwiftUI

struct AgendaView: View {
    @EnvironmentObject var store: CalendarStore
    @ObservedObject var speaker = Speaker.shared
    var voiceEnabled: Bool
    var onEventTap: (CalendarEvent) -> Void
    var onAdd: () -> Void

    private var dayKey: String {
        let f = DateFormatter(); f.dateFormat = "yyyy-MM-dd"; return f.string(from: store.selectedDay)
    }

    var body: some View {
        let day = store.selectedDay
        let weekday = frWeekdayFull[appCalendar.component(.weekday, from: day) - 1]
        let dayNum = appCalendar.component(.day, from: day)
        let monthName = frMonthNames[appCalendar.component(.month, from: day) - 1].lowercased()

        ScrollView {
            VStack(alignment: .leading, spacing: 8) {
                HStack {
                    Text("\(weekday) \(dayNum) \(monthName)")
                        .font(.subheadline).fontWeight(.bold)
                        .foregroundStyle(.secondary)
                    Spacer()
                    if voiceEnabled {
                        Button {
                            speaker.toggle(buildDaySpeech(
                                day: store.selectedDay,
                                dayEvents: store.eventsForSelectedDay,
                                weather: store.weather
                            ))
                        } label: {
                            Image(systemName: speaker.speaking ? "stop.circle.fill" : "speaker.wave.2.fill")
                                .font(.title3)
                        }
                        .accessibilityLabel(speaker.speaking ? "Arrêter la lecture" : "Écouter la journée")
                    }
                }
                .padding(.horizontal, 4)

                ForEach(weatherCards, id: \.spot.id) { card in
                    WeatherCardView(spot: card.spot, day: card.day)
                }

                let evs = store.eventsForSelectedDay
                if evs.isEmpty {
                    Button(action: onAdd) {
                        Text("Aucun événement — appuyer pour en ajouter un")
                            .font(.callout).foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity).padding()
                            .background(RoundedRectangle(cornerRadius: 12).strokeBorder(.quaternary, style: StrokeStyle(lineWidth: 1, dash: [4])))
                    }
                    .buttonStyle(.plain)
                } else {
                    ForEach(evs) { ev in
                        Button { onEventTap(ev) } label: { EventRow(ev: ev) }
                            .buttonStyle(.plain)
                    }
                }
            }
            .padding(.horizontal, 4)
            .padding(.bottom, 90)
        }
    }

    private var weatherCards: [(spot: BeachWeather, day: BeachWeatherDay)] {
        store.weather.compactMap { spot in
            guard let d = spot.days.first(where: { $0.date == dayKey }) else { return nil }
            return (spot, d)
        }
    }
}

private struct EventRow: View {
    let ev: CalendarEvent
    var body: some View {
        HStack(spacing: 10) {
            RoundedRectangle(cornerRadius: 2)
                .fill(Color(hex: ev.color ?? "#4f6bed")).frame(width: 4)
            VStack(alignment: .leading, spacing: 2) {
                (Text(ev.title) + Text(ev.recurrence != nil ? " 🔁" : "").foregroundColor(.secondary))
                    .font(.callout).fontWeight(.semibold)
                if let d = ev.description, !d.isEmpty {
                    Text(d).font(.footnote).foregroundStyle(.secondary).lineLimit(1)
                }
            }
            Spacer()
            Text(ev.allDay ? "Journée" : "\(ev.startDate.clock)\n\(ev.endDate.clock)")
                .font(.caption).foregroundStyle(.secondary)
                .multilineTextAlignment(.trailing)
        }
        .padding(10)
        .frame(maxWidth: .infinity)
        .background(RoundedRectangle(cornerRadius: 12).fill(Color(.secondarySystemBackground)))
    }
}

private struct WeatherCardView: View {
    let spot: BeachWeather
    let day: BeachWeatherDay

    private var details: String {
        var parts: [String] = []
        if let w = day.wind { parts.append("💨 \(Int(w.rounded())) km/h") }
        if let uv = day.uv { parts.append("UV \(String(format: "%.0f", uv))") }
        if let p = day.precip { parts.append("☔ \(Int(p.rounded())) %") }
        if let wv = day.wave { parts.append("🌊 \(String(format: "%.1f", wv)) m") }
        if let wt = day.water { parts.append("💧 \(String(format: "%.0f", wt))°") }
        if let sr = day.sunrise, let ss = day.sunset { parts.append("🌅 \(sr) · 🌇 \(ss)") }
        if let pol = day.pollen, pol >= 20 { parts.append(pol >= 80 ? "🤧 pollen fort" : "🤧 pollen modéré") }
        return parts.joined(separator: " · ")
    }

    var body: some View {
        HStack(spacing: 12) {
            Text(weatherEmoji(day.code)).font(.system(size: 30))
            VStack(alignment: .leading, spacing: 3) {
                HStack {
                    Text("\(spot.group == "ville" ? "🏙️" : "🏖️") \(spot.name)")
                        .font(.callout).fontWeight(.bold).foregroundStyle(Color(hex: "#0b4f8a"))
                    Spacer()
                    if let mx = day.tmax {
                        (Text("\(Int(mx.rounded()))°").fontWeight(.bold)
                         + Text(day.tmin != nil ? " / \(Int(day.tmin!.rounded()))°" : "").foregroundColor(.secondary))
                            .font(.callout)
                    }
                }
                if !details.isEmpty {
                    Text(details).font(.caption).foregroundStyle(Color(hex: "#33566f"))
                }
            }
        }
        .padding(10)
        .frame(maxWidth: .infinity)
        .background(RoundedRectangle(cornerRadius: 12).fill(Color(hex: "#e7f4fd")))
    }
}
