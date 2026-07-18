import Foundation
import UserNotifications

enum NotificationScheduler {
    static func requestPermission() async -> Bool {
        let center = UNUserNotificationCenter.current()
        return (try? await center.requestAuthorization(options: [.alert, .sound, .badge])) ?? false
    }

    /// Replaces all pending reminders with the upcoming events: 1 hour
    /// before a timed event, 9:00 the same day for an all-day event.
    /// iOS caps pending local notifications at 64, so we keep the next 60.
    static func reschedule(events: [CalendarEvent]) async {
        let center = UNUserNotificationCenter.current()
        center.removeAllPendingNotificationRequests()
        let calendar = Calendar.current
        let now = Date()
        let upcoming = events
            .filter { $0.startDate > now }
            .sorted { $0.start < $1.start }
            .prefix(60)
        for event in upcoming {
            let fireDate: Date
            let body: String
            if event.allDay {
                guard let nine = calendar.date(bySettingHour: 9, minute: 0, second: 0, of: event.startDate) else {
                    continue
                }
                fireDate = nine
                body = "Aujourd'hui"
            } else {
                fireDate = event.startDate.addingTimeInterval(-3600)
                let formatter = DateFormatter()
                formatter.locale = Locale(identifier: "fr_FR")
                formatter.timeStyle = .short
                body = "Dans 1 heure — à \(formatter.string(from: event.startDate))"
            }
            guard fireDate > now else { continue }
            let content = UNMutableNotificationContent()
            content.title = event.title
            content.body = body
            content.sound = .default
            let components = calendar.dateComponents([.year, .month, .day, .hour, .minute], from: fireDate)
            let trigger = UNCalendarNotificationTrigger(dateMatching: components, repeats: false)
            let request = UNNotificationRequest(
                identifier: "event-\(event.id)",
                content: content,
                trigger: trigger
            )
            try? await center.add(request)
        }
    }

    static func cancelAll() {
        UNUserNotificationCenter.current().removeAllPendingNotificationRequests()
    }
}
