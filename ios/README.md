# Calendrier — app iOS (webview)

Coquille iOS minimale : une **WKWebView plein écran** qui charge l'app web
Calendrier (React/PWA servie par le backend). Toute l'interface — grille
mensuelle, agenda, sélecteur de plages pour les marées — vit côté web, donc
chaque déploiement Heroku met l'app à jour **sans repasser par TestFlight**.

- **Une seule cible**, aucune capacité spéciale, aucun App Group : la
  signature automatique (Xcode Cloud ou locale) n'a rien à enregistrer à
  part le bundle ID `com.maxlestage.calendrier`, déjà créé avec la fiche
  App Store Connect
- Tirer vers le bas pour recharger la page
- Si la page ne charge pas (serveur down, mauvaise adresse), un écran de
  secours permet de corriger l'URL du serveur (persistée via `@AppStorage`)
- URL par défaut : le backend Heroku (`ContentView.swift`)

## Compiler

- **Xcode Cloud** : la fiche app App Store Connect suffit — plus aucune
  étape manuelle (le widget/extension qui exigeait un bundle ID à
  enregistrer a été supprimé avec l'app native)
- **GitHub Actions** : workflow `TestFlight` (voir ci-dessous)
- **Mac local** : ouvrir `ios/Calendrier.xcodeproj`, choisir ton Team, ⌘R

## CI TestFlight (GitHub Actions, sans Mac)

Prérequis : compte Apple Developer Program payant, et les secrets GitHub
`APPLE_TEAM_ID`, `APPSTORE_KEY_ID`, `APPSTORE_ISSUER_ID`, `APPSTORE_P8`,
`DIST_CERT_BASE64`, `DIST_CERT_PASSWORD` (le certificat se crée sans Mac via
le workflow « iOS — Créer le certificat de distribution »).

Chaque push sur `master` touchant `ios/**` (ou un *Run workflow* manuel)
archive, signe et téléverse un build. Numéro de build = numéro du run.

## Structure

```
Calendrier/
  CalendrierApp.swift   # Point d'entrée
  ContentView.swift     # WebView + écran de secours (URL serveur éditable)
  WebView.swift         # WKWebView (pull-to-refresh, gestion d'erreurs)
  Assets.xcassets/      # Icône 1024 (RGB sans alpha) + couleur d'accent
```

## Ce que la webview ne fait pas (par rapport à l'ancienne app native)

- Pas de widget d'écran d'accueil ni de notifications locales — c'étaient
  les sources de toute la complexité de signature. Si besoin un jour, la
  PWA installée depuis Safari peut recevoir des notifications web push
  (iOS 16.4+), côté web uniquement.
