import type { CalendarEvent } from "../types";
import { DAY_NAMES, isSameDay, monthGridDays, toDateKey, toTimeKey } from "../dates";

interface Props {
  year: number;
  month: number;
  events: CalendarEvent[];
  onDayClick: (day: Date) => void;
  onEventClick: (event: CalendarEvent) => void;
}

export default function CalendarGrid({ year, month, events, onDayClick, onEventClick }: Props) {
  const days = monthGridDays(year, month);
  const today = new Date();

  const eventsByDay = new Map<string, CalendarEvent[]>();
  for (const ev of events) {
    // An event can span several days: register it on each day it covers.
    const start = new Date(ev.start);
    const end = new Date(ev.end);
    const cursor = new Date(start.getFullYear(), start.getMonth(), start.getDate());
    while (cursor <= end) {
      const key = toDateKey(cursor);
      const list = eventsByDay.get(key) ?? [];
      list.push(ev);
      eventsByDay.set(key, list);
      cursor.setDate(cursor.getDate() + 1);
    }
  }

  return (
    <div className="calendar">
      <div className="weekdays">
        {DAY_NAMES.map((d) => (
          <div key={d} className="weekday">
            {d}
          </div>
        ))}
      </div>
      <div className="grid">
        {days.map((day) => {
          const inMonth = day.getMonth() === month;
          const isToday = isSameDay(day, today);
          const dayEvents = eventsByDay.get(toDateKey(day)) ?? [];
          const visible = dayEvents.slice(0, 3);
          const hidden = dayEvents.length - visible.length;
          return (
            <div
              key={day.toISOString()}
              className={`cell ${inMonth ? "" : "outside"}`}
              onClick={() => onDayClick(day)}
            >
              <span className={`day-number ${isToday ? "today" : ""}`}>{day.getDate()}</span>
              <div className="events">
                {visible.map((ev) => (
                  <button
                    key={ev.id}
                    className="event-chip"
                    style={{ background: ev.color ?? "#4f6bed" }}
                    title={ev.title}
                    onClick={(e) => {
                      e.stopPropagation();
                      onEventClick(ev);
                    }}
                  >
                    {!ev.all_day && <span className="event-time">{toTimeKey(new Date(ev.start))}</span>}
                    {ev.title}
                  </button>
                ))}
                {hidden > 0 && <span className="more">+{hidden} autres</span>}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
