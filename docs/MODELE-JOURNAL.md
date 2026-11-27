# Journal du dépôt modèle

Fichier du dépôt modèle seulement : `init.ps1` le retire de l'appli créée.

Une section par version du modèle, la plus récente en haut. Chaque section dit ce qui a changé et, surtout, **ce qu'il
faut reporter dans les applis déjà créées** : `.\tools\verifier-socle.ps1` signale les fichiers du socle en retard,
mais pas les fichiers à marqueurs (workflows, README, fiche, squelette) ni les gestes à faire à la main.

## v2.0.0 (en préparation)

Socle partagé au lieu de copié, installation pour toutes les applis, modèles et moteurs partagés : décisions 0007
à 0010. La v1 est gelée : plus aucune appli n'est créée avec elle.

### Ce qui change

| PR | Sujet |
| --- | --- |
| #76 | Décisions d'architecture : 0007 à 0010, la v2 du modèle |
| #77 | Contrat v2 et recette commune (`docs/RECETTE.md`) |
| #78 | Socle Rust partagé : crate `aria-coeur` dans `socle/coeur`, fenêtre Tauri comprise ; le squelette ne garde que son identité et ses commandes |
| #79 | Interface partagée : paquet `aria-interface` dans `socle/interface` (thème, composants, fenêtre, dialogues, Journal, Paramètres, installation, menu de l'icône) |
| #80 | Publication du socle : release du dépôt modèle avec l'archive `aria-interface-X.Y.Z.tgz` (`socle-release.yml`), PR hebdomadaire « Monter le socle » dans chaque appli (`monter-socle.yml`), CodeQL du modèle sur le Rust et le TypeScript du socle |
| #81 | Corrections de l'audit : raccourci du Bureau jamais remplacé s'il pointe ailleurs (I1), réglages au format versionné, lus champ par champ et sauvegardés avant réécriture (I2), tailles plafonnées des mises à jour (M4), copies et mesures hors du fil de la fenêtre (M5), noms réservés de Windows refusés pour les secrets (M6), notes, init.ps1 et modèle de PR (M3, M8 à M12) |
| #82 | Installation pour toutes (décision 0008) : plus de lancement sans installation ni de `--portable`, « Annuler » ferme l'appli, l'aperçu du zip demande de décompresser d'abord ; `init.ps1` perd l'option `-Installation` et les squelettes la clé `nonFournis` |

### À reporter dans les applis

**Socle commun** (`.\tools\verifier-socle.ps1` les signale ; à recopier tels quels) :

- nouveaux : `docs/decisions/0007-socle-partage.md`, `0008-installation-pour-toutes.md`,
  `0009-modeles-et-moteurs-partages.md`, `0010-windows-x64-seul.md` ;
- nouveau : `docs/RECETTE.md` ;
- modifiés : `docs/CONTRAT.md` (v2), `docs/decisions/0002-exe-portable.md`, `docs/decisions/LISEZ-MOI.md`,
  `tools/socle.txt`, `.github/pull_request_template.md`.

## v1.1.0 (proposée, pas encore taguée)

### Ce qui change

| PR | Sujet |
| --- | --- |
| #58 | Contrat public : `docs/CONTRAT.md`, au socle |
| #59 | Une promesse longue ne casse plus rustfmt dans l'appli créée |
| #60 | Désinstallation : bouton « Désinstaller » dans les Paramètres |
| #61 | Sans dépendance : WebView2 détecté, runtime Visual C++ vérifié |
| #62 | CI : analyse des scripts PowerShell par PSScriptAnalyzer |
| #63 | Dépendances Rust : cargo deny, liste des licences par cargo-about |
| #64 | Dépendances : cookie 0.18.3 (faille) |
| #65 | Dépendances : time remplace chrono dans le cœur |
| #66 | CI : Node.js épinglé par `.node-version` |
| #68 | Release : signature Authenticode par SignPath, préparée et coupée |
| #69 | Diagnostic : bouton « Exporter le diagnostic » |
| #70 | Fiche : cap de la prochaine version et budgets ; taille de l'exe dans `BUILDINFO.txt` |
| #71 | Décisions d'architecture : `docs/decisions/` |
| #72 | `CONTRIBUTING.md` |
| #73 | Ce journal |
| #74 | État de la flotte : `tools/etat-flotte.ps1` et `docs/APPLIS.md`, réservés au modèle |

