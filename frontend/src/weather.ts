/** WMO weather code → emoji + label (codes served by /api/beach-weather) */
export function weatherIcon(code: number | null): { emoji: string; label: string } {
  if (code === null) return { emoji: "🌡️", label: "Météo" };
  if (code === 0) return { emoji: "☀️", label: "Ciel dégagé" };
  if (code === 1) return { emoji: "🌤️", label: "Plutôt dégagé" };
  if (code === 2) return { emoji: "⛅", label: "Partiellement nuageux" };
  if (code === 3) return { emoji: "☁️", label: "Couvert" };
  if (code === 45 || code === 48) return { emoji: "🌫️", label: "Brouillard" };
  if (code >= 51 && code <= 57) return { emoji: "🌦️", label: "Bruine" };
  if (code >= 61 && code <= 67) return { emoji: "🌧️", label: "Pluie" };
  if (code >= 71 && code <= 77) return { emoji: "🌨️", label: "Neige" };
  if (code >= 80 && code <= 82) return { emoji: "🌦️", label: "Averses" };
  if (code === 85 || code === 86) return { emoji: "🌨️", label: "Averses de neige" };
  if (code >= 95) return { emoji: "⛈️", label: "Orage" };
  return { emoji: "🌡️", label: "Météo" };
}

const fmt = new Intl.NumberFormat("fr-FR", { maximumFractionDigits: 1 });

export function formatNumber(value: number): string {
  return fmt.format(value);
}

/** European AQI → emoji + French label. */
export function aqiInfo(aqi: number | null): { emoji: string; label: string } | null {
  if (aqi === null) return null;
  if (aqi < 20) return { emoji: "🟢", label: "air bon" };
  if (aqi < 40) return { emoji: "🟡", label: "air moyen" };
  if (aqi < 60) return { emoji: "🟠", label: "air dégradé" };
  if (aqi < 80) return { emoji: "🔴", label: "air mauvais" };
  if (aqi < 100) return { emoji: "🟣", label: "air très mauvais" };
  return { emoji: "🟤", label: "air extrême" };
}

/**
 * Indicative fire-risk from the day's heat, wind and dryness — NOT an
 * official alert. Shown only from "modéré" up. Returns null below that.
 */
export function fireRisk(day: {
  tmax: number | null;
  wind: number | null;
  precip_mm: number | null;
}): string | null {
  if (day.tmax === null) return null;
  let p = 0;
  if (day.tmax >= 32) p += 2;
  else if (day.tmax >= 27) p += 1;
  if (day.wind !== null) {
    if (day.wind >= 40) p += 2;
    else if (day.wind >= 25) p += 1;
  }
  if (day.precip_mm !== null && day.precip_mm < 1) p += 1;
  if (p >= 4) return "🔥 risque feu très élevé";
  if (p >= 3) return "🔥 risque feu élevé";
  if (p >= 2) return "🔥 risque feu modéré";
  return null;
}
