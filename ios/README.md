# Calendrier — app iOS **native (SwiftUI)**

Vraie application iOS native, écrite en **SwiftUI**. Plus de WebView : toute
l'interface (grille mensuelle, agenda, cartes météo, marées, éditeur
d'événements, recherche, réglages) est en code natif et parle au backend
Rust via son **API REST**.

Le backend reste le cerveau (marées, météo, F1, vacances, astronomie,
sauvegardes) ; l'app est un **client natif** de cette API.

- **Une seule cible**, aucune capacité spéciale, aucun App Group : la
  signature automatique n'a rien à enregistrer à part le bundle ID
  `com.maxlestage.calendrier`, déjà créé côté App Store Connect.
- **Notifications locales natives** (`UNUserNotificationCenter`) — aucun
  entitlement ni bundle ID supplémentaire :
  - **rappel avant chaque événement** à heure fixe (délai réglable) ;
  - **résumé du matin** (météo + marées + événements) à l'heure choisie.
- Splash de chargement, pull-to-refresh implicite via les rechargements.
- **Persistance à toute épreuve** (`DeviceBackup.swift`) : l'app garde une
  copie de tout (événements + réglages) sur le téléphone
  (`Application Support`). Au lancement, elle compare un marqueur avec le
  serveur : si le dyno Heroku a été remis à zéro, elle **repousse
  automatiquement sa copie** (`GET /api/state`, `POST /api/import`). Marche
  sans aucune config (pas besoin de clé Heroku) et survit même à un crash
  brutal. Bonus : hors-ligne ou serveur down, l'app affiche les données
  locales.

## Structure

```
Calendrier/
  CalendrierApp.swift    # @main → RootView
  RootView.swift         # Écran principal (barre, grille, agenda, FAB, sheets)
  MonthView.swift        # Grille mensuelle (dots, heures de marée, météo/jour)
  AgendaView.swift       # Agenda du jour + cartes météo
  EventEditorView.swift  # Création / édition / suppression d'événement
  SearchView.swift       # Recherche par titre
  SettingsView.swift     # Plages, villes, notifications, URL serveur
  Store.swift            # CalendarStore (ObservableObject) — état global
  API.swift              # Client REST du backend Rust
  Models.swift           # Types Codable (miroir de l'API)
  Notifications.swift    # Planification des notifications locales
  Utils.swift            # Dates ISO, grille, couleurs hex, emoji météo
  Assets.xcassets/       # Icône 1024 (RGB sans alpha) + couleur d'accent
```

## Configuration

- **URL du serveur** : par défaut le backend Heroku ; modifiable dans
  Réglages (⚙️) → section « Serveur » (persisté via `@AppStorage`).
- iOS 17 minimum. Pas d'`Info.plist` manuel (généré),
  `ITSAppUsesNonExemptEncryption=NO`, iPhone portrait.

## Compiler / publier

- **Xcode Cloud** : la fiche App Store Connect suffit (bundle ID déjà créé),
  aucune étape manuelle.
- **GitHub Actions** : workflow `TestFlight` (secrets `APPLE_TEAM_ID`,
  `APPSTORE_KEY_ID`, `APPSTORE_ISSUER_ID`, `APPSTORE_P8`, `DIST_CERT_BASE64`,
  `DIST_CERT_PASSWORD`). Le certificat se crée sans Mac via le workflow
  « iOS — Créer le certificat de distribution ».
- **Mac local** : ouvrir `ios/Calendrier.xcodeproj`, choisir ton Team, ⌘R.

Le `project.pbxproj` est en format synchronisé (Xcode 16,
`PBXFileSystemSynchronizedRootGroup`) : les fichiers `.swift` du dossier sont
inclus automatiquement, aucune manip du projet à l'ajout/suppression.
