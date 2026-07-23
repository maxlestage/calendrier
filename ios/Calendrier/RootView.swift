import SwiftUI

struct RootView: View {
    @StateObject private var store = CalendarStore()

    @State private var editing: EditorTarget?
    @State private var showSettings = false
    @State private var showSearch = false
    @AppStorage("calCollapsed") private var calCollapsed = false

    /// Wrap the optional event so `.sheet(item:)` can drive create *and* edit.
    private struct EditorTarget: Identifiable {
        let id: String
        let event: CalendarEvent?
        let date: Date
    }

    var body: some View {
        VStack(spacing: 8) {
            toolbar
            if let msg = store.errorMessage {
                Text("⚠ \(msg)").font(.footnote).foregroundStyle(.red)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding(8)
                    .background(RoundedRectangle(cornerRadius: 10).fill(Color.red.opacity(0.1)))
            }
            MonthView(collapsed: calCollapsed) { calCollapsed.toggle() }
            AgendaView(
                onEventTap: { ev in editing = EditorTarget(id: "e\(ev.id)", event: ev, date: ev.startDate) },
                onAdd: { editing = EditorTarget(id: "new", event: nil, date: store.selectedDay) }
            )
        }
        .padding(.horizontal, 8)
        .environmentObject(store)
        .overlay(alignment: .bottomTrailing) { fab }
        .sheet(item: $editing) { t in
            EventEditorView(existing: t.event, initialDate: t.date).environmentObject(store)
        }
        .sheet(isPresented: $showSettings) { SettingsView().environmentObject(store) }
        .sheet(isPresented: $showSearch) {
            SearchView(onPick: { store.select($0) }).environmentObject(store)
        }
        .task {
            await store.launch()
            if await Notifications.requestAuthorization() { await store.syncNotifications() }
        }
    }

    private var toolbar: some View {
        HStack {
            Button { store.shiftMonth(-1) } label: { Image(systemName: "chevron.left") }
                .buttonStyle(.bordered)
            Spacer()
            Button { store.goToday() } label: {
                Text("\(frMonthNames[store.month - 1]) \(String(store.year))")
                    .font(.title3).fontWeight(.bold).foregroundStyle(.primary)
            }
            Spacer()
            Button { showSearch = true } label: { Image(systemName: "magnifyingglass") }
                .buttonStyle(.bordered)
            Button { showSettings = true } label: { Image(systemName: "slider.horizontal.3") }
                .buttonStyle(.bordered)
            Button { store.shiftMonth(1) } label: { Image(systemName: "chevron.right") }
                .buttonStyle(.bordered)
        }
        .padding(.top, 4)
    }

    private var fab: some View {
        Button {
            editing = EditorTarget(id: "new", event: nil, date: store.selectedDay)
        } label: {
            Image(systemName: "plus").font(.title.weight(.semibold)).foregroundStyle(.white)
                .frame(width: 56, height: 56)
                .background(Circle().fill(Color.accentColor))
                .shadow(color: Color.accentColor.opacity(0.4), radius: 8, y: 4)
        }
        .padding(20)
    }
}

#Preview {
    RootView()
}
