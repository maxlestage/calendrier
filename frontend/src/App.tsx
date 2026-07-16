import { useCallback, useEffect, useMemo, useState } from "react";
import CalendarGrid from "./components/CalendarGrid";
import EventModal from "./components/EventModal";
import { createEvent, deleteEvent, fetchEvents, updateEvent } from "./api";
import { MONTH_NAMES, monthGridDays } from "./dates";
import type { CalendarEvent, EventPayload } from "./types";

interface ModalState {
  event: CalendarEvent | null;
  initialDate: Date;
}

export default function App() {
  const now = new Date();
  const [year, setYear] = useState(now.getFullYear());
  const [month, setMonth] = useState(now.getMonth());
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

  return (
    <div className="app">
      <header className="toolbar">
        <h1>📅 Calendrier</h1>
        <div className="nav">
          <button className="btn" onClick={() => shiftMonth(-1)} aria-label="Mois précédent">
            ‹
          </button>
          <span className="month-label">
            {MONTH_NAMES[month]} {year}
          </span>
          <button className="btn" onClick={() => shiftMonth(1)} aria-label="Mois suivant">
            ›
          </button>
          <button className="btn" onClick={goToday}>
            Aujourd'hui
          </button>
        </div>
        <button
          className="btn primary"
          onClick={() => setModal({ event: null, initialDate: new Date() })}
        >
          + Événement
        </button>
      </header>
      {error && <p className="error banner">⚠ {error} — le backend est-il démarré ?</p>}
      <CalendarGrid
        year={year}
        month={month}
        events={events}
        onDayClick={(day) => setModal({ event: null, initialDate: day })}
        onEventClick={(event) => setModal({ event, initialDate: new Date(event.start) })}
      />
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
