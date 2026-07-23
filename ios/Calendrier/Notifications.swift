import Foundation
import UserNotifications

/// Shows notifications even when the app is in the foreground.
final class NotificationDelegate: NSObject, UNUserNotificationCenterDelegate {
    static let shared = NotificationDelegate()
    func userNotificationCenter(
        _ center: UNUserNotificationCenter,
        willPresent notification: UNNotification,
        withCompletionHandler completionHandler: @escaping (UNNotificationPresentationOptions) -> Void
    ) {
        completionHandler([.banner, .sound, .list])
    }
}

/// Schedules local notifications natively from the fetched events + weather,
/// mirroring what the web used to compute. Local notifications need no special
/// capability or entitlement — only the user's permission.
enum Notifications {
    static func requestAuthorization() async -> Bool {
        UNUserNotificationCenter.current().delegate = NotificationDelegate.shared
        return (try? await UNUserNotificationCenter.current()
            .requestAuthorization(options: [.alert, .sound, .badge])) ?? false
    }

    private struct Item {
        let id: String
        let title: String
        let body: String
        let fireAt: Date
    }

    /// Rebuild the whole pending set from the current data + prefs.
    static func reschedule(events: [CalendarEvent], weather: [BeachWeather], prefs: NotifPrefs) async {
        let center = UNUserNotificationCenter.current()
        let settings = await center.notificationSettings()
        guard settings.authorizationStatus == .authorized || settings.authorizationStatus == .provisional
        else { return }

        var items: [Item] = []
        if prefs.eventReminders { items += eventReminders(events, leadMin: prefs.leadMin) }
        if prefs.morningBriefing { items += morningBriefings(events, weather, hour: prefs.morningHour) }

        center.removeAllPendingNotificationRequests()
        let now = Date()
        let soonest = items.filter { $0.fireAt > now }.sorted { $0.fireAt < $1.fireAt }.prefix(60)
        for it in soonest {
            let content = UNMutableNotificationContent()
            content.title = it.title
            content.body = it.body
            content.sound = .default
            let interval = it.fireAt.timeIntervalSince(now)
            let trigger = UNTimeIntervalNotificationTrigger(timeInterval: max(interval, 1), repeats: false)
            try? await center.add(UNNotificationRequest(identifier: it.id, content: content, trigger: trigger))
        }
    }

    // MARK: Builders (mirror the former web logic)

    private static func eventReminders(_ events: [CalendarEvent], leadMin: Int) -> [Item] {
        let now = Date()
        let horizon = now.addingTimeInterval(14 * 86400)
        let lead = TimeInterval(leadMin * 60)
        return events.compactMap { ev in
            guard !ev.allDay, !ev.isTide else { return nil }
            let start = ev.startDate
            guard start > now, start < horizon else { return nil }
            let fireAt = max(start.addingTimeInterval(-lead), now.addingTimeInterval(1))
            let body = (ev.description?.isEmpty == false)
                ? "\(start.clock) · \(ev.description!)" : "À \(start.clock)"
            return Item(id: "event-\(ev.id)-\(ev.start)", title: ev.title, body: body, fireAt: fireAt)
        }
    }

    private struct Day { var weather: [String] = []; var tides: [String: [(Bool, String)]] = [:]; var events: [(String?, String)] = [] }

    private static func morningBriefings(_ events: [CalendarEvent], _ weather: [BeachWeather], hour: Int) -> [Item] {
        var days: [String: Day] = [:]
        func key(_ d: Date) -> String {
            let f = DateFormatter(); f.dateFormat = "yyyy-MM-dd"; return f.string(from: d)
        }

        for place in weather {
            for d in place.days where d.tmax != nil || d.code != nil {
                let temp = d.tmax != nil
                    ? "\(Int(d.tmax!.rounded()))°" + (d.tmin != nil ? "/\(Int(d.tmin!.rounded()))°" : "")
                    : ""
                days[d.date, default: Day()].weather.append("\(place.name) \(weatherEmoji(d.code)) \(temp)")
            }
        }
        for ev in events {
            let k = key(ev.startDate)
            if ev.isTide {
                let beach = ev.title.components(separatedBy: " — ").first?
                    .replacingOccurrences(of: "🌊", with: "").trimmingCharacters(in: .whitespaces) ?? ""
                days[k, default: Day()].tides[beach, default: []].append((ev.title.contains("Pleine mer"), ev.startDate.clock))
            } else {
                days[k, default: Day()].events.append((ev.allDay ? nil : ev.startDate.clock, ev.title))
            }
        }

        var out: [Item] = []
        for (dayKey, content) in days {
            let parts = dayKey.split(separator: "-").compactMap { Int($0) }
            guard parts.count == 3 else { continue }
            var comps = DateComponents()
            comps.year = parts[0]; comps.month = parts[1]; comps.day = parts[2]; comps.hour = hour; comps.minute = 0
            guard let fire = appCalendar.date(from: comps), fire > Date() else { continue }
            var lines: [String] = []
            if !content.weather.isEmpty { lines.append("☀️ " + content.weather.joined(separator: " · ")) }
            for (beach, list) in content.tides {
                let highs = list.filter { $0.0 }.map { $0.1 }.joined(separator: " ")
                let lows = list.filter { !$0.0 }.map { $0.1 }.joined(separator: " ")
                lines.append("🌊 \(beach) — PM \(highs.isEmpty ? "—" : highs) · BM \(lows.isEmpty ? "—" : lows)")
            }
            if !content.events.isEmpty {
                let ev = content.events.sorted { ($0.0 ?? "") < ($1.0 ?? "") }
                    .map { $0.0 != nil ? "\($0.0!) \($0.1)" : $0.1 }.joined(separator: " · ")
                lines.append("📅 " + ev)
            }
            guard !lines.isEmpty else { continue }
            out.append(Item(id: "briefing-\(dayKey)", title: "☀️ Ta journée",
                            body: lines.joined(separator: "\n"), fireAt: fire))
        }
        return out
    }
}
