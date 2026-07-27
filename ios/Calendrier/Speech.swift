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

/// Remove emojis, pictographs and symbols so the voice reads only the words
/// (no guessing at 🎒, ♌, 🌊, ▲…). Keeps letters, digits and punctuation.
private func speakable(_ s: String) -> String {
    let kept = s.unicodeScalars.filter { sc in
        switch sc.value {
        case 0x2190...0x21FF,   // arrows
             0x2300...0x27BF,   // misc symbols, dingbats, zodiac (♈–♓)
             0x2B00...0x2BFF,   // misc symbols and arrows
             0x25A0...0x25FF,   // geometric shapes (▲ ▾)
             0xFE00...0xFE0F,   // variation selectors
             0x20E3,            // combining keycap
             0x1F000...0x1FAFF, // emoji & pictographs
             0x1F1E6...0x1F1FF: // regional indicator flags
            return false
        default:
            return true
        }
    }
    return String(String.UnicodeScalarView(kept))
        .components(separatedBy: .whitespaces)
        .filter { !$0.isEmpty }
        .joined(separator: " ")
}

/// Expand abbreviations so the voice says the words, not the letters.
private func expandForSpeech(_ s: String) -> String {
    var out = s
    out = out.replacingOccurrences(of: "\\bGP\\b", with: "Grand Prix", options: .regularExpression)
    out = out.replacingOccurrences(of: "\\bF1\\b", with: "Formule 1", options: .regularExpression)
    out = out.replacingOccurrences(
        of: "\\bQualifs?\\b", with: "Qualifications",
        options: [.regularExpression, .caseInsensitive]
    )
    return out
}

/// "06:46" → "6 h 46", "19:00" → "19 h" (spoken French time).
private func spokenTime(_ hhmm: String) -> String {
    let parts = hhmm.split(separator: ":")
    guard parts.count == 2 else { return hhmm }
    let h = Int(parts[0]) ?? 0
    let m = parts[1] == "00" ? "" : String(parts[1])
    return "\(h) h \(m)".trimmingCharacters(in: .whitespaces)
}

/// French decimal for speech: "3,5", or "4" when whole.
private func frNum(_ v: Double) -> String {
    let s = String(format: "%.1f", v)
    return s.hasSuffix(".0") ? String(s.dropLast(2)) : s.replacingOccurrences(of: ".", with: ",")
}

/// Read a weather card out loud: the emoji turned into words, then every value
/// shown on the card (temperatures, wind, UV, rain, waves, water, sun, air, …).
private func spokenWeather(_ spot: BeachWeather, _ d: BeachWeatherDay) -> String {
    var parts: [String] = [weatherLabel(d.code)]
    if let mx = d.tmax { parts.append("maximum \(Int(mx.rounded())) degrés") }
    if let mn = d.tmin { parts.append("minimum \(Int(mn.rounded())) degrés") }
    if let w = d.wind { parts.append("vent \(Int(w.rounded())) kilomètres par heure") }
    if let uv = d.uv { parts.append("indice UV \(frNum(uv))") }
    if let p = d.precip { parts.append("\(Int(p.rounded())) pour cent de pluie") }
    if let wv = d.wave { parts.append("vagues \(frNum(wv)) mètres") }
    if let wt = d.water { parts.append("eau à \(Int(wt.rounded())) degrés") }
    if let sr = d.sunrise, let ss = d.sunset {
        parts.append("lever du soleil à \(spokenTime(sr)), coucher à \(spokenTime(ss))")
    }
    if let aq = aqiInfo(d.aqi) {
        parts.append("qualité de l'air \(aq.label.replacingOccurrences(of: "air ", with: ""))")
    }
    if let pol = d.pollen {
        if pol >= 80 { parts.append("pollen fort") }
        else if pol >= 20 { parts.append("pollen modéré") }
    }
    if let fr = fireRisk(tmax: d.tmax, wind: d.wind, precipMm: d.precip_mm) {
        parts.append(fr.replacingOccurrences(of: "🔥 ", with: ""))
    }
    return "\(spot.name) : \(parts.joined(separator: ", "))."
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

    // Beaches and cities read as two named sections, each with its full values.
    func forToday(_ spots: [BeachWeather]) -> [(BeachWeather, BeachWeatherDay)] {
        spots.compactMap { spot in
            spot.days.first(where: { $0.date == dateKey }).map { (spot, $0) }
        }
    }
    let beachWx = forToday(weather.filter { $0.group != "ville" })
    let cityWx = forToday(weather.filter { $0.group == "ville" })
    if !beachWx.isEmpty {
        out.append("Météo des plages.")
        for (spot, d) in beachWx { out.append(spokenWeather(spot, d)) }
    }
    if !cityWx.isEmpty {
        out.append("Météo des villes.")
        for (spot, d) in cityWx { out.append(spokenWeather(spot, d)) }
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

    return expandForSpeech(speakable(out.joined(separator: " ")))
}
