import SwiftUI

@main
struct CalendrierApp: App {
    var body: some Scene {
        WindowGroup {
            // Inverted ("dark") look everywhere, sheets included — the inox
            // chrome reads best on dark surfaces.
            RootView().preferredColorScheme(.dark)
        }
    }
}
