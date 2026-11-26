# 0003 : les secrets chiffrés par DPAPI

Date : 2026-11-26.

## Contexte

Certaines applis gardent des secrets : jetons d'API, mots de passe. Ils ne doivent jamais être en clair dans les
réglages, le journal ou les sauvegardes, et l'utilisateur ne doit pas avoir à retenir un mot de passe de plus.

## Décision

Chaque secret est chiffré par la protection des données de Windows (DPAPI), liée au compte de l'utilisateur, avec une
entropie liée au nom de l'appli, et rangé dans un fichier à part dans `secrets\` du dossier de données
(`coeur/src/secrets.rs`).

## Conséquences

- Aucun mot de passe maître ; le déchiffrement est transparent pour l'utilisateur sous son compte.
- Une copie du dossier de données sur un autre compte ou un autre PC ne permet pas de lire les secrets.
- Un programme lancé sous le même compte Windows peut les déchiffrer : le README de chaque appli le dit.
- Une sauvegarde restaurée sur un autre PC ou un autre compte perd ses secrets : ils sont à ressaisir.

## À revoir si

- un secret doit suivre l'utilisateur d'une machine à l'autre ;
- une appli doit se protéger d'un programme qui tourne sous le même compte (Gestionnaire d'identification de Windows
  avec confirmation, Windows Hello) ;
- Windows remplace ou restreint DPAPI.
