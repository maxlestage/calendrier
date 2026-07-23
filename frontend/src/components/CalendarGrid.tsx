import type { BeachWeather, CalendarEvent } from "../types";
import { TIDE_COLOR } from "../types";
import { DAY_NAMES, eventCoversDay, isSameDay, monthGridDays, toDateKey, toTimeKey } from "../dates";
import { weatherIcon } from "../weather";

interface Props {
  year: number;
  month: number;
  events: CalendarEvent[];
  weather: BeachWeather[];
  selectedDay: Date;
  onSelectDay: (day: Date) => void;
  /** When true, only the selected day's week is shown (agenda gets the room) */
  collapsed: boolean;
  onToggleCollapse: () => void;
}

export default function CalendarGrid({
  year,
  month,
  events,
  weather,
  selectedDay,
  onSelectDay,
  collapsed,
  onToggleCollapse,
}: Props) {
  const allDays = monthGridDays(year, month);
  // Collapsed: show just the week (7 days) containing the selected day.
  const selIndex = allDays.findIndex((d) => isSameDay(d, selectedDay));
  const weekStart = selIndex >= 0 ? Math.floor(selIndex / 7) * 7 : 0;
  const days = collapsed ? allDays.slice(weekStart, weekStart + 7) : allDays;
  const today = new Date();
  // Weather emoji per day cell, from the first selected place
  const wxByDate = new Map<string, string>();
  for (const d of weather[0]?.days ?? []) {
    if (d.code !== null) wxByDate.set(d.date, weatherIcon(d.code).emoji);
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
              {wxByDate.has(toDateKey(day)) && (
                <span className="cell-wx">{wxByDate.get(toDateKey(day))}</span>
              )}
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
      <button
        className="cal-toggle"
        onClick={onToggleCollapse}
        aria-label={collapsed ? "Agrandir le calendrier" : "Réduire le calendrier"}
      >
        {collapsed ? "▾ Afficher le mois" : "▴ Réduire le calendrier"}
      </button>
    </div>
  );
}
