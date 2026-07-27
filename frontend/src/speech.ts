import type { BeachWeather, BeachWeatherDay, CalendarEvent } from "./types";
import { TIDE_COLOR } from "./types";
import { FULL_DAY_NAMES, MONTH_NAMES, toDateKey, toTimeKey } from "./dates";
import { aqiInfo, fireRisk, formatNumber, weatherIcon } from "./weather";

/** Whether the browser can speak (Web Speech API). */
export function speechSupported(): boolean {
  return typeof window !== "undefined" && "speechSynthesis" in window;
}

/** Speak a French utterance, cancelling anything in progress. */
export function speak(text: string, onEnd?: () => void): void {
  if (!speechSupported()) return;
  window.speechSynthesis.cancel();
  const u = new SpeechSynthesisUtterance(text);
  u.lang = "fr-FR";
  u.rate = 1;
  if (onEnd) u.onend = onEnd;
  window.speechSynthesis.speak(u);
}

export function stopSpeech(): void {
  if (speechSupported()) window.speechSynthesis.cancel();
}

/** Remove emojis, pictographs and symbols so the voice reads only the words
 * (no guessing at 🎒, ♌, 🌊, ▲…). Keeps letters, digits and punctuation. */
function speakable(s: string): string {
  return s
    .replace(
      /[\u{2190}-\u{21FF}\u{2300}-\u{27BF}\u{2B00}-\u{2BFF}\u{25A0}-\u{25FF}\u{FE00}-\u{FE0F}\u{20E3}\u{1F000}-\u{1FAFF}\u{1F1E6}-\u{1F1FF}]/gu,
      ""
    )
    .replace(/\s+/g, " ")
    .trim();
}

/** Expand abbreviations so the voice says the words, not the letters. */
function expandForSpeech(s: string): string {
  return s
    .replace(/\bGP\b/g, "Grand Prix")
    .replace(/\bF1\b/g, "Formule 1")
    .replace(/\bQualifs?\b/gi, "Qualifications");
}

/** "06:46" → "6 h 46", "19:00" → "19 h" (spoken French time). */
function spokenTime(hhmm: string): string {
  const [h, m] = hhmm.split(":");
  return `${parseInt(h, 10)} h ${m === "00" ? "" : m}`.trim();
}

/** Read a weather card out loud: the emoji turned into words, then every value
 * shown on the card (temperatures, wind, UV, rain, waves, water, sun, air, …). */
function spokenWeather(spot: BeachWeather, d: BeachWeatherDay): string {
  const parts: string[] = [weatherIcon(d.code).label.toLowerCase()];
  if (d.tmax != null) parts.push(`maximum ${Math.round(d.tmax)} degrés`);
  if (d.tmin != null) parts.push(`minimum ${Math.round(d.tmin)} degrés`);
  if (d.wind != null) parts.push(`vent ${Math.round(d.wind)} kilomètres par heure`);
  if (d.uv != null) parts.push(`indice UV ${formatNumber(d.uv)}`);
  if (d.precip != null) parts.push(`${Math.round(d.precip)} pour cent de pluie`);
  if (d.wave != null) parts.push(`vagues ${formatNumber(d.wave)} mètres`);
  if (d.water != null) parts.push(`eau à ${Math.round(d.water)} degrés`);
  if (d.sunrise && d.sunset) {
    parts.push(`lever du soleil à ${spokenTime(d.sunrise)}, coucher à ${spokenTime(d.sunset)}`);
  }
  const aq = aqiInfo(d.aqi);
  if (aq) parts.push(`qualité de l'air ${aq.label.replace("air ", "")}`);
  if (d.pollen != null && d.pollen >= 80) parts.push("pollen fort");
  else if (d.pollen != null && d.pollen >= 20) parts.push("pollen modéré");
  const fr = fireRisk(d);
  if (fr) parts.push(fr.replace("🔥 ", ""));
  return `${spot.name} : ${parts.join(", ")}.`;
}

/** A natural spoken summary of a day: weather, tides, events. */
export function buildDaySpeech(
  day: Date,
  dayEvents: CalendarEvent[],
  weather: BeachWeather[]
): string {
  const dateKey = toDateKey(day);
  const out: string[] = [
    `${FULL_DAY_NAMES[day.getDay()]} ${day.getDate()} ${MONTH_NAMES[day.getMonth()].toLowerCase()}.`,
  ];

  // Beaches and cities read as two named sections, each with its full values.
  const forToday = (spots: BeachWeather[]) =>
    spots.flatMap((spot) => {
      const d = spot.days.find((x) => x.date === dateKey);
      return d ? [{ spot, d }] : [];
    });
  const beachWx = forToday(weather.filter((s) => s.group !== "ville"));
  const cityWx = forToday(weather.filter((s) => s.group === "ville"));
  if (beachWx.length) {
    out.push("Météo des plages.");
    for (const { spot, d } of beachWx) out.push(spokenWeather(spot, d));
  }
  if (cityWx.length) {
    out.push("Météo des villes.");
    for (const { spot, d } of cityWx) out.push(spokenWeather(spot, d));
  }

  const beaches = new Map<string, { highs: string[]; lows: string[] }>();
  for (const ev of dayEvents.filter((e) => e.color === TIDE_COLOR)) {
    const beach = ev.title.split(" — ")[0].replace("🌊", "").trim();
    const rec = beaches.get(beach) ?? { highs: [], lows: [] };
    const t = spokenTime(toTimeKey(new Date(ev.start)));
    (ev.title.includes("Pleine mer") ? rec.highs : rec.lows).push(t);
    beaches.set(beach, rec);
  }
  for (const [beach, rec] of beaches) {
    const bits: string[] = [];
    if (rec.highs.length) bits.push(`pleine mer à ${rec.highs.join(" et ")}`);
    if (rec.lows.length) bits.push(`basse mer à ${rec.lows.join(" et ")}`);
    out.push(`Marées à ${beach} : ${bits.join(", ")}.`);
  }

  const evs = dayEvents.filter((e) => e.color !== TIDE_COLOR);
  if (evs.length > 0) {
    const list = evs.map((ev) =>
      ev.all_day ? ev.title : `${ev.title} à ${spokenTime(toTimeKey(new Date(ev.start)))}`
    );
    out.push(`Événements : ${list.join(", ")}.`);
  } else {
    out.push("Aucun événement.");
  }

  return expandForSpeech(speakable(out.join(" ")));
}
