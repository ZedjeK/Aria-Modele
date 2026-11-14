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
2. Décompresse d'abord le zip en entier : clic droit, « Extraire tout ».
<!-- PORTABLE:debut -->
   Choisis un dossier stable (pas Téléchargements).
<!-- PORTABLE:fin -->
   Ne lance pas l'exe depuis l'aperçu du zip dans l'Explorateur : Windows n'en
   extrait que l'exe, sans les fichiers livrés à côté de lui.
3. Lance `Aria-{{NOM}}.exe`.
<!-- INSTALLATION:debut -->
4. L'appli propose « Installer » ou « Lancer sans installer ».
   - **Installer** range l'exe et ses fichiers dans
     `%USERPROFILE%\.Aria-{{NOM}}\programme`, garde la version précédente, puis
     relance l'appli depuis là. La case « Créer un raccourci sur le Bureau »,
     décochée d'office, ajoute un raccourci sur ton Bureau seulement.
   - **Lancer sans installer** utilise l'exe là où il est.

   La case « Ne plus me proposer » coupe la question ; tu peux la réactiver dans
   les paramètres. `Aria-{{NOM}}.exe --portable` lance l'appli sans rien
   proposer.
<!-- INSTALLATION:fin -->

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
<!-- INSTALLATION:debut -->
- `programme\` : l'appli installée et sa version précédente.
<!-- INSTALLATION:fin -->

Supprimer ce dossier remet l'appli à neuf.

## Sécurité

- **Droits** : l'appli se lance sans droits administrateur.
- **Connexions sortantes** : la liste complète, destination par destination.
- **Secrets** : ce que l'appli garde, où, comment c'est protégé, et contre quoi
  cette protection ne joue pas.

Pour signaler une faille : voir [SECURITY.md](SECURITY.md).

## Limites à connaître

Ce que l'appli ne fait pas.

Mises à jour : l'empreinte SHA-256 vérifie que le téléchargement est intact, pas
d'où il vient. La provenance repose sur le compte GitHub de l'auteur, sur les
releases immuables (une version publiée ne change plus) et sur l'attestation
jointe à chaque zip. Si le compte de l'auteur était compromis, une version
piégée pourrait quand même être proposée.

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
