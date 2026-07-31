import SwiftUI

/// User's appearance choice: follow the system, or force light / dark.
enum ThemePref: String, CaseIterable, Identifiable {
    case auto, light, dark
    var id: String { rawValue }

    var label: String {
        switch self {
        case .auto: return "🌗 Automatique"
        case .light: return "☀️ Clair"
        case .dark: return "🌙 Sombre"
        }
    }

    /// nil = follow the system appearance.
    var colorScheme: ColorScheme? {
        switch self {
        case .auto: return nil
        case .light: return .light
        case .dark: return .dark
        }
    }
}

@main
struct CalendrierApp: App {
    @AppStorage("themePref") private var themeRaw = ThemePref.auto.rawValue

    var body: some Scene {
        WindowGroup {
            // Applied at the root so sheets follow the choice too.
            RootView()
                .preferredColorScheme(ThemePref(rawValue: themeRaw)?.colorScheme)
        }
    }
}
