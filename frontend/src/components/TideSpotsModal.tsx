import { useEffect, useState } from "react";
import { fetchTideSpots, saveTideSpots } from "../api";
import type { TideSpot } from "../types";

interface Props {
  onSaved: () => void;
  onClose: () => void;
}

const GROUPS: { id: TideSpot["group"]; label: string; hint?: string }[] = [
  { id: "ocean", label: "🌊 Plages de l'océan (Atlantique)" },
  {
    id: "mer",
    label: "🏖️ Plages de la mer (Méditerranée)",
    hint: "Marée faible (~20-40 cm) : le vent et la pression comptent souvent plus.",
  },
  { id: "manche", label: "⚓ Manche" },
  { id: "ports", label: "🧭 Ports de référence" },
];

export default function TideSpotsModal({ onSaved, onClose }: Props) {
  const [spots, setSpots] = useState<TideSpot[] | null>(null);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    fetchTideSpots()
      .then((list) => {
        setSpots(list);
        setSelected(new Set(list.filter((s) => s.selected).map((s) => s.key)));
      })
      .catch((err) => setError(err instanceof Error ? err.message : "Erreur de chargement"));
  }, []);

  const add = (key: string) => {
    if (!key) return;
    setSelected((prev) => new Set(prev).add(key));
  };

  const remove = (key: string) => {
    setSelected((prev) => {
      const next = new Set(prev);
      next.delete(key);
      return next;
    });
  };

  const save = async () => {
    setBusy(true);
    setError(null);
    try {
      await saveTideSpots([...selected]);
      onSaved();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Erreur inconnue");
      setBusy(false);
    }
  };

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>Marées — choisir les plages</h2>
          <button className="icon-btn" onClick={onClose} aria-label="Fermer">
            ✕
          </button>
        </div>
        {!spots && !error && <p className="muted">Chargement…</p>}
        {spots &&
          GROUPS.map((group) => {
            const groupSpots = spots.filter((s) => s.group === group.id);
            if (groupSpots.length === 0) return null;
            const chosen = groupSpots.filter((s) => selected.has(s.key));
            const available = groupSpots.filter((s) => !selected.has(s.key));
            return (
              <section key={group.id} className="tide-group">
                <h3>{group.label}</h3>
                {group.hint && <p className="muted small">{group.hint}</p>}
                {chosen.length > 0 && (
                  <div className="chips">
                    {chosen.map((s) => (
                      <span key={s.key} className="chip">
                        {s.name}
                        <button
                          className="chip-x"
                          onClick={() => remove(s.key)}
                          aria-label={`Retirer ${s.name}`}
                        >
                          ✕
                        </button>
                      </span>
                    ))}
                  </div>
                )}
                {available.length > 0 && (
                  <select
                    className="tide-select"
                    value=""
                    onChange={(e) => add(e.target.value)}
                    disabled={busy}
                  >
                    <option value="">Ajouter une plage…</option>
                    {available.map((s) => (
                      <option key={s.key} value={s.key}>
                        {s.name}
                      </option>
                    ))}
                  </select>
                )}
              </section>
            );
          })}
        {error && <p className="error">{error}</p>}
        <div className="modal-actions">
          <span className="muted small">
            {selected.size} plage{selected.size > 1 ? "s" : ""} · 4 marées/jour/plage
          </span>
          <span className="spacer" />
          <button className="btn" onClick={onClose} disabled={busy}>
            Annuler
          </button>
          <button className="btn primary" onClick={save} disabled={busy || !spots}>
            {busy ? "Enregistrement…" : "Enregistrer"}
          </button>
        </div>
      </div>
    </div>
  );
}
