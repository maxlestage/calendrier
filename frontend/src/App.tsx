import { useCallback, useEffect, useMemo, useState } from "react";
import CalendarGrid from "./components/CalendarGrid";
import DayAgenda from "./components/DayAgenda";
import EventModal from "./components/EventModal";
import { createEvent, deleteEvent, fetchEvents, updateEvent } from "./api";
import { eventCoversDay, MONTH_NAMES, monthGridDays } from "./dates";
import type { CalendarEvent, EventPayload } from "./types";

interface ModalState {
  event: CalendarEvent | null;
  initialDate: Date;
}

export default function App() {
  const now = new Date();
  const [year, setYear] = useState(now.getFullYear());
  const [month, setMonth] = useState(now.getMonth());
  const [selectedDay, setSelectedDay] = useState<Date>(now);
  const [events, setEvents] = useState<CalendarEvent[]>([]);
  const [modal, setModal] = useState<ModalState | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Bounds of the visible grid (6 weeks), not just the month
  const [from, to] = useMemo(() => {
    const days = monthGridDays(year, month);
    const first = days[0];
    const afterLast = new Date(days[days.length - 1]);
    afterLast.setDate(afterLast.getDate() + 1);
    return [first.toISOString(), afterLast.toISOString()];
  }, [year, month]);

  const reload = useCallback(() => {
    fetchEvents(from, to)
      .then((evts) => {
        setEvents(evts);
        setError(null);
      })
      .catch((err) => setError(err instanceof Error ? err.message : "Erreur de chargement"));
  }, [from, to]);

  useEffect(reload, [reload]);

  const shiftMonth = (delta: number) => {
    const d = new Date(year, month + delta, 1);
    setYear(d.getFullYear());
    setMonth(d.getMonth());
  };

  const goToday = () => {
    const d = new Date();
    setYear(d.getFullYear());
    setMonth(d.getMonth());
    setSelectedDay(d);
  };

  const selectDay = (day: Date) => {
    setSelectedDay(day);
    // Selecting a day of the previous/next month navigates there
    if (day.getMonth() !== month || day.getFullYear() !== year) {
      setYear(day.getFullYear());
      setMonth(day.getMonth());
    }
  };

  const save = async (payload: EventPayload) => {
    if (modal?.event) {
      await updateEvent(modal.event.id, payload);
    } else {
      await createEvent(payload);
    }
    setModal(null);
    reload();
  };

  const remove = async () => {
    if (modal?.event) {
      await deleteEvent(modal.event.id);
    }
    setModal(null);
    reload();
  };

  const dayEvents = events.filter((ev) => eventCoversDay(ev.start, ev.end, selectedDay));

  return (
    <div className="app">
      <header className="toolbar">
        <button className="nav-btn" onClick={() => shiftMonth(-1)} aria-label="Mois précédent">
          ‹
        </button>
        <button className="month-label" onClick={goToday} title="Revenir à aujourd'hui">
          {MONTH_NAMES[month]} {year}
        </button>
        <button className="nav-btn" onClick={() => shiftMonth(1)} aria-label="Mois suivant">
          ›
        </button>
      </header>
      {error && <p className="error banner">⚠ {error}</p>}
      <CalendarGrid
        year={year}
        month={month}
        events={events}
        selectedDay={selectedDay}
        onSelectDay={selectDay}
      />
      <DayAgenda
        day={selectedDay}
        events={dayEvents}
        onEventClick={(event) => setModal({ event, initialDate: new Date(event.start) })}
        onAdd={() => setModal({ event: null, initialDate: selectedDay })}
      />
      <button
        className="fab"
        aria-label="Nouvel événement"
        onClick={() => setModal({ event: null, initialDate: selectedDay })}
      >
        +
      </button>
      {modal && (
        <EventModal
          event={modal.event}
          initialDate={modal.initialDate}
          onSave={save}
          onDelete={remove}
          onClose={() => setModal(null)}
        />
      )}
    </div>
  );
}
