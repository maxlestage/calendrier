import SwiftUI

extension Color {
    /// "#rrggbb" → Color; falls back to the app accent blue.
    init(hex: String?) {
        let fallback = (r: 79.0, g: 107.0, b: 237.0)
        guard let hex, hex.hasPrefix("#"), hex.count == 7,
              let value = UInt32(hex.dropFirst(), radix: 16)
        else {
            self = Color(
                red: fallback.r / 255.0,
                green: fallback.g / 255.0,
                blue: fallback.b / 255.0
            )
            return
        }
        self = Color(
            red: Double((value >> 16) & 0xFF) / 255.0,
            green: Double((value >> 8) & 0xFF) / 255.0,
            blue: Double(value & 0xFF) / 255.0
        )
    }
}
