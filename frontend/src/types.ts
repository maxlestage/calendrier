export type Recurrence = "weekly" | "monthly" | "yearly" | null;

export interface CalendarEvent {
  id: number;
  title: string;
  description: string | null;
  /** ISO 8601 datetime (UTC) */
  start: string;
  /** ISO 8601 datetime (UTC) */
  end: string;
  all_day: boolean;
  color: string | null;
  recurrence: Recurrence;
}

export interface EventPayload {
  title: string;
  description: string | null;
  start: string;
  end: string;
  all_day: boolean;
  color: string | null;
  recurrence: Recurrence;
}

/** A candidate event returned by /api/parse-schedule for the user to review. */
export interface DraftEvent {
  title: string;
  start: string;
  end: string;
  all_day: boolean;
}

export interface TideSpot {
  key: string;
  name: string;
  group: "ocean" | "mer" | "manche" | "ports";
  selected: boolean;
}

/** Sea-blue used by the backend for tide events */
export const TIDE_COLOR = "#0277bd";

/** One forecast day for a beach (Open-Meteo, served by /api/beach-weather) */
export interface BeachWeatherDay {
  /** "YYYY-MM-DD" (Paris) */
  date: string;
  /** WMO weather code */
  code: number | null;
  tmax: number | null;
  tmin: number | null;
  /** km/h */
  wind: number | null;
  uv: number | null;
  /** % */
  precip: number | null;
  /** m */
  wave: number | null;
  /** °C, sea surface around midday */
  water: number | null;
  /** "HH:MM" (Paris) */
  sunrise: string | null;
  sunset: string | null;
  /** Worst pollen of the day, grains/m³ (≈4-day horizon) */
  pollen: number | null;
  /** Worst European air-quality index of the day (0 bon … 100+ extrême) */
  aqi: number | null;
  /** Precipitation total of the day, mm (fire-risk input) */
  precip_mm: number | null;
}

export interface BeachWeather {
  key: string;
  name: string;
  /** Coast group ("ocean", "mer", …) or "ville" for a city */
  group: string;
  days: BeachWeatherDay[];
}

export interface WeatherCity {
  key: string;
  name: string;
  selected: boolean;
}

/** Notification preferences (snake_case to match the backend JSON) */
export interface NotifPrefs {
  /** Hour (0–23, device-local) of the daily morning briefing */
  morning_hour: number;
  /** Minutes before a timed event to fire its reminder */
  lead_min: number;
  morning_briefing: boolean;
  event_reminders: boolean;
}

export const DEFAULT_PREFS: NotifPrefs = {
  morning_hour: 7,
  lead_min: 15,
  morning_briefing: true,
  event_reminders: true,
};

/** Sentinel stored on events that should render as shiny 3D stainless steel. */
export const STEEL = "#c0c6cf";

/** Chrome / brushed-steel gradient used to render the STEEL colour in 3D. */
export const STEEL_GRADIENT =
  "linear-gradient(135deg,#fbfcfd 0%,#d3d9e0 16%,#aab2bd 38%,#7c848f 50%,#aab2bd 62%,#e2e7ed 84%,#ffffff 100%)";

export const EVENT_COLORS = [
  "#4f6bed",
  "#0f9d58",
  "#d93025",
  "#f4a300",
  STEEL,
  "#0aa3a3",
] as const;

/** CSS background for an event colour — a shiny stainless-steel gradient for STEEL. */
export function colorBackground(color: string | null): string {
  return color === STEEL ? STEEL_GRADIENT : (color ?? "#4f6bed");
}
