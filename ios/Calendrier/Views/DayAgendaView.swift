import SwiftUI

struct DayAgendaView: View {
    @ObservedObject var viewModel: CalendarViewModel
    var onEventTap: (CalendarEvent) -> Void
    var onAdd: () -> Void

    private var dayTitle: String {
        let formatter = DateFormatter()
        formatter.locale = Locale(identifier: "fr_FR")
        formatter.dateFormat = "EEEE d MMMM"
        return formatter.string(from: viewModel.selectedDay).capitalized
    }

    private func time(_ date: Date) -> String {
        let formatter = DateFormatter()
        formatter.locale = Locale(identifier: "fr_FR")
        formatter.timeStyle = .short
        return formatter.string(from: date)
    }

    var body: some View {
        let events = viewModel.eventsOn(viewModel.selectedDay)
        VStack(alignment: .leading, spacing: 8) {
            Text(dayTitle)
                .font(.subheadline.weight(.bold))
                .foregroundStyle(.secondary)
                .padding(.horizontal, 4)
            if events.isEmpty {
                Button(action: onAdd) {
                    Text("Aucun événement — appuyer pour en ajouter un")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 20)
                        .overlay(
                            RoundedRectangle(cornerRadius: 12)
                                .strokeBorder(style: StrokeStyle(lineWidth: 1, dash: [5]))
                                .foregroundStyle(.tertiary)
                        )
                }
                .buttonStyle(.plain)
            } else {
                ForEach(events, id: \.id) { event in
                    Button {
                        onEventTap(event)
                    } label: {
                        HStack(spacing: 10) {
                            RoundedRectangle(cornerRadius: 2)
                                .fill(Color(hex: event.color))
                                .frame(width: 4)
                            VStack(alignment: .leading, spacing: 1) {
                                if event.allDay {
                                    Text("Journée")
                                        .font(.caption.weight(.semibold))
                                        .foregroundStyle(.secondary)
                                } else {
                                    Text("\(time(event.startDate)) – \(time(event.endDate))")
                                        .font(.caption.weight(.semibold))
                                        .foregroundStyle(.secondary)
                                }
                                Text(event.title)
                                    .font(.body.weight(.semibold))
                                    .foregroundStyle(.primary)
                                    .multilineTextAlignment(.leading)
                                if let description = event.description, !description.isEmpty {
                                    Text(description)
                                        .font(.caption)
                                        .foregroundStyle(.secondary)
                                        .lineLimit(2)
                                }
                            }
                            Spacer(minLength: 0)
                        }
                        .padding(12)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .background(Color(.secondarySystemGroupedBackground))
                        .clipShape(RoundedRectangle(cornerRadius: 12))
                    }
                    .buttonStyle(.plain)
                }
            }
        }
    }
}
