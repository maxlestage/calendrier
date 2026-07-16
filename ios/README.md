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
    SettingsView.swift       # URL du serveur
```
