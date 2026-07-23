import SwiftUI

/// Cover / loading screen shown at launch with the app logo, until the first
/// data load finishes. Adapts to light/dark (system background), so it flows
/// seamlessly from the iOS launch screen.
struct SplashView: View {
    /// Slight breathing animation on the logo while loading.
    @State private var pulse = false

    var body: some View {
        ZStack {
            Color(.systemBackground).ignoresSafeArea()
            VStack(spacing: 22) {
                Image("AppLogo")
                    .resizable()
                    .scaledToFit()
                    .frame(width: 120, height: 120)
                    .clipShape(RoundedRectangle(cornerRadius: 26, style: .continuous))
                    .shadow(color: .black.opacity(0.15), radius: 12, y: 5)
                    .scaleEffect(pulse ? 1.04 : 1.0)
                    .animation(.easeInOut(duration: 1.1).repeatForever(autoreverses: true), value: pulse)

                Text("Calendrier")
                    .font(.title.weight(.bold))

                ProgressView()
                    .padding(.top, 2)
            }
        }
        .onAppear { pulse = true }
    }
}

#Preview {
    SplashView()
}
