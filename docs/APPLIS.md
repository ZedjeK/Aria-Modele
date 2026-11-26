# Les applis de la famille

Fichier du dépôt modèle seulement : `init.ps1` le retire de l'appli créée.

Une ligne par appli publique. Relevé le 2026-11-26 par `.\tools\etat-flotte.ps1`, qui donne aussi les PR Dependabot
ouvertes, le ruleset et la dernière analyse CodeQL ; à relancer pour un état à jour. Toutes les applis migrent vers la
stack `rust-tauri` (décision 0001), une à la fois.

| Appli | Stack actuelle (GitHub) | Dernière version | Socle commun | Ordre de migration |
| --- | --- | --- | --- | --- |
| Aria-Cockpit | C# | v0.29.1 | 9 fichiers en retard | à décider par l'auteur |
| Aria-Compagnon | inconnue (aucun code) | aucune | 9 fichiers en retard | à décider par l'auteur |
| Aria-Etabli | Python | aucune | 5 fichiers en retard | à décider par l'auteur |
| Aria-JDR | Python | v0.3.0 | 8 fichiers en retard | à décider par l'auteur |
| Aria-MD | Rust | v0.1.0 | 5 fichiers en retard | à décider par l'auteur |
| Aria-Pilote | Python | v0.27.0 | 9 fichiers en retard | à décider par l'auteur |
| Aria-Plume | JavaScript | v0.40.0 | 9 fichiers en retard | à décider par l'auteur |
| Aria-Scope | Go | v0.6.0 | 9 fichiers en retard | à décider par l'auteur |
| Aria-Shield | Go | v0.12.0 | 9 fichiers en retard | à décider par l'auteur |
| Aria-Studio | Python | v2.4.0 | 5 fichiers en retard | à décider par l'auteur |
| Aria-Studio_refonte | Rust | aucune | 5 fichiers en retard | à décider par l'auteur |
| Aria-Vault | Go | v0.8.0 | 9 fichiers en retard | à décider par l'auteur |

« Stack actuelle » est le langage principal que GitHub attribue au dépôt. « Socle commun » compte les fichiers de
`tools/socle.txt` du modèle absents de l'appli ou différents ; `.\tools\verifier-socle.ps1`, lancé dans l'appli, les
nomme.
