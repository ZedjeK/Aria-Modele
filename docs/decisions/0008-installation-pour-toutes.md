# 0008 : chaque appli s'installe, sans lancement portable

Date : 2026-11-27. Remplace [0002](0002-exe-portable.md).

## Contexte

La décision 0002 laissait l'exe tourner là où il avait été décompressé, avec une installation proposée mais
facultative. Un exe lancé depuis Téléchargements n'a pas d'adresse stable : l'entrée de démarrage avec Windows, le
raccourci du Bureau et les mises à jour pointent vers un fichier que l'utilisateur peut déplacer ou effacer en
vidant le dossier. Au 2026-11-27, cinq applis de la famille ne proposent pas de s'installer (Etabli, JDR, Scope,
Shield, Vault) et chacune règle la question à sa façon.

## Décision

- Chaque appli s'installe dans `%USERPROFILE%\.Aria-<Nom>\programme`, sans droits administrateur. Lancé hors de ce
  dossier, l'exe propose de s'installer ; s'il est refusé, l'appli se ferme. Il n'y a ni lancement sans
  installation, ni option `--portable`.
- Lancé depuis l'aperçu d'un zip, l'exe ne s'installe pas : il demande de décompresser le zip d'abord.
- Le raccourci du Bureau reste une case, décochée d'office. Un raccourci déjà présent n'est jamais remplacé en
  silence : il est proposé en mise à jour s'il pointe vers l'appli, laissé tel quel sinon.
- Chaque appli a un bouton « Désinstaller » dans ses paramètres : il retire l'entrée de démarrage, le raccourci et
  `programme\`, et garde les données.
- Les mises à jour remplacent l'exe dans `programme\` et gardent la version précédente dans
  `sauvegardes\versions\`.
- Toujours ni installeur (MSI, NSIS), ni service, ni écriture dans `Program Files` pour une appli lancée avec les
  droits de l'utilisateur. Une appli qui tourne élevée suit la règle du contrat sur les droits de son dossier.

## Conséquences

- L'exe a toujours la même adresse : démarrage avec Windows, raccourci et mises à jour pointent vers un fichier qui
  ne bouge pas.
- Un pas de plus au premier lancement, et l'installation devient un chemin que chaque appli emprunte : elle se teste
  automatiquement dans la CI, pas seulement par un clic à la main.
- Pour essayer une appli sur un autre PC, on l'installe puis on la désinstalle.
- Les cinq applis portables changent de comportement à leur migration : leurs notes de version le disent, et la
  première version installée reprend les données de l'ancienne.
- L'appli n'apparaît pas dans « Applications installées » de Windows : elle se désinstalle par son propre bouton.

## À revoir si

- une appli doit tourner hors de toute session (vrai service) ;
- une installation pour tous les utilisateurs de la machine devient nécessaire ;
- la distribution passe par un catalogue (winget, Microsoft Store) qui impose un installeur ;
- l'installation obligatoire freine un usage ponctuel au point qu'on renonce à une appli.
