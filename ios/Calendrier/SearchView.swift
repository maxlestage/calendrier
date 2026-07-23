import SwiftUI

struct SearchView: View {
    @EnvironmentObject var store: CalendarStore
    @Environment(\.dismiss) private var dismiss
    var onPick: (Date) -> Void

    @State private var query = ""
    @State private var results: [CalendarEvent] = []
    @State private var searching = false
    @State private var task: Task<Void, Never>?

    var body: some View {
        NavigationStack {
            List(results) { ev in
                Button {
                    onPick(ev.startDate); dismiss()
                } label: {
                    HStack(spacing: 10) {
                        RoundedRectangle(cornerRadius: 2).fill(Color(hex: ev.color ?? "#4f6bed")).frame(width: 4, height: 34)
                        VStack(alignment: .leading, spacing: 2) {
                            Text(ev.title).font(.callout).fontWeight(.semibold)
                            Text(label(ev)).font(.footnote).foregroundStyle(.secondary)
                        }
                    }
                }
            }
            .overlay {
                if query.count >= 2 && !searching && results.isEmpty {
                    ContentUnavailableView("Aucun résultat", systemImage: "magnifyingglass")
                }
            }
            .searchable(text: $query, prompt: "Titre (Monza, marée, vacances)")
            .onChange(of: query) { _, q in schedule(q) }
            .navigationTitle("Rechercher")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar { ToolbarItem(placement: .cancellationAction) { Button("Fermer") { dismiss() } } }
        }
    }

    private func label(_ ev: CalendarEvent) -> String {
        let d = ev.startDate
        let f = DateFormatter(); f.locale = Locale(identifier: "fr_FR"); f.dateFormat = "d MMMM yyyy"
        return ev.allDay ? f.string(from: d) : "\(f.string(from: d)) · \(d.clock)"
    }

    private func schedule(_ q: String) {
        task?.cancel()
        let trimmed = q.trimmingCharacters(in: .whitespaces)
        guard trimmed.count >= 2 else { results = []; return }
        searching = true
        task = Task {
            try? await Task.sleep(nanoseconds: 250_000_000)
            if Task.isCancelled { return }
            let now = Date()
            let from = now.addingTimeInterval(-365 * 86400)
            let to = now.addingTimeInterval(2 * 365 * 86400)
            let found = (try? await API.events(from: from, to: to, q: trimmed)) ?? []
            if !Task.isCancelled {
                results = Array(found.prefix(50))
                searching = false
            }
        }
    }
}
