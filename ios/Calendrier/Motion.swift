import CoreMotion
import SwiftUI

/// Publishes the device tilt (roll/pitch) so the "inox" steel reflection can
/// follow the phone's movement. Uses the gyroscope via CoreMotion — no
/// entitlement or usage description required for raw device motion.
final class MotionManager: ObservableObject {
    static let shared = MotionManager()
    private let mgr = CMMotionManager()

    @Published var roll: Double = 0
    @Published var pitch: Double = 0

    func start() {
        guard mgr.isDeviceMotionAvailable, !mgr.isDeviceMotionActive else { return }
        // 15 Hz is plenty for a subtle reflection and halves the sensor work.
        mgr.deviceMotionUpdateInterval = 1.0 / 15.0
        mgr.startDeviceMotionUpdates(to: .main) { [weak self] motion, _ in
            guard let self, let m = motion else { return }
            // Only publish meaningful movement: a phone lying still still emits
            // jitter, and every publish re-renders the steel views.
            let dr = m.attitude.roll - self.roll
            let dp = m.attitude.pitch - self.pitch
            guard abs(dr) > 0.02 || abs(dp) > 0.02 else { return }
            self.roll = m.attitude.roll
            self.pitch = m.attitude.pitch
        }
    }

    func stop() { mgr.stopDeviceMotionUpdates() }
}

/// Stainless-steel gradient whose light sweep is driven by the device tilt.
func steelGradient(roll: Double, pitch: Double) -> LinearGradient {
    let dx = max(-1.0, min(1.0, roll / 1.2))
    let dy = max(-1.0, min(1.0, (pitch - 0.6) / 1.2))
    return LinearGradient(
        colors: [
            Color(hex: "#fbfcfd"), Color(hex: "#d3d9e0"), Color(hex: "#aab2bd"),
            Color(hex: "#7c848f"), Color(hex: "#aab2bd"), Color(hex: "#e2e7ed"),
            Color(hex: "#ffffff"),
        ],
        startPoint: UnitPoint(x: 0.5 - dx * 0.5, y: 0.5 - dy * 0.5),
        endPoint: UnitPoint(x: 0.5 + dx * 0.5, y: 0.5 + dy * 0.5)
    )
}

/// A metallic circle (dot / colour swatch) whose reflection follows the tilt.
/// Only this leaf re-renders on motion, keeping the rest of the UI still.
struct SteelCircle: View {
    var size: CGFloat
    @EnvironmentObject private var motion: MotionManager
    var body: some View {
        Circle()
            .fill(steelGradient(roll: motion.roll, pitch: motion.pitch))
            .frame(width: size, height: size)
    }
}

/// A metallic vertical bar (agenda event colour) that follows the tilt.
struct SteelBar: View {
    @EnvironmentObject private var motion: MotionManager
    var body: some View {
        RoundedRectangle(cornerRadius: 2)
            .fill(steelGradient(roll: motion.roll, pitch: motion.pitch))
            .frame(width: 4)
    }
}

/// The floating "+" button as a shiny inox disc that follows the tilt. A leaf
/// view so only it re-renders on motion, not the whole screen.
struct SteelFAB: View {
    @EnvironmentObject private var motion: MotionManager
    let action: () -> Void
    var body: some View {
        Button(action: action) {
            Image(systemName: "plus").font(.title.weight(.semibold))
                .foregroundStyle(Color(hex: "#2a2f36"))
                .frame(width: 56, height: 56)
                .background(
                    Circle()
                        .fill(steelGradient(roll: motion.roll, pitch: motion.pitch))
                        .overlay(Circle().strokeBorder(Color(hex: "#9aa2ad"), lineWidth: 1))
                )
                .shadow(color: .black.opacity(0.3), radius: 8, y: 4)
        }
        .padding(20)
    }
}
