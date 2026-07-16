import Foundation

enum APIError: LocalizedError {
    case badURL
    case http(Int, String)

    var errorDescription: String? {
        switch self {
        case .badURL:
            return "URL du serveur invalide — vérifie les réglages."
        case .http(let code, let message):
            return "Erreur serveur (\(code)) : \(message)"
        }
    }
}

struct APIClient {
    let baseURL: String

    private func url(_ path: String, query: [URLQueryItem] = []) throws -> URL {
        let trimmed = baseURL.hasSuffix("/") ? String(baseURL.dropLast()) : baseURL
        guard var components = URLComponents(string: trimmed + path) else {
            throw APIError.badURL
        }
        if !query.isEmpty {
            components.queryItems = query
        }
        guard let url = components.url else { throw APIError.badURL }
        return url
    }

    private func check(_ data: Data, _ response: URLResponse) throws {
        guard let http = response as? HTTPURLResponse else { return }
        guard (200..<300).contains(http.statusCode) else {
            let message = String(data: data, encoding: .utf8) ?? ""
            throw APIError.http(http.statusCode, message)
        }
    }

    func events(from: String, to: String) async throws -> [CalendarEvent] {
        let url = try url("/api/events", query: [
            URLQueryItem(name: "from", value: from),
            URLQueryItem(name: "to", value: to),
        ])
        let (data, response) = try await URLSession.shared.data(from: url)
        try check(data, response)
        return try JSONDecoder().decode([CalendarEvent].self, from: data)
    }

    private func send(_ method: String, _ path: String, body: EventPayload?) async throws -> Data {
        var request = URLRequest(url: try url(path))
        request.httpMethod = method
        if let body {
            request.setValue("application/json", forHTTPHeaderField: "Content-Type")
            request.httpBody = try JSONEncoder().encode(body)
        }
        let (data, response) = try await URLSession.shared.data(for: request)
        try check(data, response)
        return data
    }

    func create(_ payload: EventPayload) async throws -> CalendarEvent {
        let data = try await send("POST", "/api/events", body: payload)
        return try JSONDecoder().decode(CalendarEvent.self, from: data)
    }

    func update(id: Int, _ payload: EventPayload) async throws -> CalendarEvent {
        let data = try await send("PUT", "/api/events/\(id)", body: payload)
        return try JSONDecoder().decode(CalendarEvent.self, from: data)
    }

    func delete(id: Int) async throws {
        _ = try await send("DELETE", "/api/events/\(id)", body: nil)
    }
}
