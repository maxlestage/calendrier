import type { CalendarEvent } from "./types";
import { TIDE_COLOR } from "./types";
import { toDateKey, toTimeKey } from "./dates";

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
/** Hour (device-local) of the daily tide-summary notification */
const TIDE_SUMMARY_HOUR = 7;

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
 * One notification per day (at TIDE_SUMMARY_HOUR, device-local) summarising
 * every selected beach's high/low tides — instead of 4 spammy alerts a day.
 */
export function buildTideSummaries(events: CalendarEvent[], now = Date.now()): Reminder[] {
  const tides = events.filter((ev) => ev.color === TIDE_COLOR);
  if (tides.length === 0) return [];

  // day (device-local) → beach → tide lines
  const byDay = new Map<string, Map<string, { high: boolean; t: string }[]>>();
  for (const ev of tides) {
    const d = new Date(ev.start);
    const dayKey = toDateKey(d);
    const beach = ev.title.split(" — ")[0].replace("🌊", "").trim();
    const high = ev.title.includes("Pleine mer");
    const beaches = byDay.get(dayKey) ?? new Map();
    const list = beaches.get(beach) ?? [];
    list.push({ high, t: toTimeKey(d) });
    beaches.set(beach, list);
    byDay.set(dayKey, beaches);
  }

  const out: Reminder[] = [];
  for (const [dayKey, beaches] of byDay) {
    const [y, m, dd] = dayKey.split("-").map(Number);
    const fire = new Date(y, m - 1, dd, TIDE_SUMMARY_HOUR, 0).getTime();
    if (fire <= now) continue; // this morning already passed
    const lines = [...beaches].map(([beach, list]) => {
      const fmt = (high: boolean) =>
        list
          .filter((x) => x.high === high)
          .map((x) => x.t)
          .join(" ");
      const highs = fmt(true);
      const lows = fmt(false);
      return `${beach} — PM ${highs || "—"} · BM ${lows || "—"}`;
    });
    out.push({
      id: `tides-${dayKey}`,
      title: "🌊 Marées du jour",
      body: lines.join("\n"),
      fireAt: Math.floor(fire / 1000),
    });
  }
  return out;
}

/**
 * Hand the native iOS shell the reminders worth firing. The web is the brain
 * (it knows a tide from an F1 session from a personal event); the native
 * layer just schedules whatever we send. No-op outside the shell.
 *
 * - timed events (not all-day, not tides) in the next 14 days, 15 min before;
 * - one daily tide summary per morning for the selected beaches.
 */
export function syncNativeReminders(events: CalendarEvent[]): void {
  const native = nativeBridge();
  if (!native) return;
  native.postMessage([...buildReminders(events), ...buildTideSummaries(events)]);
}
