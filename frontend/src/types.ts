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

export const EVENT_COLORS = [
  "#4f6bed",
  "#0f9d58",
  "#d93025",
  "#f4a300",
  "#8e44ad",
  "#0aa3a3",
] as const;
