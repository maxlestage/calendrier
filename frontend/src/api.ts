import type { CalendarEvent, EventPayload, TideSpot } from "./types";

const BASE = "/api";

async function handle<T>(res: Response): Promise<T> {
  if (!res.ok) {
    let message = `${res.status} ${res.statusText}`;
    try {
      const body = await res.json();
      if (body?.error) message = body.error;
    } catch {
      // keep default message
    }
    throw new Error(message);
  }
  if (res.status === 204) return undefined as T;
  return res.json() as Promise<T>;
}

export function fetchEvents(from: string, to: string): Promise<CalendarEvent[]> {
  const params = new URLSearchParams({ from, to });
  return fetch(`${BASE}/events?${params}`).then((res) => handle<CalendarEvent[]>(res));
}

export function createEvent(payload: EventPayload): Promise<CalendarEvent> {
  return fetch(`${BASE}/events`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  }).then((res) => handle<CalendarEvent>(res));
}

export function updateEvent(id: number, payload: EventPayload): Promise<CalendarEvent> {
  return fetch(`${BASE}/events/${id}`, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  }).then((res) => handle<CalendarEvent>(res));
}

export function deleteEvent(id: number): Promise<void> {
  return fetch(`${BASE}/events/${id}`, { method: "DELETE" }).then((res) => handle<void>(res));
}

export function fetchTideSpots(): Promise<TideSpot[]> {
  return fetch(`${BASE}/tide-spots`).then((res) => handle<TideSpot[]>(res));
}

export function saveTideSpots(spots: string[]): Promise<TideSpot[]> {
  return fetch(`${BASE}/tide-spots`, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ spots }),
  }).then((res) => handle<TideSpot[]>(res));
}
