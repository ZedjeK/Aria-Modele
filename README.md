<!-- MODELE:debut -->
> **Ce dépôt est le modèle des applis Aria.** Il ne se lance pas tel quel.
> Crée ton appli avec « Use this template », clone-la, puis lance `.\init.ps1`
> dans le dossier du clone : le script choisit la stack, remplit le nom et se
> supprime. Ce qui suit est le squelette du README de la future appli.

<!-- MODELE:fin -->
<p align="center">
  <img src="docs/logo.png" width="160" alt="Logo d'Aria {{NOM}} : un A néon glitché avec {{ELEMENT_ORANGE}} orange">
</p>

<h1 align="center">Aria {{NOM}}</h1>

<p align="center">
  {{PROMESSE}}
</p>

## Ce qu'il fait

Ce que l'appli permet, en clair, sans jargon.

## Avant de commencer

Windows 10 ou 11, 64 bits. Droits, ports et matériel nécessaires.

## Installation

1. Télécharge `Aria-{{NOM}}-<version>.zip` depuis la page des versions.
2. Décompresse-le dans un dossier stable (pas Téléchargements).
3. Lance `Aria-{{NOM}}.exe`.

L'exe n'est pas signé : Windows peut afficher « Windows a protégé votre
ordinateur ». Clique sur « Informations complémentaires », puis sur « Exécuter
quand même ».

## Premiers pas

Le premier lancement, étape par étape.

## Au quotidien

Section optionnelle : supprime-la si elle reste vide.

## Où sont tes données

Tout est dans `%USERPROFILE%\.Aria-{{NOM}}` :

- `reglages.json` : tes réglages ;
- `journal.json` : les 500 dernières opérations ;
- `sauvegardes\` : une copie datée avant chaque modification ;
- `cache\` : fichiers temporaires de l'interface.

Supprimer ce dossier remet l'appli à neuf.

## Sécurité

- **Droits** : l'appli se lance sans droits administrateur.
- **Connexions sortantes** : la liste complète, destination par destination.
- **Secrets** : ce que l'appli garde, où, et comment c'est protégé.

Pour signaler une faille : voir [SECURITY.md](SECURITY.md).

## Limites à connaître

Ce que l'appli ne fait pas.

Projet personnel, sans audit de sécurité indépendant.

## Provenance des composants

Section à garder seulement si l'appli embarque un composant tiers. Le détail
est dans [LICENCES-TIERS.txt](LICENCES-TIERS.txt).

## Compiler soi-même

Prérequis de la stack, puis :

```powershell
.\build.ps1
```

Comment vérifier que l'exe obtenu est le bon.

## Structure

L'arborescence commentée, en dix lignes.

## Licence

MIT : voir [LICENSE](LICENSE). Les composants embarqués gardent leur propre
licence : voir [LICENCES-TIERS.txt](LICENCES-TIERS.txt).
