import SwiftUI

struct SettingsView: View {
    @ObservedObject var viewModel: CalendarViewModel
    @Environment(\.dismiss) private var dismiss

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
