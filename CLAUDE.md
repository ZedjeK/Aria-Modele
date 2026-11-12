<!-- MODELE:debut -->
> **Tu es dans le dépôt modèle, pas dans une appli.** Ici on maintient le socle
> commun, les squelettes de `stacks/` et `init.ps1`. Les marqueurs `{{...}}` et
> `<<STACK:...>>` sont voulus : ne les remplace pas à la main. Avant de proposer
> un commit, lance `.\tools\tester.ps1`, qui rejoue `init.ps1` sur une stack
> d'essai. Le texte ci-dessous est celui que recevra chaque appli.

<!-- MODELE:fin -->
# Aria {{NOM}}

Appli Windows de la famille Aria. Ce fichier fixe ce que toute appli Aria doit
respecter. En cas de doute, ces règles passent avant une préférence de style.

## Fiche projet

| | |
| --- | --- |
| Nom | Aria {{NOM}} (dépôt et exe : `Aria-{{NOM}}`) |
| Promesse | {{PROMESSE}} |
| Élément orange du logo | {{ELEMENT_ORANGE}} |
| Fond animé | à renseigner : motif, ce qui est aléatoire, nombre par défaut |
| Stack | {{STACK}} |
| Droits administrateur | non, sauf mention ici : à renseigner |
| Démarrage automatique | {{DEMARRAGE_AUTO}} |
| Port local (interface web) | {{PORT}} |
| Données de l'utilisateur | `%USERPROFILE%\.Aria-{{NOM}}` ; secrètes : à renseigner |
| Connexions sortantes | à renseigner : liste complète, destination par destination |
| Composants tiers embarqués | à renseigner |
| Réussite de la v0.1 | à renseigner : un critère mesurable |

Les lignes « à renseigner » se complètent avant la première ligne de code.

## Principes

1. Tout en local. Aucune télémétrie, rien n'est envoyé à l'auteur.
2. Rien à installer : exe portable, ni installeur ni service laissé en place.
3. Rien sans accord, et tout se restaure : récapitulatif, sauvegarde avant
   modification, annulation. Une restauration ne supprime rien.
4. Le README ne promet que ce que le code tient (section « Limites à connaître »).
5. Moindre privilège : pas de droits administrateur si on peut s'en passer.
6. Français et tutoiement : interface, README, erreurs, commentaires, commits.

Toute question de paramétrage se tranche en restant fidèle au contrat ci-dessous.

## Contrat de comportement

- Données, réglages, journal, sauvegardes, cache de l'interface : tout dans
  `%USERPROFILE%\.Aria-{{NOM}}`. Rien dans `%APPDATA%`, `%LOCALAPPDATA%` ni `%TEMP%`.
- Fichiers du dossier : `reglages.json`, `journal.json` (500 dernières
  opérations), `sauvegardes\`, `cache\`.
- Démarrage automatique : désactivé par défaut, case dans les réglages et dans le
  menu de l'icône, lancement avec `--silencieux`, état relu du système.
- Icône de notification (appli en fond) : Ouvrir, Démarrage automatique, Quitter.
- Instance unique : relancer l'exe ramène la fenêtre existante.
- Mises à jour : lire `releases/latest`, proposer, vérifier le SHA-256, garder
  l'ancienne version.
- Réseau local : `127.0.0.1` seulement, jamais `0.0.0.0`.
- Appli à extension de navigateur : l'exe est l'appli. Il range l'extension dans
  `%USERPROFILE%\.Aria-{{NOM}}\extension` et l'y met à jour.

## Identité visuelle

Logo : « A » néon glitché (blanc, cyan, bleu) dans un cadre biseauté, avec un
élément orange. Polices : Space Grotesk (interface), IBM Plex Mono (code),
embarquées. Palette : fond `#030812`, panneau `#06101f`, texte `#ddebff`, texte
secondaire `#93a8c8`, cyan `#7df9ff`, bleu `#5c8bff`, orange `#ff8a1f`,
rouge `#ff5c7a`, ambre `#ffd166`, ligne `#1a2d4d`. Jamais Cascadia Mono.

Fond animé : il dit ce que fait l'appli. Zone principale seulement, jamais sous
le menu latéral ; panneaux opaques ; couleurs de la palette ; éléments tirés au
hasard à chaque passage. Réglages obligatoires, enregistrés dans `reglages.json` :
`fond.actif`, `fond.vitesse` (lente, normale, rapide), `fond.minimum` et
`fond.maximum`. En pause quand la fenêtre est cachée, coupé si Windows demande
moins d'animations.

## Façon de travailler

- Compiler avec `.\build.ps1`, tester avec `.\tools\tester.ps1`, avant toute
  proposition de commit.
- Jamais de commit directement sur `main` : une branche courte et une PR,
  premier commit compris.
- Pousser un tag et publier une release : toujours l'auteur du dépôt, jamais la
  session. La session prépare la version et les notes, et s'arrête au brouillon.
- Pas de commit ni de push sans demande explicite. Un commit par sujet, ajout
  des fichiers par leur nom.
- La version vit à un seul endroit. `.\tools\version.ps1` l'affiche, et le tag
  `vX.Y.Z` doit lui correspondre.
- Le job de CI s'appelle `tests` et ne se renomme jamais : le ruleset l'exige.
- Aucun secret dans le dépôt, les workflows ou les journaux. Actions GitHub
  épinglées par SHA.
- Un bug hors sujet repéré en chemin devient une issue, pas un correctif en douce.
- Textes publiés (commit, PR, release) : voix neutre, en français.

## À tenir à jour

`README.md`, `LICENCES-TIERS.txt`, `CHANGELOG.md`, `docs/releases/<tag>.md`.

## Interdits

Télémétrie · écoute sur `0.0.0.0` · écriture hors du dossier de données · service
Windows laissé en place · raccourci créé sans accord · élévation de droits ·
suppression d'un fichier de l'utilisateur sans copie préalable.
