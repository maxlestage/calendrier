import type { BeachWeather, CalendarEvent, NotifPrefs } from "./types";
import { TIDE_COLOR } from "./types";
import { toDateKey, toTimeKey } from "./dates";
import { weatherIcon } from "./weather";

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

const HORIZON_DAYS = 14;

interface Reminder {
  id: string;
  title: string;
  body: string;
  /** Epoch seconds at which the notification fires */
  fireAt: number;
}

/** Per-event reminders: timed events (not all-day, not tides) in the next 14
 * days, `leadMin` minutes before they start. */
export function buildReminders(
  events: CalendarEvent[],
  leadMin: number,
  now = Date.now(),
): Reminder[] {
  const horizon = now + HORIZON_DAYS * 86400_000;
  const leadMs = leadMin * 60_000;
  return events
    .filter((ev) => !ev.all_day && ev.color !== TIDE_COLOR)
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

interface DayContent {
  weather: string[];
  tides: Map<string, { high: boolean; t: string }[]>;
  events: { time: string | null; title: string }[];
}

/**
 * One "morning briefing" notification per day at `hour` (device-local),
 * combining the weather of the selected places, the tides of the selected
 * beaches, and the day's events — one glance instead of many pings.
 */
export function buildMorningBriefings(
  events: CalendarEvent[],
  weather: BeachWeather[],
  hour: number,
  now = Date.now(),
): Reminder[] {
  const days = new Map<string, DayContent>();
  const ensure = (k: string): DayContent => {
    let e = days.get(k);
    if (!e) {
      e = { weather: [], tides: new Map(), events: [] };
      days.set(k, e);
    }
    return e;
  };

  for (const place of weather) {
    for (const d of place.days) {
      if (d.tmax === null && d.code === null) continue;
      const emoji = weatherIcon(d.code).emoji;
      const temp =
        d.tmax !== null
          ? `${Math.round(d.tmax)}°${d.tmin !== null ? `/${Math.round(d.tmin)}°` : ""}`
          : "";
      ensure(d.date).weather.push(`${place.name} ${emoji} ${temp}`.trim());
    }
  }

  for (const ev of events) {
    const start = new Date(ev.start);
    const dayKey = toDateKey(start);
    if (ev.color === TIDE_COLOR) {
      const beach = ev.title.split(" — ")[0].replace("🌊", "").trim();
      const e = ensure(dayKey);
      const list = e.tides.get(beach) ?? [];
      list.push({ high: ev.title.includes("Pleine mer"), t: toTimeKey(start) });
      e.tides.set(beach, list);
    } else {
      ensure(dayKey).events.push({
        time: ev.all_day ? null : toTimeKey(start),
        title: ev.title,
      });
    }
  }

  const out: Reminder[] = [];
  for (const [dayKey, content] of days) {
    const [y, m, dd] = dayKey.split("-").map(Number);
    const fire = new Date(y, m - 1, dd, hour, 0).getTime();
    if (fire <= now) continue; // that morning already passed
    const lines: string[] = [];
    if (content.weather.length) lines.push(`☀️ ${content.weather.join(" · ")}`);
    for (const [beach, list] of content.tides) {
      const fmt = (high: boolean) =>
        list
          .filter((x) => x.high === high)
          .map((x) => x.t)
          .join(" ");
      lines.push(`🌊 ${beach} — PM ${fmt(true) || "—"} · BM ${fmt(false) || "—"}`);
    }
    if (content.events.length) {
      const evStr = content.events
        .sort((a, b) => (a.time ?? "").localeCompare(b.time ?? ""))
        .map((e) => (e.time ? `${e.time} ${e.title}` : e.title))
        .join(" · ");
      lines.push(`📅 ${evStr}`);
    }
    if (!lines.length) continue;
    out.push({
      id: `briefing-${dayKey}`,
      title: "☀️ Ta journée",
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
 */
export function syncNativeReminders(
  events: CalendarEvent[],
  weather: BeachWeather[],
  prefs: NotifPrefs,
): void {
  const native = nativeBridge();
  if (!native) return;
  const items: Reminder[] = [];
  if (prefs.event_reminders) items.push(...buildReminders(events, prefs.lead_min));
  if (prefs.morning_briefing)
    items.push(...buildMorningBriefings(events, weather, prefs.morning_hour));
  native.postMessage(items);
}
