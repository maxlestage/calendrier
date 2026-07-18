import SwiftUI

struct SettingsView: View {
    @ObservedObject var viewModel: CalendarViewModel
    @Environment(\.dismiss) private var dismiss
    @AppStorage("notificationsEnabled") private var notificationsEnabled = false

    var body: some View {
        NavigationStack {
            Form {
                Section("Serveur") {
                    TextField("https://mon-app.herokuapp.com", text: viewModel.$serverURL)
                        .keyboardType(.URL)
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()
                } footer: {
                    Text("URL du backend Calendrier (Actix Web). L'app parle à /api/events.")
                }
                Section("Rappels") {
                    Toggle("Notifications avant les événements", isOn: $notificationsEnabled)
                        .onChange(of: notificationsEnabled) { _, enabled in
                            Task {
                                if enabled {
                                    let granted = await NotificationScheduler.requestPermission()
                                    if granted {
                                        await viewModel.reload()
                                    } else {
                                        notificationsEnabled = false
                                    }
                                } else {
                                    NotificationScheduler.cancelAll()
                                }
                            }
                        }
                } footer: {
                    Text("1 h avant un événement horodaté, à 9 h le jour même pour un événement « journée entière ».")
                }
            }
            .navigationTitle("Réglages")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button("OK") {
                        Task { await viewModel.reload() }
                        dismiss()
                    }
                }
            }
        }
    }
}
