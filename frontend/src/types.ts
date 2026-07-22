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
}

export interface EventPayload {
  title: string;
  description: string | null;
  start: string;
  end: string;
  all_day: boolean;
  color: string | null;
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
}

export interface BeachWeather {
  key: string;
  name: string;
  group: string;
  days: BeachWeatherDay[];
}

export const EVENT_COLORS = [
  "#4f6bed",
  "#0f9d58",
  "#d93025",
  "#f4a300",
  "#8e44ad",
  "#0aa3a3",
] as const;
