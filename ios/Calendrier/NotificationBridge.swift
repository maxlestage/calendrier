import Foundation
import UserNotifications
import WebKit

/// Bridges the web app to iOS **local** notifications.
///
/// The web page is the brain: it knows a tide from an F1 session from a
/// personal event, so it computes the list of reminders worth firing and
/// posts it to `window.webkit.messageHandlers.reminders`. This class just
/// schedules whatever it receives.
///
/// Local notifications need **no special capability, entitlement, or extra
/// bundle ID** (that's only for remote/push notifications) — only the user's
/// one-time permission. So this adds nothing to the app's code-signing.
final class NotificationBridge: NSObject, WKScriptMessageHandler, UNUserNotificationCenterDelegate {
    /// JS handler name: `window.webkit.messageHandlers.reminders.postMessage(...)`.
    static let messageName = "reminders"

    /// iOS keeps at most 64 pending local notifications; stay under it.
    private let maxPending = 60

    override init() {
        super.init()
        UNUserNotificationCenter.current().delegate = self
    }

    // MARK: - Receiving the schedule from the web

    func userContentController(
        _ controller: WKUserContentController,
        didReceive message: WKScriptMessage
    ) {
        guard message.name == Self.messageName,
              let items = message.body as? [[String: Any]] else { return }

        let center = UNUserNotificationCenter.current()
        // Ask only when there is actually something to schedule, so the
        // permission prompt appears in a meaningful context.
        center.requestAuthorization(options: [.alert, .sound, .badge]) { granted, _ in
            guard granted else { return }
            self.reschedule(items, center: center)
        }
    }

    /// Replace the whole pending set: the web always sends the current full
    /// schedule, so wiping and rebuilding keeps them in sync with no dupes.
    private func reschedule(_ items: [[String: Any]], center: UNUserNotificationCenter) {
        center.removeAllPendingNotificationRequests()
        let now = Date().timeIntervalSince1970

        let upcoming = items.compactMap { item -> (id: String, title: String, body: String, fireAt: Double)? in
            guard let id = item["id"] as? String,
                  let title = item["title"] as? String,
                  let fireAt = (item["fireAt"] as? NSNumber)?.doubleValue,
                  fireAt > now else { return nil }
            let body = item["body"] as? String ?? ""
            return (id, title, body, fireAt)
        }
        .sorted { $0.fireAt < $1.fireAt }
        .prefix(maxPending)

        for r in upcoming {
            let content = UNMutableNotificationContent()
            content.title = r.title
            content.body = r.body
            content.sound = .default
            let trigger = UNTimeIntervalNotificationTrigger(
                timeInterval: r.fireAt - now,
                repeats: false
            )
            let request = UNNotificationRequest(identifier: r.id, content: content, trigger: trigger)
            center.add(request)
        }
    }

    // MARK: - UNUserNotificationCenterDelegate

    /// Show the banner even when the app is in the foreground.
    func userNotificationCenter(
        _ center: UNUserNotificationCenter,
        willPresent notification: UNNotification,
        withCompletionHandler completionHandler: @escaping (UNNotificationPresentationOptions) -> Void
    ) {
        completionHandler([.banner, .sound, .list])
    }
}
