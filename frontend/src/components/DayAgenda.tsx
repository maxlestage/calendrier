import type { BeachWeather, CalendarEvent } from "../types";
import { FULL_DAY_NAMES, MONTH_NAMES, toDateKey, toTimeKey } from "../dates";
import { formatNumber, weatherIcon } from "../weather";

interface Props {
  day: Date;
  events: CalendarEvent[];
  weather: BeachWeather[];
  onEventClick: (event: CalendarEvent) => void;
  onAdd: () => void;
}

export default function DayAgenda({ day, events, weather, onEventClick, onAdd }: Props) {
  const sorted = [...events].sort((a, b) => a.start.localeCompare(b.start));
  const dateKey = toDateKey(day);
  // One weather card per selected beach, when the forecast covers this day
  const cards = weather.flatMap((spot) => {
    const forecast = spot.days.find((d) => d.date === dateKey);
    return forecast ? [{ spot, forecast }] : [];
  });
  return (
    <section className="agenda">
      <h2 className="agenda-title">
        {FULL_DAY_NAMES[day.getDay()]} {day.getDate()} {MONTH_NAMES[day.getMonth()].toLowerCase()}
      </h2>
      {cards.length > 0 && (
        <ul className="beach-weather">
          {cards.map(({ spot, forecast }) => {
            const icon = weatherIcon(forecast.code);
            const details = [
              forecast.wind !== null ? `💨 ${Math.round(forecast.wind)} km/h` : null,
              forecast.uv !== null ? `UV ${formatNumber(forecast.uv)}` : null,
              forecast.precip !== null ? `☔ ${Math.round(forecast.precip)} %` : null,
              forecast.wave !== null ? `🌊 ${formatNumber(forecast.wave)} m` : null,
              forecast.water !== null ? `💧 eau ${formatNumber(forecast.water)}°` : null,
            ].filter(Boolean);
            return (
              <li key={spot.key} className="beach-weather-card" title={icon.label}>
                <span className="beach-weather-emoji">{icon.emoji}</span>
                <span className="beach-weather-body">
                  <span className="beach-weather-head">
                    <span className="beach-weather-name">🏖️ {spot.name}</span>
                    <span className="beach-weather-temp">
                      {forecast.tmax !== null ? `${Math.round(forecast.tmax)}°` : "—"}
                      {forecast.tmin !== null && (
                        <span className="beach-weather-tmin"> / {Math.round(forecast.tmin)}°</span>
                      )}
                    </span>
                  </span>
                  <span className="beach-weather-details">{details.join(" · ")}</span>
                </span>
              </li>
            );
          })}
        </ul>
      )}
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
