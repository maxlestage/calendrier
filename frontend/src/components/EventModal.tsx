import { useState } from "react";
import type { CalendarEvent, EventPayload, Recurrence } from "../types";
import { EVENT_COLORS } from "../types";
import { toDateKey, toTimeKey } from "../dates";

interface Props {
  /** Existing event when editing, null when creating */
  event: CalendarEvent | null;
  /** Pre-selected day when creating */
  initialDate: Date;
  onSave: (payload: EventPayload) => Promise<void>;
  onDelete: () => Promise<void>;
  onClose: () => void;
}

export default function EventModal({ event, initialDate, onSave, onDelete, onClose }: Props) {
  const startDate = event ? new Date(event.start) : initialDate;
  const endDate = event ? new Date(event.end) : initialDate;

  const [title, setTitle] = useState(event?.title ?? "");
  const [description, setDescription] = useState(event?.description ?? "");
  const [date, setDate] = useState(toDateKey(startDate));
  const [startTime, setStartTime] = useState(event && !event.all_day ? toTimeKey(startDate) : "09:00");
  const [endTime, setEndTime] = useState(event && !event.all_day ? toTimeKey(endDate) : "10:00");
  const [allDay, setAllDay] = useState(event?.all_day ?? false);
  const [color, setColor] = useState(event?.color ?? EVENT_COLORS[0]);
  const [recurrence, setRecurrence] = useState<Recurrence>(event?.recurrence ?? null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!title.trim()) {
      setError("Le titre est obligatoire.");
      return;
    }
    const start = allDay ? new Date(`${date}T00:00`) : new Date(`${date}T${startTime}`);
    const end = allDay ? new Date(`${date}T23:59`) : new Date(`${date}T${endTime}`);
    if (end < start) {
      setError("La fin doit être après le début.");
      return;
    }
    setBusy(true);
    setError(null);
    try {
      await onSave({
        title: title.trim(),
        description: description.trim() || null,
        start: start.toISOString(),
        end: end.toISOString(),
        all_day: allDay,
        color,
        recurrence,
      });
    } catch (err) {
      setError(err instanceof Error ? err.message : "Erreur inconnue");
      setBusy(false);
    }
  };

  const remove = async () => {
    setBusy(true);
    setError(null);
    try {
      await onDelete();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Erreur inconnue");
      setBusy(false);
    }
  };

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>{event ? "Modifier l'événement" : "Nouvel événement"}</h2>
          <button className="icon-btn" onClick={onClose} aria-label="Fermer">
            ✕
          </button>
        </div>
        <form onSubmit={submit}>
          <label>
            Titre
            <input
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="Réunion, anniversaire…"
              autoFocus
            />
          </label>
          <label>
            Description
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              rows={2}
              placeholder="Détails (optionnel)"
            />
          </label>
          <div className="row">
            <label>
              Date
              <input type="date" value={date} onChange={(e) => setDate(e.target.value)} />
            </label>
            <label className="checkbox">
              <input
                type="checkbox"
                checked={allDay}
                onChange={(e) => setAllDay(e.target.checked)}
              />
              Journée entière
            </label>
          </div>
          {!allDay && (
            <div className="row">
              <label>
                Début
                <input
                  type="time"
                  value={startTime}
                  onChange={(e) => setStartTime(e.target.value)}
                />
              </label>
              <label>
                Fin
                <input type="time" value={endTime} onChange={(e) => setEndTime(e.target.value)} />
              </label>
            </div>
          )}
          <label>
            Répétition
            <select
              className="tide-select"
              value={recurrence ?? ""}
              onChange={(e) => setRecurrence((e.target.value || null) as Recurrence)}
            >
              <option value="">Jamais</option>
              <option value="weekly">Chaque semaine</option>
              <option value="monthly">Chaque mois</option>
              <option value="yearly">Chaque année (anniversaires…)</option>
            </select>
          </label>
          {recurrence && event && (
            <p className="muted small">
              Modifier ou supprimer agit sur toute la série.
            </p>
          )}
          <div className="color-picker">
            {EVENT_COLORS.map((c) => (
              <button
                key={c}
                type="button"
                className={`color-dot ${color === c ? "selected" : ""}`}
                style={{ background: c }}
                onClick={() => setColor(c)}
                aria-label={`Couleur ${c}`}
              />
            ))}
          </div>
          {error && <p className="error">{error}</p>}
          <div className="modal-actions">
            {event && (
              <button type="button" className="btn danger" onClick={remove} disabled={busy}>
                Supprimer
              </button>
            )}
            <span className="spacer" />
            <button type="button" className="btn" onClick={onClose} disabled={busy}>
              Annuler
            </button>
            <button type="submit" className="btn primary" disabled={busy}>
              {event ? "Enregistrer" : "Créer"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
