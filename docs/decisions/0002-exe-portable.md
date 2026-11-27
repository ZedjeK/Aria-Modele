# 0002 : un exe portable, sans installeur ni service

Date : 2026-11-26. Remplacée par [0008](0008-installation-pour-toutes.md) le 2026-11-27.

## Contexte

Un installeur (MSI, NSIS) demande souvent les droits administrateur, écrit dans `Program Files` et dans le registre
de la machine, et laisse des traces quand on l'enlève. Un service Windows tourne en permanence, avec des droits
élevés. Pour des outils personnels, c'est beaucoup de surface pour peu de gain.

## Décision

Chaque version est un zip qui contient un exe portable et ce qu'il livre à côté de lui (`fichiers.txt`). L'exe
tourne là où il est. Avec l'accord de l'utilisateur, il peut se ranger lui-même dans
`%USERPROFILE%\.Aria-<Nom>\programme`, sans droits administrateur, et créer un raccourci sur le Bureau de
l'utilisateur. Aucun service n'est installé. Un bouton « Désinstaller » retire ce que l'appli a écrit hors de son
dossier de données, puis `programme\`.

## Conséquences

- Rien ne demande les droits administrateur pour installer, mettre à jour ou désinstaller.
- L'appli n'apparaît pas dans « Applications installées » de Windows : elle se désinstalle par son propre bouton.
- Les mises à jour sont faites par l'appli elle-même (décision 0004).
- Pas de tâche hors session : une appli qui sert en arrière-plan démarre avec la session de l'utilisateur.

## À revoir si

- une appli doit tourner hors de toute session (vrai service) ;
- une installation pour tous les utilisateurs de la machine devient nécessaire ;
- la distribution passe par un catalogue (winget, Microsoft Store) qui impose un installeur.
