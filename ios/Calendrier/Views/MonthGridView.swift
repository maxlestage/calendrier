import SwiftUI

struct MonthGridView: View {
    @ObservedObject var viewModel: CalendarViewModel

    private let weekdays = ["Lun", "Mar", "Mer", "Jeu", "Ven", "Sam", "Dim"]
    private let columns = Array(repeating: GridItem(.flexible(), spacing: 0), count: 7)

    var body: some View {
        VStack(spacing: 4) {
            HStack {
                ForEach(weekdays, id: \.self) { day in
                    Text(day.uppercased())
                        .font(.caption2.weight(.semibold))
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity)
                }
            }
            LazyVGrid(columns: columns, spacing: 2) {
                ForEach(viewModel.gridDays, id: \.self) { day in
                    DayCell(viewModel: viewModel, day: day)
                }
            }
        }
        .padding(8)
        .background(Color(.secondarySystemGroupedBackground))
        .clipShape(RoundedRectangle(cornerRadius: 16))
    }
}

private struct DayCell: View {
    @ObservedObject var viewModel: CalendarViewModel
    let day: Date

    var body: some View {
        let cal = viewModel.calendar
        let inMonth = viewModel.isInDisplayedMonth(day)
        let isToday = cal.isDateInToday(day)
        let isSelected = cal.isDate(day, inSameDayAs: viewModel.selectedDay)
        let dayEvents = viewModel.eventsOn(day)

        Button {
            viewModel.select(day)
        } label: {
            VStack(spacing: 3) {
                Text("\(cal.component(.day, from: day))")
                    .font(.callout.weight(.semibold))
                    .foregroundStyle(isToday ? Color.white : (inMonth ? Color.primary : Color.secondary))
                    .frame(width: 30, height: 30)
                    .background(isToday ? Color.accentColor : Color.clear)
                    .clipShape(Circle())
                    .overlay(
                        Circle().strokeBorder(
                            isSelected ? Color.accentColor : Color.clear,
                            lineWidth: 2
                        )
                    )
                HStack(spacing: 3) {
                    ForEach(dayEvents.prefix(4), id: \.id) { event in
                        Circle()
                            .fill(Color(hex: event.color))
                            .frame(width: 5, height: 5)
                    }
                }
                .frame(height: 5)
            }
            .frame(maxWidth: .infinity, minHeight: 46)
            .opacity(inMonth ? 1 : 0.45)
        }
        .buttonStyle(.plain)
    }
}
