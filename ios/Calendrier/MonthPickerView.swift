import SwiftUI

/// Sheet to jump to any month and year (opened by tapping the month title).
struct MonthPickerView: View {
    @EnvironmentObject var store: CalendarStore
    @Environment(\.dismiss) private var dismiss

    @State private var month = 1
    @State private var year = 2026

    var body: some View {
        NavigationStack {
            HStack(spacing: 0) {
                Picker("Mois", selection: $month) {
                    ForEach(1...12, id: \.self) { m in
                        Text(frMonthNames[m - 1]).tag(m)
                    }
                }
                .pickerStyle(.wheel)
                .frame(maxWidth: .infinity)

                Picker("Année", selection: $year) {
                    ForEach(2000...2100, id: \.self) { y in
                        Text(String(y)).tag(y)
                    }
                }
                .pickerStyle(.wheel)
                .frame(maxWidth: .infinity)
            }
            .padding()
            .navigationTitle("Choisir le mois")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Aujourd'hui") { store.goToday(); dismiss() }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button("OK") { store.setMonth(year: year, month: month); dismiss() }
                }
            }
            .onAppear { month = store.month; year = store.year }
        }
    }
}
