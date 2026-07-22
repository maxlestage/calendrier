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

- **Événements thématiques générés à chaque démarrage**, pour l'année en cours **et** la suivante :
  - 🏎️ **F1** (rouge) : calendrier récupéré depuis l'[API Jolpica](https://api.jolpi.ca) (successeur d'Ergast, sans clé) à chaque démarrage, avec **heures exactes des courses, qualifications, sprints et qualifs sprint** ; repli sur la saison 2026 embarquée (journée entière) si hors-ligne
  - 🔭 **Astronomie** (bleu nuit) : **éclipses solaires et lunaires calculées** (Meeus ch. 54, types total/annulaire/partiel validés contre le catalogue NASA), équinoxes et solstices à la minute (Meeus ch. 27), pics des essaims de météores (Perséides, Géminides…)
  - ✨ **Astrologie** (turquoise) : débuts des saisons zodiacales + **pleines et nouvelles lunes calculées** à l'instant exact (Meeus ch. 49, validé contre les éphémérides)
  - 🎆 **Feux d'artifice** (orange) : 14 Juillet, 15 août, Saint-Sylvestre — horaires de soirée (~22h30)
  - 🎬 **Sorties cinéma** France (violet) : liste 2026 embarquée ; ajoutez une config var `TMDB_API_KEY` (clé gratuite sur themoviedb.org) pour récupérer automatiquement les sorties à venir les plus populaires
  - 🌊 **Marées** (bleu mer) : pleines et basses mers avec heures et hauteurs, via l'[API WorldTides](https://www.worldtides.info) (prédictions officielles SHOM/FES). **L'utilisateur choisit ses plages dans l'app** : bouton 🌊 → listes déroulantes par côte, avec **toutes les plages** au catalogue (~26 plages de l'océan Atlantique de Carnac à Hendaye, ~29 plages de la Méditerranée d'Argelès à Porto-Vecchio, + Manche et ports de référence). Le choix est **persisté en base** (`GET/PUT /api/tide-spots`) : sélectionner une plage récupère ses marées immédiatement, la retirer supprime ses événements. Marnage méditerranéen faible (~20-40 cm) : le vent/la pression comptent souvent plus que la marée. **Aucune clé requise** : par défaut les marées sont déduites de la courbe de hauteur d'eau horaire d'[Open-Meteo](https://open-meteo.com) (modèle de marée hydrodynamique, gratuit, sans clé) — extremums affinés par interpolation, précision de l'ordre de quelques minutes, source indiquée dans la description de l'événement. Si la config var `WORLDTIDES_API_KEY` est présente, l'app utilise à la place les extremums officiels WorldTides (SHOM/FES). Un modèle « maison » (ajustement harmonique) a été écarté à dessein : des horaires de marée faux sont dangereux (baïnes, pêche à pied, estran) — les deux sources s'appuient sur de vrais modèles de marée. `TIDE_PORTS` (spots ou groupes `ocean`/`mer`/`manche`/`ports`) reste un repli si aucun choix n'a été fait dans l'app ; `TIDE_DAYS` : horizon en jours (défaut 14) ; l'appel API n'est fait que lorsque l'horizon stocké d'un spot devient court
  - Dédup au démarrage par jour civil de Paris (pas de doublons entre redémarrages) ; les dates plus vieilles que 3 mois ne sont pas réinsérées ; un événement pré-chargé supprimé peut réapparaître au redémarrage suivant. `SEED_DISABLED=1` désactive tout

- 🏖️ **Météo des plages** : pour les plages sélectionnées (les mêmes que les marées), une carte météo par plage s'affiche en tête de l'agenda du jour — temps (soleil/nuages/pluie/orage), températures max/min, vent, indice UV, probabilité de pluie, **hauteur de vagues et température de l'eau** (API marine). Prévisions à 7 jours via [Open-Meteo](https://open-meteo.com) (**gratuit, sans clé — rien à configurer**), servies en direct par `GET /api/beach-weather` avec un cache mémoire de 30 min ; la météo n'est pas stockée en événements car elle change en permanence
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
| `GET` | `/api/tide-spots` | Catalogue des plages + sélection courante |
| `PUT` | `/api/tide-spots` | Enregistre la sélection de plages (`{"spots": [...]}`) |
| `GET` | `/api/beach-weather` | Météo 7 jours des plages sélectionnées (Open-Meteo, cache 30 min) |

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

## App iOS (webview)

L'app iOS ([`ios/`](ios/README.md)) est une **coquille WKWebView** qui charge
l'app web : une seule cible, aucune capacité spéciale, signature automatique
sans étape manuelle — et chaque déploiement Heroku met l'app à jour sans
repasser par TestFlight. Pull-to-refresh, écran de secours pour changer
l'URL du serveur.

Le front web est aussi une **PWA** : Safari → Partager → « Sur l'écran
d'accueil » pour l'installer en plein écran avec son icône.

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
