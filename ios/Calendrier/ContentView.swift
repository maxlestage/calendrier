import SwiftUI

struct ContentView: View {
    @AppStorage("serverURL") private var serverURL = "https://calendrier-89594ce603e6.herokuapp.com"
    @State private var loadFailed = false
    @State private var draft = ""

    var body: some View {
        Group {
            if !loadFailed, let url = URL(string: serverURL), url.scheme?.hasPrefix("http") == true {
                WebView(url: url) {
                    loadFailed = true
                }
                .ignoresSafeArea()
            } else {
                fallback
            }
        }
        .onAppear { draft = serverURL }
    }

    /// Shown when the page cannot load (server down, wrong URL): lets the
    /// user fix the server address without rebuilding the app.
    private var fallback: some View {
        VStack(spacing: 16) {
            Image(systemName: "wifi.exclamationmark")
                .font(.largeTitle)
                .foregroundStyle(.secondary)
            Text("Impossible de charger le calendrier")
                .font(.headline)
            Text("Vérifie ta connexion ou l'adresse du serveur.")
                .font(.subheadline)
                .foregroundStyle(.secondary)
            TextField("https://mon-app.herokuapp.com", text: $draft)
                .keyboardType(.URL)
                .textInputAutocapitalization(.never)
                .autocorrectionDisabled()
                .textFieldStyle(.roundedBorder)
                .padding(.horizontal, 24)
            Button("Réessayer") {
                let trimmed = draft.trimmingCharacters(in: .whitespaces)
                if !trimmed.isEmpty {
                    serverURL = trimmed
                }
                loadFailed = false
            }
            .buttonStyle(.borderedProminent)
        }
        .padding()
    }
}

#Preview {
    ContentView()
}
