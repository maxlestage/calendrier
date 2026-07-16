# Calendrier

Application de calendrier full-stack :

- **Frontend** : React 18 + TypeScript, exécuté avec [Bun](https://bun.sh) (Vite comme bundler/dev server)
- **Backend** : Rust, [Actix Web](https://actix.rs) + [SeaORM](https://www.sea-ql.org/SeaORM/) (SQLite)

## Fonctionnalités

- Vue mensuelle (semaine commençant le lundi), navigation mois précédent/suivant, bouton « Aujourd'hui »
- Création d'un événement en cliquant sur un jour ou via le bouton « + Événement »
- Édition et suppression en cliquant sur un événement
- Titre, description, date, heures de début/fin, journée entière, couleur
- Persistance en SQLite via SeaORM (migrations exécutées automatiquement au démarrage)

## Démarrage

### Backend (port 8080)

```bash
cd backend
cargo run
```

Variables d'environnement optionnelles :

| Variable | Défaut | Description |
| --- | --- | --- |
| `DATABASE_URL` | `sqlite://calendar.db?mode=rwc` | URL de la base SQLite |
| `HOST` | `127.0.0.1` | Adresse d'écoute |
| `PORT` | `8080` | Port d'écoute |

### Frontend (port 5173)

```bash
cd frontend
bun install
bun run dev
```

Puis ouvrir <http://localhost:5173>. Le dev server proxifie `/api` vers le backend sur le port 8080.

## API

| Méthode | Route | Description |
| --- | --- | --- |
| `GET` | `/api/events?from=&to=` | Liste des événements (bornes ISO 8601 optionnelles) |
| `GET` | `/api/events/{id}` | Détail d'un événement |
| `POST` | `/api/events` | Création |
| `PUT` | `/api/events/{id}` | Mise à jour |
| `DELETE` | `/api/events/{id}` | Suppression |

Corps JSON pour `POST`/`PUT` :

```json
{
  "title": "Réunion équipe",
  "description": "Point hebdo",
  "start": "2026-07-16T09:00:00Z",
  "end": "2026-07-16T10:00:00Z",
  "all_day": false,
  "color": "#4f6bed"
}
```

## Structure

```
backend/
  src/
    main.rs          # Bootstrap Actix, connexion DB, migrations, routes
    handlers.rs      # Handlers CRUD /api/events
    entities/        # Entités SeaORM
    migration/       # Migrations SeaORM
frontend/
  src/
    App.tsx                    # État global, navigation mois, chargement des événements
    api.ts                     # Client REST
    dates.ts                   # Utilitaires calendrier (grille 6 semaines, formats)
    components/CalendarGrid.tsx  # Grille mensuelle
    components/EventModal.tsx    # Modale création/édition
```
