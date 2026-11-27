# Décisions d'architecture

Une page par décision qui engage toutes les applis de la famille : ce qui a été décidé, pourquoi, ce que ça coûte, et
ce qui doit faire revenir dessus. Le contrat (`docs/CONTRAT.md`) dit les règles ; ces pages disent d'où elles viennent.

Chaque page suit le même plan :

- **Contexte** : la situation et la question posée ;
- **Décision** : ce qui a été choisi ;
- **Conséquences** : ce que ça permet, ce que ça coûte ;
- **À revoir si** : les signes qui doivent faire rouvrir la question.

Une décision ne se réécrit pas : une nouvelle page la remplace, et l'ancienne dit par quelle page elle est remplacée.

| N° | Décision |
| --- | --- |
| [0001](0001-stack-unique-rust-tauri.md) | Une seule stack, Rust et Tauri, pour Windows seulement |
| [0002](0002-exe-portable.md) | Un exe portable, sans installeur ni service (remplacée par 0008) |
| [0003](0003-secrets-dpapi.md) | Les secrets chiffrés par DPAPI |
| [0004](0004-mises-a-jour.md) | Les mises à jour : release GitHub, SHA-256, attestation, signature minisign |
| [0005](0005-aucune-telemetrie.md) | Aucune télémétrie |
| [0006](0006-signature-signpath.md) | La signature Authenticode par SignPath Foundation |
| [0007](0007-socle-partage.md) | Un socle partagé et versionné, publié depuis le dépôt modèle |
| [0008](0008-installation-pour-toutes.md) | Chaque appli s'installe, sans lancement portable |
| [0009](0009-modeles-et-moteurs-partages.md) | Les modèles et les moteurs locaux partagés entre les applis |
| [0010](0010-windows-x64-seul.md) | Windows x64 seulement |
