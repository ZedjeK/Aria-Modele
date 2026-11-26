# Notes de version

Un fichier par tag : `v1.2.3.md` pour le tag `v1.2.3`. Le workflow de release publie ce fichier tel quel dans la
release en brouillon ; s'il manque, aucune release n'est créée. `.\tools\verifier-release.ps1 -Tag v1.2.3` vérifie
avant le tag que les notes, la fiche projet et les licences sont complètes.

Pour une nouvelle version, pars du modèle `.github/release-notes.md`. Si l'appli a un `Cargo.lock`, régénère d'abord la
liste des crates Rust de `LICENCES-TIERS.txt` par `.\tools\licences-rust.ps1` : `verifier-release.ps1` refuse une liste
qui ne correspond plus à `Cargo.lock`.
