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
