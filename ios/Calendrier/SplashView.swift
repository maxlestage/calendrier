import SwiftUI

/// Cover / loading screen shown at launch until the first data load finishes.
///
/// Deliberately keeps the brushed-steel field of the app icon whatever the
/// chosen theme, so the icon appears to expand into the app.
struct SplashView: View {
    @State private var appeared = false
    /// Position of the light sweeping across the progress track, -1 → 1.
    @State private var sweep = -1.0

    private let steel = LinearGradient(
        colors: [
            Color(hex: "#454e59"), Color(hex: "#252b32"), Color(hex: "#616c78"),
            Color(hex: "#272d34"), Color(hex: "#171b1f"),
        ],
        startPoint: .topLeading, endPoint: .bottomTrailing
    )

    var body: some View {
        ZStack {
            steel.ignoresSafeArea()

            VStack(spacing: 0) {
                Spacer()

                Image("AppLogo")
                    .resizable()
                    .scaledToFit()
                    .frame(width: 116, height: 116)
                    .clipShape(RoundedRectangle(cornerRadius: 26, style: .continuous))
                    .overlay(
                        RoundedRectangle(cornerRadius: 26, style: .continuous)
                            .strokeBorder(.white.opacity(0.18), lineWidth: 1)
                    )
                    .shadow(color: .black.opacity(0.45), radius: 18, y: 10)
                    .scaleEffect(appeared ? 1 : 0.92)
                    .opacity(appeared ? 1 : 0)

                Text("Calendrier")
                    .font(.system(.largeTitle, design: .rounded).weight(.bold))
                    .foregroundStyle(.white)
                    .kerning(0.5)
                    .padding(.top, 22)
                    .opacity(appeared ? 1 : 0)

                progressTrack
                    .padding(.top, 26)
                    .opacity(appeared ? 1 : 0)

                Spacer()
                credit.padding(.bottom, 40)
            }
            .padding(.horizontal, 32)
        }
        .onAppear {
            withAnimation(.easeOut(duration: 0.5)) { appeared = true }
            withAnimation(.easeInOut(duration: 1.3).repeatForever(autoreverses: false)) {
                sweep = 1
            }
        }
    }

    /// Slim track with a highlight sweeping through it — quieter than a
    /// spinner and echoes the moving reflection on the app's steel surfaces.
    private var progressTrack: some View {
        GeometryReader { geo in
            let w = geo.size.width
            Capsule().fill(.white.opacity(0.12))
                .overlay(alignment: .leading) {
                    Capsule()
                        .fill(
                            LinearGradient(
                                colors: [.clear, .white.opacity(0.85), .clear],
                                startPoint: .leading, endPoint: .trailing
                            )
                        )
                        .frame(width: w * 0.4)
                        .offset(x: sweep * w * 0.75)
                }
                .clipShape(Capsule())
        }
        .frame(width: 150, height: 4)
    }

    private var credit: some View {
        VStack(spacing: 3) {
            Text("Créé et développé par")
                .font(.caption)
                .foregroundStyle(.white.opacity(0.55))
            Text("Maxime Nathan Lestage")
                .font(.subheadline.weight(.semibold))
                .foregroundStyle(.white.opacity(0.92))
        }
        .opacity(appeared ? 1 : 0)
    }
}

#Preview {
    SplashView()
}
