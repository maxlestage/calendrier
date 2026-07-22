import type { CalendarEvent } from "../types";
import { TIDE_COLOR } from "../types";
import { DAY_NAMES, eventCoversDay, isSameDay, monthGridDays, toTimeKey } from "../dates";

interface Props {
  year: number;
  month: number;
  events: CalendarEvent[];
  selectedDay: Date;
  onSelectDay: (day: Date) => void;
}

export default function CalendarGrid({ year, month, events, selectedDay, onSelectDay }: Props) {
  const days = monthGridDays(year, month);
  const today = new Date();

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
          const isSelected = isSameDay(day, selectedDay);
          const dayEvents = events.filter((ev) => eventCoversDay(ev.start, ev.end, day));
          // Tides get first-class visibility: high-tide times printed in the
          // cell (sea blue); the other categories keep their color dots.
          const tideHighs = [
            ...new Set(
              dayEvents
                .filter((ev) => ev.color === TIDE_COLOR && ev.title.includes("Pleine mer"))
                .map((ev) => toTimeKey(new Date(ev.start)))
            ),
          ]
            .sort()
            .slice(0, 2);
          const dots = dayEvents.filter((ev) => ev.color !== TIDE_COLOR).slice(0, 4);
          return (
            <button
              key={day.toISOString()}
              className={`cell ${inMonth ? "" : "outside"} ${isSelected ? "selected" : ""}`}
              onClick={() => onSelectDay(day)}
            >
              <span className={`day-number ${isToday ? "today" : ""}`}>{day.getDate()}</span>
              <span className="dots">
                {dots.map((ev) => (
                  <span key={ev.id} className="dot" style={{ background: ev.color ?? "#4f6bed" }} />
                ))}
              </span>
              {tideHighs.length > 0 && (
                <span className="cell-tides">
                  {tideHighs.map((t) => (
                    <span key={t}>▲{t}</span>
                  ))}
                </span>
              )}
            </button>
          );
        })}
      </div>
    </div>
  );
}
