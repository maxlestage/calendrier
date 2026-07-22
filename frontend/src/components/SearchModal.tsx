import { useEffect, useRef, useState } from "react";
import { searchEvents } from "../api";
import type { CalendarEvent } from "../types";
import { MONTH_NAMES, toTimeKey } from "../dates";

interface Props {
  onPick: (day: Date) => void;
  onClose: () => void;
}

export default function SearchModal({ onPick, onClose }: Props) {
  const [q, setQ] = useState("");
  const [results, setResults] = useState<CalendarEvent[]>([]);
  const [searching, setSearching] = useState(false);
  const timer = useRef<number>();

  useEffect(() => {
    window.clearTimeout(timer.current);
    const query = q.trim();
    if (query.length < 2) {
      setResults([]);
      return;
    }
    setSearching(true);
    timer.current = window.setTimeout(() => {
      searchEvents(query)
        .then((evts) => setResults(evts.slice(0, 50)))
        .catch(() => setResults([]))
        .finally(() => setSearching(false));
    }, 250);
    return () => window.clearTimeout(timer.current);
  }, [q]);

  const fmt = (ev: CalendarEvent) => {
    const d = new Date(ev.start);
    const date = `${d.getDate()} ${MONTH_NAMES[d.getMonth()].toLowerCase()} ${d.getFullYear()}`;
    return ev.all_day ? date : `${date} · ${toTimeKey(d)}`;
  };

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>Rechercher</h2>
          <button className="icon-btn" onClick={onClose} aria-label="Fermer">
            ✕
          </button>
        </div>
        <input
          type="search"
          className="search-input"
          value={q}
          onChange={(e) => setQ(e.target.value)}
          placeholder="Titre d'un événement… (Monza, marée, vacances)"
          autoFocus
        />
        {q.trim().length >= 2 && !searching && results.length === 0 && (
          <p className="muted small">Aucun résultat.</p>
        )}
        <ul className="search-results">
          {results.map((ev) => (
            <li key={`${ev.id}-${ev.start}`}>
              <button
                className="agenda-item"
                onClick={() => onPick(new Date(ev.start))}
              >
                <span className="agenda-bar" style={{ background: ev.color ?? "#4f6bed" }} />
                <span className="agenda-body">
                  <span className="agenda-event-title">{ev.title}</span>
                  <span className="agenda-desc">{fmt(ev)}</span>
                </span>
              </button>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}
