import type { CalendarEvent } from "../types";
import { FULL_DAY_NAMES, MONTH_NAMES, toTimeKey } from "../dates";

interface Props {
  day: Date;
  events: CalendarEvent[];
  onEventClick: (event: CalendarEvent) => void;
  onAdd: () => void;
}

export default function DayAgenda({ day, events, onEventClick, onAdd }: Props) {
  const sorted = [...events].sort((a, b) => a.start.localeCompare(b.start));
  return (
    <section className="agenda">
      <h2 className="agenda-title">
        {FULL_DAY_NAMES[day.getDay()]} {day.getDate()} {MONTH_NAMES[day.getMonth()].toLowerCase()}
      </h2>
      {sorted.length === 0 ? (
        <button className="agenda-empty" onClick={onAdd}>
          Aucun événement — appuyer pour en ajouter un
        </button>
      ) : (
        <ul className="agenda-list">
          {sorted.map((ev) => (
            <li key={ev.id}>
              <button className="agenda-item" onClick={() => onEventClick(ev)}>
                <span className="agenda-bar" style={{ background: ev.color ?? "#4f6bed" }} />
                <span className="agenda-time">
                  {ev.all_day ? (
                    "Journée"
                  ) : (
                    <>
                      {toTimeKey(new Date(ev.start))}
                      <br />
                      {toTimeKey(new Date(ev.end))}
                    </>
                  )}
                </span>
                <span className="agenda-body">
                  <span className="agenda-event-title">{ev.title}</span>
                  {ev.description && <span className="agenda-desc">{ev.description}</span>}
                </span>
              </button>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
