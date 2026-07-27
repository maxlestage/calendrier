import { useState } from "react";
import { createEventsBatch, parseSchedule } from "../api";
import { FULL_DAY_NAMES, MONTH_NAMES } from "../dates";
import { EVENT_COLORS } from "../types";
import type { DraftEvent, EventPayload } from "../types";

interface Props {
  onImported: () => void;
  onClose: () => void;
}

interface Row extends DraftEvent {
  keep: boolean;
}

/** Human-readable "Lundi 7 sept. · 09:00–10:30" for a draft. */
function label(d: DraftEvent): string {
  const s = new Date(d.start);
  const day = `${FULL_DAY_NAMES[s.getDay()]} ${s.getDate()} ${MONTH_NAMES[s.getMonth()].slice(0, 4).toLowerCase()}`;
  if (d.all_day) return `${day} · toute la journée`;
  const e = new Date(d.end);
  const hm = (x: Date) =>
    `${String(x.getHours()).padStart(2, "0")}:${String(x.getMinutes()).padStart(2, "0")}`;
  return `${day} · ${hm(s)}–${hm(e)}`;
}

export default function ImportModal({ onImported, onClose }: Props) {
  const [text, setText] = useState("");
  const [rows, setRows] = useState<Row[] | null>(null);
  const [busy, setBusy] = useState(false);
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const runOcr = async (file: File) => {
    setBusy(true);
    setError(null);
    setStatus("Lecture de l'image… (le premier scan télécharge le moteur, patiente un peu)");
    try {
      // Loaded on demand so the OCR engine never weighs on the normal app start.
      const { createWorker } = await import("tesseract.js");
      const worker = await createWorker("fra");
      const {
        data: { text: ocr },
      } = await worker.recognize(file);
      await worker.terminate();
      setText((prev) => (prev ? `${prev}\n${ocr}` : ocr));
      setStatus("Image lue ✓ — vérifie le texte puis analyse-le.");
    } catch {
      setError("Lecture de l'image impossible. Tu peux coller le texte à la main.");
      setStatus(null);
    } finally {
      setBusy(false);
    }
  };

  const analyse = async () => {
    setBusy(true);
    setError(null);
    setStatus(null);
    try {
      const drafts = await parseSchedule(text);
      if (drafts.length === 0) {
        setError("Aucun événement détecté. Ajoute des jours et des horaires (ex : « Lundi 9h-10h Maths »).");
        setRows(null);
      } else {
        setRows(drafts.map((d) => ({ ...d, keep: true })));
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : "Analyse impossible");
    } finally {
      setBusy(false);
    }
  };

  const importKept = async () => {
    if (!rows) return;
    const kept = rows.filter((r) => r.keep);
    if (kept.length === 0) {
      onClose();
      return;
    }
    setBusy(true);
    setError(null);
    try {
      const payloads: EventPayload[] = kept.map((r) => ({
        title: r.title.trim() || "Sans titre",
        description: null,
        start: r.start,
        end: r.end,
        all_day: r.all_day,
        color: EVENT_COLORS[4], // violet: marks imported events
        recurrence: null,
      }));
      await createEventsBatch(payloads);
      onImported();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Import impossible");
      setBusy(false);
    }
  };

  const keptCount = rows?.filter((r) => r.keep).length ?? 0;

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>Scanner un emploi du temps</h2>
          <button className="icon-btn" onClick={onClose} aria-label="Fermer">
            ✕
          </button>
        </div>

        {!rows && (
          <>
            <p className="muted small">
              Prends en photo n'importe quel planning (papier, écran, capture) ou colle son texte :
              l'app en extrait les événements, tu vérifies, et tout est copié dans ton calendrier.
            </p>

            <label className="btn primary import-photo">
              📷 Prendre une photo / choisir une image
              <input
                type="file"
                accept="image/*"
                capture="environment"
                hidden
                disabled={busy}
                onChange={(e) => {
                  const f = e.target.files?.[0];
                  if (f) runOcr(f);
                  e.target.value = "";
                }}
              />
            </label>

            <textarea
              className="import-text"
              placeholder={"Ou colle/écris ici, par exemple :\nLundi 9h-10h30 Mathématiques\nMardi 14h Sport\n12/09 9h Rendez-vous médecin"}
              value={text}
              onChange={(e) => setText(e.target.value)}
              rows={7}
            />

            {status && <p className="muted small">{status}</p>}
            {error && <p className="error small">⚠ {error}</p>}

            <button className="btn primary" onClick={analyse} disabled={busy || text.trim().length === 0}>
              {busy ? "…" : "Analyser le texte"}
            </button>
          </>
        )}

        {rows && (
          <>
            <p className="muted small">
              {rows.length} événement{rows.length > 1 ? "s" : ""} détecté{rows.length > 1 ? "s" : ""}.
              Décoche ceux à ignorer et corrige les titres si besoin.
            </p>
            <div className="import-list">
              {rows.map((r, i) => (
                <div key={i} className={`import-row${r.keep ? "" : " off"}`}>
                  <input
                    type="checkbox"
                    checked={r.keep}
                    onChange={(e) =>
                      setRows((rs) => rs!.map((x, j) => (j === i ? { ...x, keep: e.target.checked } : x)))
                    }
                    aria-label="Inclure cet événement"
                  />
                  <div className="import-row-body">
                    <input
                      className="import-title"
                      value={r.title}
                      onChange={(e) =>
                        setRows((rs) => rs!.map((x, j) => (j === i ? { ...x, title: e.target.value } : x)))
                      }
                    />
                    <span className="muted small">{label(r)}</span>
                  </div>
                </div>
              ))}
            </div>
            {error && <p className="error small">⚠ {error}</p>}
            <div className="import-actions">
              <button className="btn" onClick={() => setRows(null)} disabled={busy}>
                ‹ Retour
              </button>
              <button className="btn primary" onClick={importKept} disabled={busy || keptCount === 0}>
                {busy ? "Import…" : `Importer ${keptCount} événement${keptCount > 1 ? "s" : ""}`}
              </button>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
