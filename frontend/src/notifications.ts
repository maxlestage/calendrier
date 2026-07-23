import type { CalendarEvent } from "./types";
import { TIDE_COLOR } from "./types";

interface NativeBridge {
  postMessage: (msg: unknown) => void;
}

/** The iOS shell injects window.webkit.messageHandlers.reminders. */
function nativeBridge(): NativeBridge | null {
  const w = window as unknown as {
    webkit?: { messageHandlers?: { reminders?: NativeBridge } };
  };
  return w.webkit?.messageHandlers?.reminders ?? null;
}

/** True inside the native iOS shell (false in Safari / desktop / PWA). */
export function hasNativeReminders(): boolean {
  return nativeBridge() !== null;
}

const LEAD_MIN = 15;
const HORIZON_DAYS = 14;

interface Reminder {
  id: string;
  title: string;
  body: string;
  /** Epoch seconds at which the notification fires */
  fireAt: number;
}

/** Reminders the native shell should schedule, derived from the events. */
export function buildReminders(events: CalendarEvent[], now = Date.now()): Reminder[] {
  const horizon = now + HORIZON_DAYS * 86400_000;
  const leadMs = LEAD_MIN * 60_000;
  return events
    .filter((ev) => !ev.all_day && ev.color !== TIDE_COLOR) // tides = 4/day, too noisy
    .map((ev) => ({ ev, start: new Date(ev.start).getTime() }))
    .filter(({ start }) => start > now && start < horizon)
    .map(({ ev, start }) => {
      const fireAt = Math.max(start - leadMs, now + 1000);
      const time = new Date(start).toLocaleTimeString("fr-FR", {
        hour: "2-digit",
        minute: "2-digit",
      });
      return {
        id: `event-${ev.id}-${ev.start}`,
        title: ev.title,
        body: ev.description ? `${time} · ${ev.description}` : `À ${time}`,
        fireAt: Math.floor(fireAt / 1000),
      };
    });
}

/**
 * Hand the native iOS shell the reminders worth firing. The web is the brain
 * (it knows a tide from an F1 session from a personal event); the native
 * layer just schedules whatever we send. No-op outside the shell.
 *
 * Rule: timed events (not all-day, not tides) in the next 14 days, 15 min
 * before they start.
 */
export function syncNativeReminders(events: CalendarEvent[]): void {
  const native = nativeBridge();
  if (!native) return;
  native.postMessage(buildReminders(events));
}
