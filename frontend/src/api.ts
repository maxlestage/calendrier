import type { BeachWeather, CalendarEvent, EventPayload, TideSpot, WeatherCity } from "./types";

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

/** Title search over a wide window (1 year back, 2 years ahead) */
export function searchEvents(q: string): Promise<CalendarEvent[]> {
  const now = Date.now();
  const from = new Date(now - 365 * 86400_000).toISOString();
  const to = new Date(now + 2 * 365 * 86400_000).toISOString();
  const params = new URLSearchParams({ from, to, q });
  return fetch(`${BASE}/events?${params}`).then((res) => handle<CalendarEvent[]>(res));
}

export function fetchSchoolZone(): Promise<string> {
  return fetch(`${BASE}/school-zone`)
    .then((res) => handle<{ zone: string }>(res))
    .then((b) => b.zone);
}

export function saveSchoolZone(zone: string): Promise<string> {
  return fetch(`${BASE}/school-zone`, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ zone }),
  })
    .then((res) => handle<{ zone: string }>(res))
    .then((b) => b.zone);
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

export function fetchBeachWeather(): Promise<BeachWeather[]> {
  return fetch(`${BASE}/beach-weather`)
    .then((res) => handle<{ spots: BeachWeather[] }>(res))
    .then((body) => body.spots);
}

export function saveTideSpots(spots: string[]): Promise<TideSpot[]> {
  return fetch(`${BASE}/tide-spots`, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ spots }),
  }).then((res) => handle<TideSpot[]>(res));
}

export function fetchWeatherCities(): Promise<WeatherCity[]> {
  return fetch(`${BASE}/weather-cities`).then((res) => handle<WeatherCity[]>(res));
}

export function saveWeatherCities(cities: string[]): Promise<WeatherCity[]> {
  return fetch(`${BASE}/weather-cities`, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ cities }),
  }).then((res) => handle<WeatherCity[]>(res));
}
