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
> Heroku. Pour compenser, le backend sauvegarde dans la config var
> `CALENDAR_BACKUP` quand le dyno s'arrête (SIGTERM) : les événements des
> **3 derniers mois** (les récurrents quel que soit leur âge) **et les
> réglages** (plages, villes sélectionnées) ; au démarrage d'un nouveau
> dyno, une base vide est re-remplie depuis cette sauvegarde — les
> réglages d'abord, pour que marées et vacances se re-remplissent seuls.

### Activer la sauvegarde/restauration entre dynos

1. Récupère ta clé API : dashboard Heroku → avatar → **Account Settings** →
   section **API Key** → « Reveal »
2. Dans l'app → **Settings** → **Config Vars**, ajoute :
   - `HEROKU_API_KEY` = ta clé
   - `HEROKU_APP_NAME` = le nom exact de l'app (ex. `calendrier-89594ce603e6`)

**Filet de sécurité côté téléphone (aucune configuration)** : l'app web
garde en plus une copie de l'état (événements + réglages) dans le stockage
persistant de l'appareil (webview iOS / PWA / navigateur). À chaque
ouverture, elle compare un marqueur : si le serveur a redémarré sur une
base vide (config vars absentes, crash brutal sans SIGTERM…), le téléphone
**repousse automatiquement sa copie** vers le serveur (`GET /api/state`,
`POST /api/import`, import dédupliqué). Il suffit donc d'ouvrir l'app pour
retrouver ses données — la sauvegarde Heroku reste utile pour restaurer
*sans* ouvrir l'app (abonnement ICS, autre appareil).

Sans les deux variables ci-dessus, la sauvegarde côté serveur est inactive ;
le filet côté téléphone prend le relais dès que tu ouvres l'app. Limites :
seuls les 3 derniers mois sont conservés (les événements récurrents le sont
toujours), et la sauvegarde Heroku doit tenir dans une config var (~32 Ko
compressés, largement assez pour un agenda personnel). `GET /api/export`
renvoie à tout moment le snapshot en JSON si tu veux une copie manuelle.

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

- 🎒 **Jours fériés et vacances scolaires** : les 11 fériés français calculés chaque année (Pâques par l'algorithme grégorien, testé), et les vacances scolaires **entièrement automatiques** : chaque plage et ville du catalogue connaît sa zone (A/B/C, Corse) via son académie, et les calendriers des zones de **tes lieux sélectionnés** sont récupérés depuis l'API open-data officielle de l'Éducation nationale (gratuite, sans clé) — rien à configurer, changer de plages/villes met à jour les vacances ; périodes affichées en vert sur toute leur durée
- 🏖️ **Météo des plages et des villes** : pour les plages sélectionnées (les mêmes que les marées) **et les villes de France choisies** (~45 grandes villes au catalogue, listes déroulantes dans le même sélecteur 🌊), une carte météo par lieu s'affiche en tête de l'agenda du jour — temps (soleil/nuages/pluie/orage), températures max/min, vent, indice UV, probabilité de pluie, et pour les plages **hauteur de vagues et température de l'eau** (API marine). Prévisions à 7 jours via [Open-Meteo](https://open-meteo.com) (**gratuit, sans clé — rien à configurer**), servies en direct par `GET /api/beach-weather` avec un cache mémoire de 30 min ; la météo n'est pas stockée en événements car elle change en permanence. Sélection de villes persistée en base (`GET/PUT /api/weather-cities`)
La carte météo de chaque lieu affiche aussi **lever/coucher du soleil** 🌅🌇 et une **alerte pollen** 🤧 (modéré/fort, API qualité de l'air Open-Meteo) ; la **grille du mois** montre l'emoji du temps sous chacun des 7 prochains jours (premier lieu sélectionné).

- Vue mensuelle (semaine commençant le lundi), navigation mois précédent/suivant, bouton « Aujourd'hui »
- 🔍 **Recherche** d'événements par titre (bouton loupe) — taper sur un résultat navigue vers son jour
- 🔁 **Événements récurrents** : répétition chaque semaine, chaque mois ou chaque année (anniversaires…) ; le 31 du mois est ramené au dernier jour des mois plus courts ; modifier/supprimer agit sur toute la série
- 📲 **Abonnement depuis le Calendrier iPhone/Android** : flux iCalendar sur `GET /api/calendar.ics` (récurrences en RRULE). Sur iOS : Réglages → Apps → Calendrier → Comptes → Ajouter un compte → Autre → **Ajouter un cal. avec abonnement** → `https://<ton-app>.herokuapp.com/api/calendar.ics` — tous les événements (marées comprises) apparaissent dans le calendrier natif, avec ses vraies notifications
- Création d'un événement en cliquant sur un jour ou via le bouton « + Événement »
- Édition et suppression en cliquant sur un événement
- Titre, description, date, heures de début/fin, journée entière, couleur, répétition
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
| `GET` | `/api/events?from=&to=&q=` | Liste des événements (bornes ISO 8601 et recherche titre optionnelles, récurrences développées) |
| `GET` | `/api/export` | Snapshot JSON des 3 derniers mois (celui sauvegardé entre dynos) |
| `GET` | `/api/state` | Snapshot + réglages (la copie que garde l'appareil) |
| `POST` | `/api/import` | Restaure une copie appareil (réglages puis événements, dédupliqué) |
| `GET` | `/api/events/{id}` | Détail d'un événement |
| `POST` | `/api/events` | Création |
| `PUT` | `/api/events/{id}` | Mise à jour |
| `DELETE` | `/api/events/{id}` | Suppression |
| `GET` | `/api/tide-spots` | Catalogue des plages + sélection courante |
| `PUT` | `/api/tide-spots` | Enregistre la sélection de plages (`{"spots": [...]}`) |
| `GET` | `/api/beach-weather` | Météo 7 jours des plages **et villes** sélectionnées (Open-Meteo, cache 30 min) |
| `GET` | `/api/weather-cities` | Catalogue des villes + sélection courante |
| `PUT` | `/api/weather-cities` | Enregistre la sélection de villes (`{"cities": [...]}`) |
| `GET` | `/api/calendar.ics` | Flux iCalendar (abonnement calendrier natif) |

Corps JSON pour `POST`/`PUT` :

```json
{
  "title": "Réunion équipe",
  "description": "Point hebdo",
  "start": "2026-07-16T09:00:00Z",
  "end": "2026-07-16T10:00:00Z",
  "all_day": false,
  "color": "#4f6bed",
  "recurrence": "weekly"
}
```

## App iOS (webview)

L'app iOS ([`ios/`](ios/README.md)) est une **coquille WKWebView** qui charge
l'app web : une seule cible, aucune capacité spéciale, signature automatique
sans étape manuelle — et chaque déploiement Heroku met l'app à jour sans
repasser par TestFlight. Pull-to-refresh, écran de secours pour changer
l'URL du serveur, et **notifications locales natives** : le web calcule les
rappels (événements à heure fixe des 14 prochains jours, 15 min avant ; plus
un résumé des marées du jour chaque matin) et le shell les planifie via
`UNUserNotificationCenter` — sans aucun entitlement ni bundle ID
supplémentaire. Pour des notifications *sans* l'app (marées, F1…),
l'abonnement ICS au Calendrier natif reste l'autre option.

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
