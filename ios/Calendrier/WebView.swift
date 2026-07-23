import SwiftUI
import WebKit

struct WebView: UIViewRepresentable {
    let url: URL
    var onLoaded: () -> Void
    var onFailure: () -> Void

    func makeCoordinator() -> Coordinator {
        Coordinator(onLoaded: onLoaded, onFailure: onFailure)
    }

    func makeUIView(context: Context) -> WKWebView {
        let configuration = WKWebViewConfiguration()
        configuration.allowsInlineMediaPlayback = true

        // Let the web app schedule iOS local notifications through the bridge.
        let controller = WKUserContentController()
        controller.add(context.coordinator.notifications, name: NotificationBridge.messageName)
        configuration.userContentController = controller

        let webView = WKWebView(frame: .zero, configuration: configuration)
        webView.navigationDelegate = context.coordinator
        webView.isOpaque = false
        webView.backgroundColor = UIColor.systemBackground
        webView.scrollView.contentInsetAdjustmentBehavior = .never

        let refresh = UIRefreshControl()
        refresh.addTarget(
            context.coordinator,
            action: #selector(Coordinator.refresh),
            for: .valueChanged
        )
        webView.scrollView.refreshControl = refresh

        context.coordinator.webView = webView
        webView.load(URLRequest(url: url))
        return webView
    }

    func updateUIView(_ webView: WKWebView, context: Context) {
        // Reload only when the target host actually changed (URL edited in
        // the fallback screen), not on every SwiftUI update.
        if webView.url == nil || webView.url?.host != url.host {
            webView.load(URLRequest(url: url))
        }
    }

    final class Coordinator: NSObject, WKNavigationDelegate {
        weak var webView: WKWebView?
        let onLoaded: () -> Void
        let onFailure: () -> Void
        /// Owns the JS↔native notification bridge for this web view's lifetime.
        let notifications = NotificationBridge()

        init(onLoaded: @escaping () -> Void, onFailure: @escaping () -> Void) {
            self.onLoaded = onLoaded
            self.onFailure = onFailure
        }

        @objc func refresh() {
            webView?.reload()
        }

        func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
            webView.scrollView.refreshControl?.endRefreshing()
            onLoaded()
        }

        func webView(_ webView: WKWebView, didFail navigation: WKNavigation!, withError error: Error) {
            webView.scrollView.refreshControl?.endRefreshing()
            onFailure()
        }

        func webView(
            _ webView: WKWebView,
            didFailProvisionalNavigation navigation: WKNavigation!,
            withError error: Error
        ) {
            webView.scrollView.refreshControl?.endRefreshing()
            onFailure()
        }
    }
}
