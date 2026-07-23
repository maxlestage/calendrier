import SwiftUI

struct SettingsView: View {
    @EnvironmentObject var store: CalendarStore
    @Environment(\.dismiss) private var dismiss

    @State private var spots: [TideSpot] = []
    @State private var cities: [WeatherCity] = []
    @State private var selectedSpots: Set<String> = []
    @State private var selectedCities: Set<String> = []
    @State private var prefs: NotifPrefs = .fallback
    @AppStorage("serverURL") private var serverURL = API.defaultBase
    @AppStorage("voiceEnabled") private var voiceEnabled = false
    @State private var loaded = false
    @State private var busy = false
    @State private var error: String?

    private let groups: [(id: String, label: String)] = [
        ("ocean", "🌊 Plages de l'océan (Atlantique)"),
        ("mer", "🏖️ Plages de la mer (Méditerranée)"),
        ("manche", "⚓ Manche"),
        ("ports", "🧭 Ports de référence"),
    ]

    var body: some View {
        NavigationStack {
            Form {
                Section("🔔 Notifications") {
                    Toggle("☀️ Résumé du matin", isOn: $prefs.morningBriefing)
                    if prefs.morningBriefing {
                        Picker("Heure du résumé", selection: $prefs.morningHour) {
                            ForEach(0..<24, id: \.self) { h in Text(String(format: "%02dh00", h)).tag(h) }
                        }
                    }
                    Toggle("⏰ Rappel avant mes événements", isOn: $prefs.eventReminders)
                    if prefs.eventReminders {
                        Picker("Délai", selection: $prefs.leadMin) {
                            ForEach([0, 5, 10, 15, 30, 60, 120], id: \.self) { m in
                                Text(m == 0 ? "à l'heure" : "\(m) min avant").tag(m)
                            }
                        }
                    }
                }

                Section {
                    ForEach(cities) { c in row(c.name, selectedCities.contains(c.key)) { toggle(&selectedCities, c.key) } }
                } header: { Text("🏙️ Villes de France — météo") }
                footer: { Text("🎒 Les vacances scolaires suivent automatiquement la zone de tes plages et villes.") }

                ForEach(groups, id: \.id) { g in
                    let inGroup = spots.filter { $0.group == g.id }
                    if !inGroup.isEmpty {
                        Section(g.label) {
                            ForEach(inGroup) { s in row(s.name, selectedSpots.contains(s.key)) { toggle(&selectedSpots, s.key) } }
                        }
                    }
                }

                Section("🔊 Lecture vocale") {
                    Toggle("Bouton pour écouter la journée (météo, marées, événements)", isOn: $voiceEnabled)
                }

                Section {
                    TextField("https://mon-app.herokuapp.com", text: $serverURL)
                        .keyboardType(.URL).textInputAutocapitalization(.never).autocorrectionDisabled()
                } header: { Text("Serveur") }
                footer: { Text("Adresse du backend. À changer seulement si tu héberges ta propre instance.") }

                if let error { Section { Text(error).foregroundStyle(.red).font(.footnote) } }
            }
            .navigationTitle("Réglages")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) { Button("Fermer") { dismiss() } }
                ToolbarItem(placement: .confirmationAction) {
                    Button("Enregistrer") { Task { await save() } }.disabled(busy || !loaded)
                }
            }
            .task { await load() }
        }
    }

    private func row(_ name: String, _ on: Bool, _ tap: @escaping () -> Void) -> some View {
        Button(action: tap) {
            HStack {
                Text(name).foregroundStyle(.primary)
                Spacer()
                if on { Image(systemName: "checkmark").foregroundStyle(Color.accentColor) }
            }
        }
    }

    private func toggle(_ set: inout Set<String>, _ key: String) {
        if set.contains(key) { set.remove(key) } else { set.insert(key) }
    }

    private func load() async {
        spots = (try? await API.tideSpots()) ?? []
        cities = (try? await API.weatherCities()) ?? []
        prefs = (try? await API.prefs()) ?? .fallback
        selectedSpots = Set(spots.filter { $0.selected }.map { $0.key })
        selectedCities = Set(cities.filter { $0.selected }.map { $0.key })
        loaded = true
    }

    private func save() async {
        busy = true; defer { busy = false }
        do {
            _ = try await API.saveTideSpots(Array(selectedSpots))
            _ = try await API.saveWeatherCities(Array(selectedCities))
            _ = try await API.savePrefs(prefs)
            await store.refreshAll()
            dismiss()
        } catch {
            self.error = error.localizedDescription
        }
    }
}
