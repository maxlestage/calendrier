import SwiftUI

struct EventFormView: View {
    @ObservedObject var viewModel: CalendarViewModel
    let existing: CalendarEvent?
    @Environment(\.dismiss) private var dismiss

    @State private var title: String
    @State private var details: String
    @State private var day: Date
    @State private var allDay: Bool
    @State private var startTime: Date
    @State private var endTime: Date
    @State private var color: String
    @State private var errorMessage: String?
    @State private var busy = false

    init(viewModel: CalendarViewModel, existing: CalendarEvent?, initialDay: Date) {
        self.viewModel = viewModel
        self.existing = existing
        let cal = Calendar.current
        _title = State(initialValue: existing?.title ?? "")
        _details = State(initialValue: existing?.description ?? "")
        _day = State(initialValue: existing.map { cal.startOfDay(for: $0.startDate) } ?? cal.startOfDay(for: initialDay))
        _allDay = State(initialValue: existing?.allDay ?? false)
        let defaultStart = cal.date(bySettingHour: 9, minute: 0, second: 0, of: initialDay) ?? initialDay
        let defaultEnd = cal.date(bySettingHour: 10, minute: 0, second: 0, of: initialDay) ?? initialDay
        _startTime = State(initialValue: (existing != nil && existing!.allDay == false) ? existing!.startDate : defaultStart)
        _endTime = State(initialValue: (existing != nil && existing!.allDay == false) ? existing!.endDate : defaultEnd)
        _color = State(initialValue: existing?.color ?? EventColors.palette[0])
    }

    var body: some View {
        NavigationStack {
            Form {
                Section {
                    TextField("Titre", text: $title)
                    TextField("Description (optionnel)", text: $details, axis: .vertical)
                        .lineLimit(2...4)
                }
                Section {
                    DatePicker("Date", selection: $day, displayedComponents: .date)
                    Toggle("Journée entière", isOn: $allDay)
                    if !allDay {
                        DatePicker("Début", selection: $startTime, displayedComponents: .hourAndMinute)
                        DatePicker("Fin", selection: $endTime, displayedComponents: .hourAndMinute)
                    }
                }
                Section("Couleur") {
                    HStack(spacing: 14) {
                        ForEach(EventColors.palette, id: \.self) { hex in
                            Circle()
                                .fill(Color(hex: hex))
                                .frame(width: 32, height: 32)
                                .overlay(
                                    Circle().strokeBorder(
                                        color == hex ? Color.primary : Color.clear,
                                        lineWidth: 2.5
                                    )
                                )
                                .onTapGesture { color = hex }
                        }
                    }
                }
                if let errorMessage {
                    Section {
                        Text(errorMessage).foregroundStyle(.red)
                    }
                }
                if existing != nil {
                    Section {
                        Button("Supprimer l'événement", role: .destructive) {
                            Task { await remove() }
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
                    Button(existing == nil ? "Créer" : "Enregistrer") {
                        Task { await save() }
                    }
                    .disabled(busy || title.trimmingCharacters(in: .whitespaces).isEmpty)
                }
            }
        }
    }

    private func combine(day: Date, time: Date) -> Date {
        let cal = Calendar.current
        let timeParts = cal.dateComponents([.hour, .minute], from: time)
        return cal.date(
            bySettingHour: timeParts.hour ?? 0,
            minute: timeParts.minute ?? 0,
            second: 0,
            of: day
        ) ?? day
    }

    private func save() async {
        let cal = Calendar.current
        let start: Date
        let end: Date
        if allDay {
            start = cal.startOfDay(for: day)
            end = cal.date(bySettingHour: 23, minute: 59, second: 0, of: day) ?? day
        } else {
            start = combine(day: day, time: startTime)
            end = combine(day: day, time: endTime)
        }
        guard end >= start else {
            errorMessage = "La fin doit être après le début."
            return
        }
        busy = true
        errorMessage = nil
        let formatter = ISO8601DateFormatter()
        let payload = EventPayload(
            title: title.trimmingCharacters(in: .whitespaces),
            description: details.isEmpty ? nil : details,
            start: formatter.string(from: start),
            end: formatter.string(from: end),
            allDay: allDay,
            color: color
        )
        do {
            try await viewModel.save(event: existing, payload: payload)
            dismiss()
        } catch {
            errorMessage = error.localizedDescription
            busy = false
        }
    }

    private func remove() async {
        guard let existing else { return }
        busy = true
        errorMessage = nil
        do {
            try await viewModel.delete(event: existing)
            dismiss()
        } catch {
            errorMessage = error.localizedDescription
            busy = false
        }
    }
}
