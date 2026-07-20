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
événements » : appui long sur l'écran d'accueil → « + » → Calendrier. Il se
rafraîchit environ toutes les 30 minutes.

> **Pas d'App Group.** Le projet n'utilise **aucune capacité spéciale** :
> c'est volontaire, car une capacité comme App Groups doit être enregistrée
> à la main sur le portail Apple et casse la signature automatique (Xcode
> Cloud comme signature auto échouent alors avec « No profiles were found »).
> Le widget lit donc l'URL du serveur dans les réglages standards que l'app y
> recopie ; si le widget tourne sans que l'app ait été ouverte, il utilise
> l'URL par défaut codée dans `CalendrierWidget.swift` (le backend déployé).

## Notifications

Réglages (⚙️) → « Notifications avant les événements » : demande la
permission puis programme un rappel **1 h avant** chaque événement horodaté
et **à 9 h** le jour même pour les événements « journée entière »
(60 prochains événements max, re-programmés à chaque rafraîchissement).

## CI TestFlight (sans Mac !)

Deux workflows GitHub Actions permettent de livrer l'app sur **TestFlight
entièrement depuis le cloud** — aucun Mac nécessaire, tout se pilote depuis
un téléphone. Prérequis : un compte **Apple Developer Program payant**
(99 €/an, TestFlight n'existe pas pour les comptes gratuits).

### Mise en place (une seule fois, ~15 min depuis un navigateur)

1. **Clé API App Store Connect** : [appstoreconnect.apple.com](https://appstoreconnect.apple.com)
   → Users and Access → Integrations → App Store Connect API → « + » (rôle
   *App Manager*). Note le **Key ID**, l'**Issuer ID**, télécharge le `.p8`
2. **Secrets GitHub** (repo → Settings → Secrets and variables → Actions) :
   - `APPSTORE_KEY_ID` — le Key ID
   - `APPSTORE_ISSUER_ID` — l'Issuer ID
   - `APPSTORE_P8` — le contenu du fichier `.p8` (texte)
   - `APPLE_TEAM_ID` — le Team ID (Membership details du compte développeur)
3. **Certificat de distribution sans Mac** : Actions → « iOS — Créer le
   certificat de distribution » → *Run workflow* en choisissant un mot de
   passe. Le résumé du job affiche le `.p12` en base64 → colle-le dans le
   secret `DIST_CERT_BASE64`, et le mot de passe dans `DIST_CERT_PASSWORD`
   (limite Apple : 2-3 certificats de distribution — réutilise-le, ne le
   régénère pas à chaque fois)
4. **Déclarer l'app dans App Store Connect** :
   [appstoreconnect.apple.com](https://appstoreconnect.apple.com) → Apps →
   « + » → nouvelle app iOS avec le bundle ID `com.maxlestage.calendrier`.
   Les App IDs `com.maxlestage.calendrier` et
   `com.maxlestage.calendrier.CalendrierWidget` s'enregistrent
   **automatiquement** à la première signature (aucune capacité spéciale à
   activer — l'App Group a été retiré exprès).

### Utilisation

Chaque push sur `master` touchant `ios/**` (ou un *Run workflow* manuel)
archive, signe et téléverse un build : workflow **TestFlight**. Le numéro de
build = numéro du run. Après 5-30 min de traitement Apple, le build apparaît
dans App Store Connect → TestFlight, et l'app s'installe sur l'iPhone via
l'app TestFlight (testeur interne = pas de review Apple).

### Xcode Cloud (alternative sans GitHub Actions)

Si tu utilises plutôt **Xcode Cloud** (configurable depuis App Store Connect),
la fiche app dans App Store Connect suffit pour l'app principale : créer la
fiche enregistre son bundle ID (`com.maxlestage.calendrier`).

> ⚠️ **Extension widget : une étape manuelle unique.** La signature
> éphémère de Xcode Cloud sait créer des *profils* pour un bundle ID
> existant, mais **ne peut pas créer le bundle ID d'une extension**. Sans
> lui, l'export échoue avec « Automatic signing cannot register bundle
> identifier "com.maxlestage.calendrier.CalendrierWidget" ». Deux façons de
> l'enregistrer (une seule fois) :
>
> 1. **Portail Apple (2 min, marche au téléphone)** :
>    [developer.apple.com/account/resources/identifiers](https://developer.apple.com/account/resources/identifiers/list)
>    → « + » → App IDs → type *App* → identifiant **explicite**
>    `com.maxlestage.calendrier.CalendrierWidget`, description
>    « Calendrier Widget », **aucune capacité** à cocher → Register.
> 2. **Workflow GitHub** « iOS — Enregistrer un bundle ID » (Actions →
>    Run workflow) — nécessite les secrets `APPSTORE_*` ci-dessus.
>
> Ensuite, relancer un build Xcode Cloud : la signature trouve l'identifiant
> et crée les profils toute seule.

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
  Assets.xcassets/           # Icône d'app 1024 (RGB, sans alpha) + couleur d'accent
CalendrierWidget/
  CalendrierWidget.swift     # Widget « Prochains événements » (small/medium)
  Info.plist                 # Point d'extension WidgetKit
```
