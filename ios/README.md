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
  CalendrierApp.swift      # Point d'entrée
  ContentView.swift        # WebView + écran de secours (URL serveur éditable)
  WebView.swift            # WKWebView (pull-to-refresh, gestion d'erreurs)
  NotificationBridge.swift # Notifications locales pilotées par le web
  Assets.xcassets/         # Icône 1024 (RGB sans alpha) + couleur d'accent
```

## Notifications locales (natif, sans complexité de signature)

Les **notifications locales** (rappels d'événements) marchent depuis la
coquille WKWebView **sans nouvelle capacité, entitlement ni bundle ID** —
contrairement au widget/aux notifications push qui avaient tout cassé.
`UNUserNotificationCenter` planifie des notifications locales dans la cible
principale, il faut seulement l'autorisation de l'utilisateur (demandée à la
première programmation).

Le **web reste le cerveau** : l'app web calcule la liste des rappels et
l'envoie au shell via `window.webkit.messageHandlers.reminders` :

- les événements *à heure fixe* des 14 prochains jours (sauf marées —
  4/jour = trop), rappel **15 min avant** ;
- un **résumé des marées du jour** chaque matin (07:00 heure locale) par
  plage sélectionnée (pleines/basses mers), au lieu d'alertes individuelles.

`NotificationBridge` ne fait que planifier ce qu'il reçoit (au plus 60, la
limite iOS étant 64). Aucune logique dupliquée : la même app web sert la
PWA, le navigateur et la coquille iOS.

> Le `project.pbxproj` est en format synchronisé (Xcode 16,
> `PBXFileSystemSynchronizedRootGroup`) : `NotificationBridge.swift` est
> inclus automatiquement, aucune manip du projet.

## Ce que la webview ne fait toujours pas

- Pas de widget d'écran d'accueil (c'était la source de la complexité de
  signature : une extension = un bundle ID à enregistrer). Les
  notifications, elles, ne l'exigent pas et sont donc désormais natives.
