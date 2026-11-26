# Contribuer

Ce dépôt suit le contrat des applis de la famille, [docs/CONTRAT.md](docs/CONTRAT.md) : il fait foi. Une modification
qui change un comportement met à jour le contrat (dans le dépôt modèle), la fiche projet, le README (dont « Limites à
connaître ») et le CHANGELOG dans la même pull request.

## Langue

Français et tutoiement partout : interface, README, messages d'erreur, commentaires, commits, pull requests, issues.

## Compiler et tester

Il faut Rust, installé par [rustup](https://rustup.rs) (la version est dans `rust-toolchain.toml`), et Node.js (la
version exacte est dans `.node-version`). Puis, dans le dossier du dépôt :

```powershell
rustup toolchain install
.\tools\tester.ps1
.\build.ps1
```

`tester.ps1` lance les contrôles et les tests de l'interface et du Rust ; `build.ps1` construit l'exe dans
`.work\sortie\`. Les deux passent avant tout commit. Le contrôle des dépendances Rust (`cargo deny`, règles dans
`deny.toml`) demande cargo-deny ; sans lui, `tester.ps1` saute ce contrôle et le dit, et la CI le fait.

## Une pull request par sujet

- Jamais de commit direct sur `main` : une branche courte depuis `main` à jour, puis une pull request.
- Une pull request ne traite qu'un sujet ; un bug trouvé en chemin devient une issue.
- Branches : `feat/…`, `fix/…`, `docs/…`, `ci/…`, `deps/…`, `securite/…`, `release/…`.
- Titre « Sujet : détail » ; le corps dit quoi, pourquoi, la preuve (tests lancés, sortie utile) et ce qui n'a pas pu
  être vérifié. Une issue réglée se lie par `Closes #n`.
- Un label par pull request, qui la range dans les notes de version :

  | Label | Pour |
  | --- | --- |
  | `feat` | une fonction nouvelle |
  | `fix` | une correction |
  | `deps` | une dépendance |
  | `docs` | la documentation |
  | `ci` | la CI, les workflows, les outils du dépôt |
  | `securite` | la sécurité |

- Rien de secret, ni d'adresse personnelle, ni de chemin propre à un poste dans un commit : le dépôt est public.

## Fusion

- `main` est protégée : pull request obligatoire, check `tests` exigé, alertes CodeQL de sévérité élevée bloquantes.
- Une pull request se fusionne en « squash », seulement sur une CI verte.
- Une modification fusionnée qui casse quelque chose se défait par une nouvelle pull request (`git revert`), jamais en
  réécrivant `main`.

## Versions

La version vit à un seul endroit, lu par `.\tools\version.ps1`, et le tag `vX.Y.Z` lui est égal.

1. Une pull request monte la version et ajoute les notes `docs/releases/vX.Y.Z.md` (à partir de
   `.github/release-notes.md`), complètes, avec « décompresse d'abord le zip » ; s'il y a un `Cargo.lock`,
   `.\tools\licences-rust.ps1` régénère la liste des crates de `LICENCES-TIERS.txt`.
2. `.\tools\verifier-release.ps1 -Tag vX.Y.Z` vérifie les notes, la fiche et les licences.
3. Le tag `vX.Y.Z`, posé sur le commit de fusion dans `main`, déclenche `release.yml` : tests, exe, zip, empreinte,
   attestation de provenance, puis une release en **brouillon**. Aucun workflow ne publie.
4. Publication d'abord en prerelease, pour essayer la version sans l'annoncer : l'appli ne lit que la dernière
   version publiée hors prerelease.
5. Puis passage en version courante.

Essai à blanc, sans tag : `gh workflow run Release --ref main` construit le zip et liste ce qui manque encore.

### Retour arrière

Les tags `v*` sont verrouillés : ils ne se suppriment ni ne se déplacent.

- **Tag poussé par erreur, release encore en brouillon** : supprime le brouillon (`gh release delete vX.Y.Z`, sans
  toucher au tag). Le numéro est perdu : la version suivante prend le numéro d'après.
- **Version publiée avec un défaut** : publie une version corrective `vX.Y.(Z+1)` ; une version publiée ne change
  plus, et on ne réutilise jamais un numéro.

## Signaler

- Un bug : le formulaire d'issue, avec le diagnostic de l'appli (page Journal, « Exporter le diagnostic »).
- Une faille de sécurité : jamais en issue publique ; voir [SECURITY.md](SECURITY.md).
