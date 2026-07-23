import SwiftUI

struct MonthView: View {
    @EnvironmentObject var store: CalendarStore

    private var weatherByDate: [String: String] {
        var map: [String: String] = [:]
        for d in store.weather.first?.days ?? [] where d.code != nil {
            map[d.date] = weatherEmoji(d.code)
        }
        return map
    }

    private let dayKeyFmt: DateFormatter = {
        let f = DateFormatter(); f.dateFormat = "yyyy-MM-dd"; return f
    }()

    var body: some View {
        VStack(spacing: 0) {
            HStack(spacing: 0) {
                ForEach(frWeekdayShort, id: \.self) { d in
                    Text(d.uppercased())
                        .font(.caption2).fontWeight(.semibold)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity)
                }
            }
            .padding(.vertical, 6)

            let cols = Array(repeating: GridItem(.flexible(), spacing: 0), count: 7)
            LazyVGrid(columns: cols, spacing: 0) {
                ForEach(monthGridDays(year: store.year, month: store.month), id: \.self) { day in
                    cell(day)
                        .contentShape(Rectangle())
                        .onTapGesture { store.select(day) }
                }
            }
        }
        .background(Color(.secondarySystemBackground))
        .clipShape(RoundedRectangle(cornerRadius: 16))
    }

    @ViewBuilder
    private func cell(_ day: Date) -> some View {
        let inMonth = appCalendar.component(.month, from: day) == store.month
        let isToday = day.sameDay(as: Date())
        let isSelected = day.sameDay(as: store.selectedDay)
        let dayEvents = store.events.filter { eventCoversDay($0, day) }
        let tideHighs = Array(
            Set(dayEvents.filter { $0.isTide && $0.title.contains("Pleine mer") }.map { $0.startDate.clock })
        ).sorted().prefix(2)
        let dots = dayEvents.filter { !$0.isTide }.prefix(4)

        VStack(spacing: 3) {
            ZStack {
                if isToday { Circle().fill(Color.accentColor).frame(width: 30, height: 30) }
                Text("\(appCalendar.component(.day, from: day))")
                    .font(.callout).fontWeight(.semibold)
                    .foregroundStyle(isToday ? .white : (inMonth ? .primary : .secondary))
                    .frame(width: 30, height: 30)
                    .overlay {
                        if isSelected && !isToday {
                            Circle().strokeBorder(Color.accentColor, lineWidth: 2)
                        }
                    }
            }
            if let emoji = weatherByDate[dayKeyFmt.string(from: day)] {
                Text(emoji).font(.system(size: 11))
            }
            HStack(spacing: 3) {
                ForEach(Array(dots.enumerated()), id: \.offset) { _, ev in
                    Circle().fill(Color(hex: ev.color ?? "#4f6bed")).frame(width: 6, height: 6)
                }
            }
            .frame(height: 6)
            if !tideHighs.isEmpty {
                VStack(spacing: 0) {
                    ForEach(Array(tideHighs), id: \.self) { t in
                        Text("▲\(t)").font(.system(size: 9, weight: .bold))
                            .foregroundStyle(Color(hex: tideColorHex))
                    }
                }
            }
        }
        .frame(maxWidth: .infinity, minHeight: 58)
        .padding(.vertical, 4)
        .opacity(inMonth ? 1 : 0.5)
    }
}
