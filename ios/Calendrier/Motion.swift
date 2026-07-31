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
        mgr.deviceMotionUpdateInterval = 1.0 / 30.0
        mgr.startDeviceMotionUpdates(to: .main) { [weak self] motion, _ in
            guard let self, let m = motion else { return }
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
