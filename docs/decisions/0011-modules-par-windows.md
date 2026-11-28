# 0011 : Les modules en option passent par Windows, sans greffon Tauri

Date : 2026-11-28. Précise [0007](0007-socle-partage.md).

## Contexte

Le contrat prévoit cinq modules en option qui restaient à faire : notifications, raccourcis clavier globaux, fichiers
et dossiers, échanges entre applis, droits administrateur. Tauri propose des greffons pour les trois premiers, mais
Tauri ne lit les autorisations d'un greffon que dans les dépendances directes de l'appli : chaque appli devrait
ajouter le greffon et ses autorisations à la main, et le socle ne pourrait pas les tenir à jour à sa place.

Les échanges entre applis demandent aussi de savoir comment une appli en trouve une autre sur la boucle locale.

## Décision

- Chaque module est une fonction de la crate `aria-coeur`, écrite avec l'API de Windows (crates `windows` et
  `winreg`) : fenêtres « Ouvrir » et « Enregistrer » par `IFileOpenDialog` et `IFileSaveDialog`, associations dans
  `HKCU\Software\Classes`, notifications de Windows, raccourcis par `RegisterHotKey`. Aucun greffon Tauri.
- Les commandes d'un module ne sont publiées à l'appli (`DEP_ARIA_COEUR_COMMANDES`) et autorisées (capacité du
  module) que si sa fonction est activée.
- **Échanges entre applis** : chaque appli qui les active a un port fixe, noté dans `docs/PORTS.md` ; s'il est pris au
  lancement, elle écoute sur un port libre choisi par Windows. Dans les deux cas, le port et le jeton du lancement sont
  annoncés dans le dossier commun `%USERPROFILE%\.Aria`, et retirés à la fermeture.

## Conséquences

- Une appli active un module par une ligne dans `appli/Cargo.toml` ; ses commandes, ses autorisations et ses réglages
  viennent du socle.
- Le code Windows du socle grandit, avec ses blocs `unsafe` (un par appel, la raison écrite) ; il se vérifie par la CI
  Windows et par la recette, pas sur Linux.
- Un port fixe se retrouve facilement en cas de problème ; le port de secours évite qu'un port pris bloque l'appli.
- Le jeton annoncé dans le dossier commun est lisible par tout programme lancé sous le même compte : la même limite
  que DPAPI (décision 0003), que le README de l'appli dit dans « Limites à connaître ».

## À revoir si

- Tauri sait lire les autorisations d'un greffon venu d'une dépendance indirecte ;
- une API de Windows utilisée par un module est retirée ou change de comportement ;
- un programme tiers lit le jeton d'échange pour piloter une appli.
