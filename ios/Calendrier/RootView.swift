import Combine
import SwiftUI

struct RootView: View {
    @StateObject private var store = CalendarStore()
    /// Plain reference, deliberately NOT @StateObject/@ObservedObject: observing
    /// it here would re-render the whole screen at the sensor's rate. Only the
    /// leaf views (SteelFAB, SteelCircle, SteelBar) subscribe via @EnvironmentObject.
    private let motion = MotionManager.shared
    @Environment(\.scenePhase) private var scenePhase

    @State private var editing: EditorTarget?
    @State private var showSettings = false
    @State private var showSearch = false
    @State private var showScan = false
    @State private var showMonthPicker = false
    @State private var ready = false
    @AppStorage("calCollapsed") private var calCollapsed = false
    @AppStorage("voiceEnabled") private var voiceEnabled = true

    /// Recompute the weather every hour.
    private let hourly = Timer.publish(every: 3600, on: .main, in: .common).autoconnect()

    /// Wrap the optional event so `.sheet(item:)` can drive create *and* edit.
    private struct EditorTarget: Identifiable {
        let id: String
        let event: CalendarEvent?
        let date: Date
    }

    var body: some View {
        ZStack {
            mainView
            if !ready {
                SplashView().transition(.opacity)
            }
        }
        .task {
            motion.start()
            await store.launch()
            withAnimation(.easeOut(duration: 0.4)) { ready = true }
            if await Notifications.requestAuthorization() { await store.syncNotifications() }
        }
        .onReceive(hourly) { _ in
            Task { await store.loadWeather() }
        }
        .onChange(of: scenePhase) { _, phase in
            // Initial launch is handled by .task; react to later transitions.
            guard ready else { return }
            switch phase {
            case .active:
                motion.start()
                Task { await store.onForeground() }
            case .background:
                motion.stop()
                Task { await store.backupLocally() }
            default:
                break
            }
        }
    }

    private var mainView: some View {
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
                voiceEnabled: voiceEnabled,
                onEventTap: { ev in editing = EditorTarget(id: "e\(ev.id)", event: ev, date: ev.startDate) },
                onAdd: { editing = EditorTarget(id: "new", event: nil, date: store.selectedDay) }
            )
        }
        .padding(.horizontal, 8)
        .overlay(alignment: .bottomTrailing) {
            SteelFAB { editing = EditorTarget(id: "new", event: nil, date: store.selectedDay) }
        }
        .environmentObject(store)
        .environmentObject(motion)
        .sheet(item: $editing) { t in
            EventEditorView(existing: t.event, initialDate: t.date)
                .environmentObject(store)
                .environmentObject(motion)
        }
        .sheet(isPresented: $showSettings) { SettingsView().environmentObject(store) }
        .sheet(isPresented: $showSearch) {
            SearchView(onPick: { store.select($0) }).environmentObject(store)
        }
        .sheet(isPresented: $showScan) { ScanScheduleView().environmentObject(store) }
        .sheet(isPresented: $showMonthPicker) {
            MonthPickerView().environmentObject(store)
                .presentationDetents([.height(280)])
        }
    }

    private var toolbar: some View {
        HStack {
            Button { store.shiftMonth(-1) } label: { Image(systemName: "chevron.left") }
                .buttonStyle(.bordered)
            Spacer()
            Button { showMonthPicker = true } label: {
                Text("\(frMonthNames[store.month - 1]) \(String(store.year))")
                    .font(.title3).fontWeight(.bold).foregroundStyle(.primary)
            }
            Spacer()
            Button { showSearch = true } label: { Image(systemName: "magnifyingglass") }
                .buttonStyle(.bordered)
            Button { showScan = true } label: { Image(systemName: "doc.viewfinder") }
                .buttonStyle(.bordered)
            Button { showSettings = true } label: { Image(systemName: "slider.horizontal.3") }
                .buttonStyle(.bordered)
            Button { store.shiftMonth(1) } label: { Image(systemName: "chevron.right") }
                .buttonStyle(.bordered)
        }
        .padding(.top, 4)
    }

}

#Preview {
    RootView()
}
