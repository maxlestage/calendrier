# Calendrier — app iOS native (SwiftUI)

App iOS native connectée au backend Calendrier (Actix Web + SeaORM). Même
philosophie que le front web mobile-first : grille mensuelle avec pastilles
colorées, agenda du jour sélectionné, bouton « + », création/édition en sheet.

## Prérequis

- Un Mac avec **Xcode 16 ou plus récent** (le projet utilise les dossiers
  synchronisés, format Xcode 16)
- iOS 17+ sur l'appareil ou le simulateur
- Un compte Apple (gratuit) pour signer l'app et l'installer sur un iPhone

## Compiler et lancer

1. Ouvrir `ios/Calendrier.xcodeproj` dans Xcode
2. Cible « Calendrier » → onglet *Signing & Capabilities* → choisir ton
   *Team* (ton Apple ID) ; Xcode gère la signature automatiquement
3. Sélectionner ton iPhone (ou un simulateur) et ⌘R

Avec un compte Apple gratuit, l'app installée sur un vrai iPhone expire au
bout de 7 jours (limitation Apple) — il suffit de re-lancer depuis Xcode.
Pour une installation durable : compte développeur payant + TestFlight.

## Configuration

Au premier lancement, ouvre ⚙️ (en haut à droite) et renseigne l'URL de ton
backend, par ex. `https://ton-app.herokuapp.com`. L'app parle à
`/api/events` en JSON, comme le front web. La valeur est mémorisée
(`@AppStorage`).

Note : iOS exige HTTPS (App Transport Security). Le backend Heroku est en
HTTPS, donc rien à faire. Pour tester contre un backend local en HTTP, il
faudrait ajouter une exception ATS dans les réglages du projet.

## Widget d'écran d'accueil

La cible **CalendrierWidget** fournit un widget (petit et moyen) « Prochains
événements » : appui long sur l'écran d'accueil → « + » → Calendrier. Le
widget lit l'URL du serveur via l'App Group `group.com.maxlestage.calendrier`
(partagé avec l'app) et se rafraîchit environ toutes les 30 minutes.

Si la signature refuse l'App Group (selon le type de compte), supprime la
capacité « App Groups » des deux cibles : le widget utilisera alors l'URL
par défaut codée dans `CalendrierWidget.swift` (à adapter).

## Notifications

Réglages (⚙️) → « Notifications avant les événements » : demande la
permission puis programme un rappel **1 h avant** chaque événement horodaté
et **à 9 h** le jour même pour les événements « journée entière »
(60 prochains événements max, re-programmés à chaque rafraîchissement).

## Structure

```
Calendrier/
  CalendrierApp.swift        # Point d'entrée
  ContentView.swift          # Navigation, toolbar, FAB, sheets
  CalendarViewModel.swift    # État (mois, jour sélectionné, événements), API
  APIClient.swift            # Client REST /api/events
  Models.swift               # CalendarEvent, EventPayload (JSON snake_case)
  ColorHex.swift             # "#rrggbb" → Color
  Views/
    MonthGridView.swift      # Grille 6 semaines, lundi en premier, pastilles
    DayAgendaView.swift      # Liste des événements du jour
    EventFormView.swift      # Formulaire création/édition (Form + sheet)
    SettingsView.swift       # URL du serveur + rappels
  Notifications.swift        # Programmation des rappels locaux
  Calendrier.entitlements    # App Group (partage de l'URL avec le widget)
CalendrierWidget/
  CalendrierWidget.swift     # Widget « Prochains événements » (small/medium)
  Info.plist                 # Point d'extension WidgetKit
  CalendrierWidget.entitlements
```
