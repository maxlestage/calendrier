import AVFoundation
import Foundation

/// French text-to-speech for the day summary (weather + tides + events).
/// Uses AVSpeechSynthesizer — no capability or entitlement needed.
final class Speaker: NSObject, ObservableObject, AVSpeechSynthesizerDelegate {
    static let shared = Speaker()
    private let synth = AVSpeechSynthesizer()
    @Published var speaking = false

    override init() {
        super.init()
        synth.delegate = self
    }

    /// Toggle: speak the text, or stop if already speaking.
    func toggle(_ text: String) {
        if synth.isSpeaking {
            synth.stopSpeaking(at: .immediate)
            speaking = false
            return
        }
        try? AVAudioSession.sharedInstance().setCategory(.playback, options: [.duckOthers])
        try? AVAudioSession.sharedInstance().setActive(true)
        let u = AVSpeechUtterance(string: text)
        u.voice = AVSpeechSynthesisVoice(language: "fr-FR")
        synth.speak(u)
        speaking = true
    }

    func speechSynthesizer(_ s: AVSpeechSynthesizer, didFinish utterance: AVSpeechUtterance) {
        speaking = false
    }

    func speechSynthesizer(_ s: AVSpeechSynthesizer, didCancel utterance: AVSpeechUtterance) {
        speaking = false
    }
}

/// "06:46" → "6 h 46", "19:00" → "19 h" (spoken French time).
private func spokenTime(_ hhmm: String) -> String {
    let parts = hhmm.split(separator: ":")
    guard parts.count == 2 else { return hhmm }
    let h = Int(parts[0]) ?? 0
    let m = parts[1] == "00" ? "" : String(parts[1])
    return "\(h) h \(m)".trimmingCharacters(in: .whitespaces)
}

/// A natural spoken summary of a day: weather, tides, events.
func buildDaySpeech(day: Date, dayEvents: [CalendarEvent], weather: [BeachWeather]) -> String {
    let fmt = DateFormatter()
    fmt.dateFormat = "yyyy-MM-dd"
    let dateKey = fmt.string(from: day)

    let weekday = frWeekdayFull[appCalendar.component(.weekday, from: day) - 1]
    let dnum = appCalendar.component(.day, from: day)
    let mname = frMonthNames[appCalendar.component(.month, from: day) - 1].lowercased()
    var out = ["\(weekday) \(dnum) \(mname)."]

    for spot in weather {
        guard let d = spot.days.first(where: { $0.date == dateKey }) else { continue }
        var s = "\(spot.name) : \(weatherLabel(d.code))"
        if let mx = d.tmax { s += ", \(Int(mx.rounded())) degrés" }
        if let wt = d.water { s += ", eau à \(Int(wt.rounded())) degrés" }
        out.append(s + ".")
    }

    var beaches: [String: (highs: [String], lows: [String])] = [:]
    for ev in dayEvents where ev.isTide {
        let beach = ev.title.components(separatedBy: " — ").first?
            .replacingOccurrences(of: "🌊", with: "").trimmingCharacters(in: .whitespaces) ?? ""
        var rec = beaches[beach] ?? ([], [])
        let t = spokenTime(ev.startDate.clock)
        if ev.title.contains("Pleine mer") { rec.highs.append(t) } else { rec.lows.append(t) }
        beaches[beach] = rec
    }
    for (beach, rec) in beaches {
        var bits: [String] = []
        if !rec.highs.isEmpty { bits.append("pleine mer à " + rec.highs.joined(separator: " et ")) }
        if !rec.lows.isEmpty { bits.append("basse mer à " + rec.lows.joined(separator: " et ")) }
        out.append("Marées à \(beach) : \(bits.joined(separator: ", ")).")
    }

    let evs = dayEvents.filter { !$0.isTide }
    if evs.isEmpty {
        out.append("Aucun événement.")
    } else {
        let list = evs.map { $0.allDay ? $0.title : "\($0.title) à \(spokenTime($0.startDate.clock))" }
        out.append("Événements : \(list.joined(separator: ", ")).")
    }

    return out.joined(separator: " ")
}
