import Foundation

enum APIError: LocalizedError {
    case badURL
    case badStatus(Int, String)
    var errorDescription: String? {
        switch self {
        case .badURL: return "Adresse du serveur invalide."
        case let .badStatus(code, msg): return msg.isEmpty ? "Erreur serveur (\(code))." : msg
        }
    }
}

/// Native client for the Rust backend. The base URL is user-editable
/// (@AppStorage "serverURL"), defaulting to the Heroku deployment.
struct API {
    static let defaultBase = "https://calendrier-89594ce603e6.herokuapp.com"

    static var base: String {
        UserDefaults.standard.string(forKey: "serverURL") ?? defaultBase
    }

    private static func url(_ path: String, query: [URLQueryItem] = []) throws -> URL {
        guard var comps = URLComponents(string: base + "/api" + path) else { throw APIError.badURL }
        if !query.isEmpty { comps.queryItems = query }
        guard let u = comps.url else { throw APIError.badURL }
        return u
    }

    private static func run<T: Decodable>(_ request: URLRequest, as: T.Type) async throws -> T {
        let (data, resp) = try await URLSession.shared.data(for: request)
        guard let http = resp as? HTTPURLResponse else { throw APIError.badStatus(0, "") }
        guard (200..<300).contains(http.statusCode) else {
            let msg = (try? JSONDecoder().decode([String: String].self, from: data))?["error"] ?? ""
            throw APIError.badStatus(http.statusCode, msg)
        }
        return try JSONDecoder().decode(T.self, from: data)
    }

    private static func send(_ request: URLRequest) async throws {
        let (data, resp) = try await URLSession.shared.data(for: request)
        guard let http = resp as? HTTPURLResponse, (200..<300).contains(http.statusCode) else {
            let code = (resp as? HTTPURLResponse)?.statusCode ?? 0
            let msg = (try? JSONDecoder().decode([String: String].self, from: data))?["error"] ?? ""
            throw APIError.badStatus(code, msg)
        }
    }

    private static func json<B: Encodable>(_ url: URL, method: String, body: B) throws -> URLRequest {
        var req = URLRequest(url: url)
        req.httpMethod = method
        req.setValue("application/json", forHTTPHeaderField: "Content-Type")
        req.httpBody = try JSONEncoder().encode(body)
        return req
    }

    // MARK: Events

    static func events(from: Date, to: Date, q: String? = nil) async throws -> [CalendarEvent] {
        var query = [
            URLQueryItem(name: "from", value: from.isoString),
            URLQueryItem(name: "to", value: to.isoString),
        ]
        if let q, !q.isEmpty { query.append(URLQueryItem(name: "q", value: q)) }
        return try await run(URLRequest(url: try url("/events", query: query)), as: [CalendarEvent].self)
    }

    static func create(_ payload: EventPayload) async throws -> CalendarEvent {
        try await run(try json(try url("/events"), method: "POST", body: payload), as: CalendarEvent.self)
    }

    static func update(_ id: Int, _ payload: EventPayload) async throws -> CalendarEvent {
        try await run(try json(try url("/events/\(id)"), method: "PUT", body: payload), as: CalendarEvent.self)
    }

    static func delete(_ id: Int) async throws {
        var req = URLRequest(url: try url("/events/\(id)"))
        req.httpMethod = "DELETE"
        try await send(req)
    }

    // MARK: Weather / tides / cities / prefs

    static func beachWeather() async throws -> [BeachWeather] {
        try await run(URLRequest(url: try url("/beach-weather")), as: BeachWeatherResponse.self).spots
    }

    static func tideSpots() async throws -> [TideSpot] {
        try await run(URLRequest(url: try url("/tide-spots")), as: [TideSpot].self)
    }

    static func saveTideSpots(_ spots: [String]) async throws -> [TideSpot] {
        try await run(try json(try url("/tide-spots"), method: "PUT", body: ["spots": spots]),
                      as: [TideSpot].self)
    }

    static func weatherCities() async throws -> [WeatherCity] {
        try await run(URLRequest(url: try url("/weather-cities")), as: [WeatherCity].self)
    }

    static func saveWeatherCities(_ cities: [String]) async throws -> [WeatherCity] {
        try await run(try json(try url("/weather-cities"), method: "PUT", body: ["cities": cities]),
                      as: [WeatherCity].self)
    }

    static func prefs() async throws -> NotifPrefs {
        try await run(URLRequest(url: try url("/prefs")), as: NotifPrefs.self)
    }

    static func savePrefs(_ p: NotifPrefs) async throws -> NotifPrefs {
        try await run(try json(try url("/prefs"), method: "PUT", body: p), as: NotifPrefs.self)
    }
}
