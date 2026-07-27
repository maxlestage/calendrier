import { useState } from "react";

const origin = window.location.origin;
const icsUrl = `${origin}/api/calendar.ics`;
const csvUrl = `${origin}/api/export.csv`;
const printUrl = `${origin}/api/print`;

interface Props {
  onClose: () => void;
}

export default function ShareModal({ onClose }: Props) {
  const [note, setNote] = useState<string | null>(null);

  const shareLink = async () => {
    const nav = navigator as Navigator & { share?: (d: ShareData) => Promise<void> };
    if (nav.share) {
      try {
        await nav.share({ title: "Mon calendrier", url: icsUrl });
        return;
      } catch {
        // user cancelled or unsupported: fall through to copy
      }
    }
    try {
      await navigator.clipboard.writeText(icsUrl);
      setNote("Lien copié ✓");
    } catch {
      setNote(icsUrl);
    }
  };

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>Partager / Exporter</h2>
          <button className="icon-btn" onClick={onClose} aria-label="Fermer">
            ✕
          </button>
        </div>

        <section className="tide-group share-col">
          <h3>👥 Partager le calendrier</h3>
          <p className="muted small">
            Envoie ce lien : la personne l'ajoute dans son appli Calendrier (abonnement, lecture
            seule) et voit tes événements, mis à jour automatiquement.
          </p>
          <button className="btn primary" onClick={shareLink}>
            Partager le lien d'abonnement
          </button>
          {note && <p className="muted small">{note}</p>}
        </section>

        <section className="tide-group share-col">
          <h3>⬇️ Exporter</h3>
          <a className="btn" href={icsUrl} download>
            Télécharger .ics (calendrier)
          </a>
          <a className="btn" href={csvUrl} download>
            Télécharger .csv (tableur)
          </a>
        </section>

        <section className="tide-group share-col">
          <h3>🖨️ Imprimer</h3>
          <button className="btn" onClick={() => window.open(printUrl, "_blank")}>
            Ouvrir la version imprimable
          </button>
        </section>
      </div>
    </div>
  );
}
