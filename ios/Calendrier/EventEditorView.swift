import SwiftUI

struct EventEditorView: View {
    @EnvironmentObject var store: CalendarStore
    @Environment(\.dismiss) private var dismiss

    /// nil = create; non-nil = edit.
    let existing: CalendarEvent?
    let initialDate: Date

    @State private var title = ""
    @State private var description = ""
    @State private var date = Date()
    @State private var startTime = Date()
    @State private var endTime = Date()
    @State private var allDay = false
    @State private var color = eventColors[0]
    @State private var recurrence = ""
    @State private var error: String?
    @State private var busy = false

    var body: some View {
        NavigationStack {
            Form {
                Section {
                    TextField("Titre", text: $title)
                    TextField("Description (optionnel)", text: $description, axis: .vertical)
                }
                Section {
                    DatePicker("Date", selection: $date, displayedComponents: .date)
                    Toggle("Journée entière", isOn: $allDay)
                    if !allDay {
                        DatePicker("Début", selection: $startTime, displayedComponents: .hourAndMinute)
                        DatePicker("Fin", selection: $endTime, displayedComponents: .hourAndMinute)
                    }
                }
                Section("Couleur") {
                    HStack(spacing: 12) {
                        ForEach(eventColors, id: \.self) { c in
                            Circle().fill(Color(hex: c)).frame(width: 30, height: 30)
                                .overlay { if c == color { Circle().strokeBorder(.primary, lineWidth: 3) } }
                                .onTapGesture { color = c }
                        }
                    }
                }
                Section("Répétition") {
                    Picker("Répétition", selection: $recurrence) {
                        Text("Jamais").tag("")
                        Text("Chaque semaine").tag("weekly")
                        Text("Chaque mois").tag("monthly")
                        Text("Chaque année").tag("yearly")
                    }
                    if existing != nil && !recurrence.isEmpty {
                        Text("Modifier ou supprimer agit sur toute la série.")
                            .font(.footnote).foregroundStyle(.secondary)
                    }
                }
                if let error {
                    Section { Text(error).foregroundStyle(.red).font(.footnote) }
                }
                if existing != nil {
                    Section {
                        Button(role: .destructive) { Task { await remove() } } label: {
                            Text("Supprimer").frame(maxWidth: .infinity)
                        }
                        .disabled(busy)
                    }
                }
            }
            .navigationTitle(existing == nil ? "Nouvel événement" : "Modifier")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Annuler") { dismiss() }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button(existing == nil ? "Créer" : "OK") { Task { await submit() } }
                        .disabled(busy)
                }
            }
            .onAppear(perform: prime)
        }
    }

    private func prime() {
        if let ev = existing {
            title = ev.title
            description = ev.description ?? ""
            date = ev.startDate
            startTime = ev.startDate
            endTime = ev.endDate
            allDay = ev.allDay
            color = ev.color ?? eventColors[0]
            recurrence = ev.recurrence ?? ""
        } else {
            date = initialDate
            let cal = appCalendar
            startTime = cal.date(bySettingHour: 9, minute: 0, second: 0, of: initialDate) ?? initialDate
            endTime = cal.date(bySettingHour: 10, minute: 0, second: 0, of: initialDate) ?? initialDate
        }
    }

    private func combined(_ time: Date) -> Date {
        let d = appCalendar.dateComponents([.year, .month, .day], from: date)
        let t = appCalendar.dateComponents([.hour, .minute], from: time)
        var c = DateComponents()
        c.year = d.year; c.month = d.month; c.day = d.day; c.hour = t.hour; c.minute = t.minute
        return appCalendar.date(from: c) ?? date
    }

    private func submit() async {
        let name = title.trimmingCharacters(in: .whitespaces)
        guard !name.isEmpty else { error = "Le titre est obligatoire."; return }

        let start: Date
        let end: Date
        if allDay {
            start = appCalendar.startOfDay(for: date)
            end = appCalendar.date(bySettingHour: 23, minute: 59, second: 0, of: date) ?? start
        } else {
            start = combined(startTime)
            end = combined(endTime)
        }
        guard end >= start else { error = "La fin doit être après le début."; return }

        let payload = EventPayload(
            title: name,
            description: description.isEmpty ? nil : description,
            start: start.isoString, end: end.isoString,
            allDay: allDay, color: color,
            recurrence: recurrence.isEmpty ? nil : recurrence
        )
        busy = true; defer { busy = false }
        do { try await store.save(payload, editing: existing?.id); dismiss() }
        catch { self.error = error.localizedDescription }
    }

    private func remove() async {
        guard let id = existing?.id else { return }
        busy = true; defer { busy = false }
        do { try await store.delete(id); dismiss() }
        catch { self.error = error.localizedDescription }
    }
}
