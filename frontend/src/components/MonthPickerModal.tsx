import { useState } from "react";

const MONTHS_SHORT = [
  "Janv.",
  "Févr.",
  "Mars",
  "Avr.",
  "Mai",
  "Juin",
  "Juil.",
  "Août",
  "Sept.",
  "Oct.",
  "Nov.",
  "Déc.",
];

interface Props {
  year: number;
  month: number;
  onPick: (year: number, month: number) => void;
  onToday: () => void;
  onClose: () => void;
}

export default function MonthPickerModal({ year, month, onPick, onToday, onClose }: Props) {
  const [y, setY] = useState(year);

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>Choisir le mois</h2>
          <button className="icon-btn" onClick={onClose} aria-label="Fermer">
            ✕
          </button>
        </div>

        <div className="year-row">
          <button className="nav-btn" onClick={() => setY(y - 1)} aria-label="Année précédente">
            ‹
          </button>
          <span className="year-value">{y}</span>
          <button className="nav-btn" onClick={() => setY(y + 1)} aria-label="Année suivante">
            ›
          </button>
        </div>

        <div className="month-grid">
          {MONTHS_SHORT.map((label, i) => (
            <button
              key={label}
              className={`month-cell ${i === month && y === year ? "selected" : ""}`}
              onClick={() => onPick(y, i)}
            >
              {label}
            </button>
          ))}
        </div>

        <div className="modal-actions">
          <button className="btn" onClick={onToday}>
            Aujourd'hui
          </button>
          <span className="spacer" />
          <button className="btn" onClick={onClose}>
            Fermer
          </button>
        </div>
      </div>
    </div>
  );
}
