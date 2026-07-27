import { useState } from "react";

const origin = window.location.origin;
const appUrl = origin;
const icsUrl = `${origin}/api/calendar.ics`;
const csvUrl = `${origin}/api/export.csv`;
const printUrl = `${origin}/api/print`;

interface Props {
  onClose: () => void;
}

export default function ShareModal({ onClose }: Props) {
  const [note, setNote] = useState<string | null>(null);

  const share = async (title: string, url: string) => {
    const nav = navigator as Navigator & { share?: (d: ShareData) => Promise<void> };
    if (nav.share) {
      try {
        await nav.share({ title, url });
        return;
      } catch {
        // cancelled/unsupported → copy
      }
    }
    try {
      await navigator.clipboard.writeText(url);
      setNote("Lien copié ✓");
    } catch {
      setNote(url);
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
          <h3>👩‍❤️‍👨 Modifier ensemble (conjoint)</h3>
          <p className="muted small">
            Envoie ce lien à ton conjoint : il ouvre la <b>même app</b> et vous partagez le même
            calendrier — chacun peut ajouter et modifier, tout apparaît chez les deux.
          </p>
          <button className="btn primary" onClick={() => share("Notre calendrier", appUrl)}>
            Partager l'accès (modifier)
          </button>
          <p className="muted small">
            ⚠️ Ce lien donne un accès complet (ajout, modification, suppression). Ne le partage
            qu'à des personnes de confiance.
          </p>
        </section>

        <section className="tide-group share-col">
          <h3>👀 Voir seulement (amis)</h3>
          <p className="muted small">
            Lien d'abonnement : la personne l'ajoute dans son appli Calendrier et voit tes
            événements en lecture seule (mis à jour automatiquement).
          </p>
          <button className="btn" onClick={() => share("Mon calendrier", icsUrl)}>
            Partager le lien d'abonnement
          </button>
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

        {note && <p className="muted small">{note}</p>}
      </div>
    </div>
  );
}
