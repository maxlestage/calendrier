import SwiftUI

struct ContentView: View {
    @StateObject private var viewModel = CalendarViewModel()

    private enum Sheet: Identifiable {
        case create
        case edit(CalendarEvent)
        case settings

        var id: String {
            switch self {
            case .create: return "create"
            case .edit(let event): return "edit-\(event.id)"
            case .settings: return "settings"
            }
        }
    }

    @State private var sheet: Sheet?

    var body: some View {
        NavigationStack {
            ZStack(alignment: .bottomTrailing) {
                ScrollView {
                    VStack(spacing: 12) {
                        if let error = viewModel.errorMessage {
                            Text("⚠ \(error)")
                                .font(.footnote)
                                .foregroundStyle(.red)
                                .frame(maxWidth: .infinity, alignment: .leading)
                                .padding(10)
                                .background(Color.red.opacity(0.1))
                                .clipShape(RoundedRectangle(cornerRadius: 10))
                        }
                        MonthGridView(viewModel: viewModel)
                        DayAgendaView(
                            viewModel: viewModel,
                            onEventTap: { sheet = .edit($0) },
                            onAdd: { sheet = .create }
                        )
                        Spacer(minLength: 80)
                    }
                    .padding(.horizontal, 12)
                }
                Button {
                    sheet = .create
                } label: {
                    Image(systemName: "plus")
                        .font(.title2.weight(.semibold))
                        .foregroundStyle(.white)
                        .frame(width: 56, height: 56)
                        .background(Color.accentColor)
                        .clipShape(Circle())
                        .shadow(color: Color.accentColor.opacity(0.4), radius: 8, y: 4)
                }
                .padding(20)
                .accessibilityLabel("Nouvel événement")
            }
            .background(Color(.systemGroupedBackground))
            .navigationTitle(viewModel.monthTitle)
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarLeading) {
                    Button {
                        viewModel.shiftMonth(-1)
                    } label: {
                        Image(systemName: "chevron.left")
                    }
                    .accessibilityLabel("Mois précédent")
                }
                ToolbarItem(placement: .topBarTrailing) {
                    HStack(spacing: 4) {
                        Button {
                            viewModel.goToday()
                        } label: {
                            Image(systemName: "calendar.circle")
                        }
                        .accessibilityLabel("Aujourd'hui")
                        Button {
                            viewModel.shiftMonth(1)
                        } label: {
                            Image(systemName: "chevron.right")
                        }
                        .accessibilityLabel("Mois suivant")
                        Button {
                            sheet = .settings
                        } label: {
                            Image(systemName: "gearshape")
                        }
                        .accessibilityLabel("Réglages")
                    }
                }
            }
            .sheet(item: $sheet) { sheet in
                switch sheet {
                case .create:
                    EventFormView(
                        viewModel: viewModel,
                        existing: nil,
                        initialDay: viewModel.selectedDay
                    )
                case .edit(let event):
                    EventFormView(
                        viewModel: viewModel,
                        existing: event,
                        initialDay: event.startDate
                    )
                case .settings:
                    SettingsView(viewModel: viewModel)
                }
            }
            .task { await viewModel.reload() }
            .refreshable { await viewModel.reload() }
        }
    }
}

#Preview {
    ContentView()
}