Pas encore faits, faute des clés minisign : signature des mises à jour, canal d'essai, recette et chaîne de release
complète.

### À reporter dans les applis

**Socle commun** (`.\tools\verifier-socle.ps1` les signale ; à recopier tels quels) :

- nouveaux : `docs/CONTRAT.md`, `CONTRIBUTING.md`, `PSScriptAnalyzerSettings.psd1`, `docs/decisions/` (sept fichiers) ;
- modifiés : `tools/socle.txt`, `tools/verifier-release.ps1` (empreinte de `Cargo.lock` dans `LICENCES-TIERS.txt`),
  `docs/releases/LISEZ-MOI.md`.

**Fichiers à marqueurs, à reporter à la main** (en remplaçant `{{NOM}}` par le nom de l'appli) :

| Fichier | Ce qui change |
| --- | --- |
| `.github/workflows/tests.yml` | étape « Analyse des scripts PowerShell » |
| `.github/workflows/tests.yml`, `release.yml` (bloc des outils) | `setup-node` par `node-version-file: .node-version` ; `taiki-e/install-action` (cargo-deny 0.20.2, cargo-about 0.9.3) |
| `.github/workflows/release.yml` | trois étapes de signature SignPath (coupées), `timeout-minutes: 60`, `actions: read`, gestes d'activation en tête |
| `.github/ISSUE_TEMPLATE/bug.yml` | champ « Diagnostic » à la place de « Extrait du journal » |
| `tools/empaqueter.ps1` | taille de l'exe dans `BUILDINFO.txt` |
| `README.md` | Windows 11 ou 10 22H2, « Si ça ne marche pas », « Désinstaller », signalement par le diagnostic, cargo-deny, `.node-version`, lien vers la politique de signature |
| `docs/FICHE.md` | quatre lignes : objectif de la prochaine version, non-objectifs, plateformes, budget |
| `docs/POLITIQUE-SIGNATURE.md` | nouveau |
| `LICENCES-TIERS.txt` | les blocs « Tauri » et « Autres crates » deviennent la partie générée (lignes repères) |

**Squelette rust-tauri** (à reporter fichier par fichier, ou par une migration de l'appli) : `.node-version`,
`deny.toml`, `about.toml`, `about.hbs`, `tools/licences-rust.ps1`, `tools/tester.ps1` (runtime Visual C++,
cargo deny), `coeur` (désinstallation, diagnostic, `time` au lieu de `chrono`, `#[rustfmt::skip]` sur `PROMESSE`),
`appli` (désinstallation, WebView2, diagnostic), interface (Paramètres, Journal, dialogue de désinstallation),
`Cargo.lock` (cookie 0.18.3, time).

**Gestes à la main** :

- compléter les quatre nouvelles lignes de `docs/FICHE.md` : `verifier-release.ps1` les exige avant une version ;
- avant la prochaine version : `.\tools\licences-rust.ps1` (cargo-about 0.9.3) ;
- rien pour la signature SignPath tant que l'appli n'est pas acceptée par la fondation ; avant de l'activer :
  ZedjeK/Aria-Modele#67 (signature dans un job séparé).

## v1.0.0

Première version publiée du modèle : stack `rust-tauri` seule, contrat des applis tenu par le squelette (dossier de
données, journal, secrets DPAPI, icône de notification, démarrage automatique, installation par l'exe, mises à jour
vérifiées), interface sur le modèle d'Aria Studio et d'Aria Cockpit.
