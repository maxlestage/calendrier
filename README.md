# Calendrier

Application de calendrier full-stack :

- **Frontend** : React 18 + TypeScript **mobile-first**, exécuté avec [Bun](https://bun.sh) (Vite comme bundler/dev server)
- **Backend** : Rust, [Actix Web](https://actix.rs) + [SeaORM](https://www.sea-ql.org/SeaORM/) (SQLite)

## Déploiement Heroku en un tap 🚀

[![Deploy to Heroku](https://www.herokucdn.com/deploy/button.svg)](https://heroku.com/deploy?template=https://github.com/maxlestage/calendrier)

Le bouton crée l'application, builde le frontend (buildpack Node.js) puis le
backend (buildpack Rust), et démarre un seul dyno : le binaire Rust sert l'API
**et** le frontend buildé. Les migrations s'exécutent automatiquement au
démarrage — rien d'autre à faire.

> ⚠️ La base est **SQLite sur le disque du dyno**, qui est éphémère chez
> Heroku. Pour compenser, le backend garde en mémoire un snapshot des
> **3 derniers mois** d'événements et le sauvegarde dans la config var
> `CALENDAR_BACKUP` quand le dyno s'arrête (SIGTERM) ; au démarrage d'un
> nouveau dyno, une base vide est re-remplie depuis cette sauvegarde.

### Activer la sauvegarde/restauration entre dynos

1. Récupère ta clé API : dashboard Heroku → avatar → **Account Settings** →
   section **API Key** → « Reveal »
2. Dans l'app → **Settings** → **Config Vars**, ajoute :
   - `HEROKU_API_KEY` = ta clé
   - `HEROKU_APP_NAME` = le nom exact de l'app (ex. `calendrier-89594ce603e6`)

Sans ces deux variables, l'app fonctionne mais les événements sont perdus à
chaque redémarrage du dyno (au moins une fois par jour). Limites : seuls les
3 derniers mois sont conservés, un crash brutal (sans SIGTERM) perd les
changements depuis le dernier arrêt propre, et la sauvegarde doit tenir dans
une config var (~32 Ko compressés, largement assez pour un agenda personnel).
`GET /api/export` renvoie à tout moment le snapshot en JSON si tu veux une
copie manuelle.

> Nécessite un compte Heroku (les dynos sont payants, il n'y a plus d'offre
> gratuite chez Heroku).

Pour des déploiements continus ensuite : dashboard Heroku → l'app → onglet
**Deploy** → « Connect to GitHub » → activer *Automatic deploys* sur `master`.

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
| `HOST` | `0.0.0.0` | Adresse d'écoute |
| `PORT` | `8080` | Port d'écoute |
| `STATIC_DIR` | `frontend/dist` | Dossier du frontend buildé (servi s'il existe, avec fallback SPA) |

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
| `GET` | `/api/export` | Snapshot JSON des 3 derniers mois (celui sauvegardé entre dynos) |
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
app.json            # Config du bouton « Deploy to Heroku » (buildpacks)
Procfile            # Commande du dyno web (binaire Rust)
Cargo.toml          # Workspace Cargo (racine, requis par le buildpack Rust)
package.json        # Script heroku-postbuild qui builde le frontend
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
