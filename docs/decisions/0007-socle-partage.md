# 0007 : un socle partagé et versionné, publié depuis Aria-Modele

Date : 2026-11-27.

## Contexte

`init.ps1` copie Aria-Modele une fois dans chaque appli, puis chaque copie vit sa vie. Au 2026-11-26, les douze
applis de la famille avaient chacune de 5 à 9 fichiers du socle commun en retard (`docs/APPLIS.md`), et
`docs/MODELE-JOURNAL.md` tient la liste de ce qu'il faut reporter à la main.

La dérive a déjà commencé dans le code. Aria MD et Aria Studio refonte sont déjà sur `rust-tauri` et portent
chacune leur copie du cœur : au 2026-11-27, sur six fichiers du cœur comparés à ceux du modèle (`donnees.rs`,
`reglages.rs`, `journal.rs`, `secrets.rs`, `mises_a_jour.rs`, `installation.rs`), quatre diffèrent dans Aria MD et
cinq dans Aria Studio refonte. Une fois toutes les applis passées sur `rust-tauri` (décision 0001), chaque correctif
de sécurité se ferait douze fois. Les dix autres applis n'ont pas encore migré : changer la façon de partager
maintenant ne coûte que ces deux reprises.

## Décision

- Le code commun devient un socle versionné, publié depuis Aria-Modele, qui garde aussi le squelette. Le socle
  comprend :
  - la crate `aria-coeur`, avec ce que chaque appli a toujours : dossier de données, réglages au format versionné,
    journal, sauvegardes et restauration, secrets, instance unique, installation, désinstallation, mises à jour,
    diagnostic, tâches longues, reprise des données d'une ancienne version ;
  - les modules en option, comme fonctions à activer de cette crate (features Cargo) : démarrage avec Windows et
    icône de notification, moteur local et ses mises à jour, téléchargement vérifié, notifications, raccourcis
    globaux, fichiers et dossiers, partage de la VRAM, échanges entre applis, droits administrateur limités à
    l'opération qui les exige ;
  - un paquet pour l'interface commune : thème, composants, dialogues, fond animé, assistant de premier lancement ;
  - des workflows GitHub réutilisables : tests, release, CodeQL, tests de l'installation sous Windows.
- Le socle et le squelette ont une seule version et un seul tag `vX.Y.Z` : ils changent ensemble.
- Les applis récupèrent le socle directement depuis Aria-Modele, à un tag précis, sans registre public :
  - la crate par une dépendance git, déclarée dans le manifeste de l'appli (`appli/Cargo.toml`) :
    `aria-coeur = { git = "https://github.com/ZedjeK/Aria-Modele", tag = "vX.Y.Z" }` ;
  - le paquet d'interface par son archive (`.tgz`), jointe à la release `vX.Y.Z` d'Aria-Modele : npm ne sait pas
    installer un sous-dossier d'un dépôt git. Le fichier de verrouillage de l'appli garde l'empreinte de l'archive.
- Une fonction retirée du socle est d'abord marquée dépréciée dans une version mineure, puis retirée dans la version majeure
  suivante. Chaque version a son entrée dans le journal des changements.
- Une appli dépend du socle par une version précise. Un workflow réutilisable du socle lit chaque semaine la
  dernière release d'Aria-Modele et ouvre une seule PR qui monte la crate et le paquet ensemble, testée par la CI de
  l'appli. Dependabot continue de suivre les autres dépendances, et les workflows réutilisables, appelés par leur
  SHA.
- `init.ps1` ne copie plus le code du socle dans l'appli créée : elle reçoit un squelette mince (le point d'entrée,
  une page d'exemple, la fiche, le README) et dépend du socle.
- La v1 d'Aria-Modele est gelée : plus aucune appli n'est créée avec elle. Ses écarts connus (audit du 2026-11-27)
  sont corrigés dans le socle, pas dans la v1.

## Conséquences

- Un correctif du socle arrive dans chaque appli par une PR, au lieu d'être reporté à la main. `tools/socle.txt`,
  `tools/verifier-socle.ps1` et les marqueurs de stack d'`init.ps1` se réduisent d'autant.
- Une version du socle peut casser une appli : la CI de chaque appli couvre ce qu'elle utilise du socle, et une
  rupture passe toujours par une version majeure.
- Aria-Modele porte deux rôles : ses protections, sa CI et ses releases couvrent le socle et le squelette ensemble.
  Une correction du squelette seul fait aussi monter la version du socle.
- Aucun compte ni jeton de registre : tout reste sur GitHub. En contrepartie, la montée de version du socle repose
  sur un workflow maison, testé comme le reste du socle.
- `deny.toml` des applis autorise une seule source git : Aria-Modele.
- Le code du socle est public, comme celui du modèle aujourd'hui.

## À revoir si

- une appli doit rester longtemps sur une version du socle que les autres ont quittée ;
- le socle et le squelette ont besoin de rythmes de version différents : ils se séparent alors en deux dépôts ;
- le workflow de montée de version coûte plus à tenir qu'une publication sur crates.io et npm ;
- Dependabot cesse de suivre les workflows réutilisables ;
- la famille tombe à trois applis vivantes ou moins : recopier coûte alors moins que publier.
