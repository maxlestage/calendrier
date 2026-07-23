import SwiftUI

struct ContentView: View {
    @AppStorage("serverURL") private var serverURL = "https://calendrier-89594ce603e6.herokuapp.com"
    @State private var loadFailed = false
    @State private var isLoading = true
    @State private var draft = ""

    var body: some View {
        Group {
            if !loadFailed, let url = URL(string: serverURL), url.scheme?.hasPrefix("http") == true {
                ZStack {
                    WebView(
                        url: url,
                        onLoaded: {
                            withAnimation(.easeOut(duration: 0.35)) { isLoading = false }
                        },
                        onFailure: {
                            isLoading = false
                            loadFailed = true
                        }
                    )
                    .ignoresSafeArea()

                    if isLoading {
                        splash.transition(.opacity)
                    }
                }
            } else {
                fallback
            }
        }
        .onAppear { draft = serverURL }
    }

    /// Branded loading screen shown until the web app's first page finishes —
    /// covers the network wait (a cold Heroku dyno can take a few seconds)
    /// instead of a blank web view.
    private var splash: some View {
        ZStack {
            Color(.systemBackground).ignoresSafeArea()
            VStack(spacing: 16) {
                Text("🌊")
                    .font(.system(size: 64))
                Text("Calendrier")
                    .font(.title2.weight(.semibold))
                ProgressView()
                    .padding(.top, 4)
            }
        }
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
                isLoading = true
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
