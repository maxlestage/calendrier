import type { CalendarEvent } from "./types";

/**
 * Device-side backup: the phone (WKWebView / PWA storage) keeps a copy of
 * the server state and pushes it back when a fresh dyno boots empty — a
 * safety net that works even without the Heroku config-var backup.
 *
 * Loss detection uses a `backup_marker` setting: written server-side on
 * first load, mirrored locally. A server whose marker no longer matches the
 * local copy has lost its database.
 */

export interface ServerState {
  events: CalendarEvent[];
  settings: { key: string; value: string }[];
}

const KEY = "calendrier-device-backup";
export const MARKER_KEY = "backup_marker";

export function getSetting(state: ServerState, key: string): string | null {
  return state.settings.find((s) => s.key === key)?.value ?? null;
}

export function saveLocal(state: ServerState): void {
  try {
    localStorage.setItem(KEY, JSON.stringify(state));
  } catch {
    // Storage full or unavailable (private mode): the app still works,
    // only the device safety net is off.
  }
}

export function loadLocal(): ServerState | null {
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as ServerState;
    if (!Array.isArray(parsed.events) || !Array.isArray(parsed.settings)) return null;
    return parsed;
  } catch {
    return null;
  }
}

export function newMarker(): string {
  return typeof crypto !== "undefined" && "randomUUID" in crypto
    ? crypto.randomUUID()
    : `${Math.random().toString(36).slice(2)}-${Math.random().toString(36).slice(2)}`;
}
