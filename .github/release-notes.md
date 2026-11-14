**Aria {{NOM}}** {VERSION} : {{PROMESSE}}

<!-- Modèle des notes de version. À chaque version : copie ce fichier vers docs/releases/v{VERSION}.md dans la PR
     de version, remplace {VERSION} et {REPO_URL}, remplis les sections, supprime ce commentaire. Le workflow de
     release publie ce fichier tel quel. -->

## Nouveautés

- Ce qui change pour toi, une ligne par nouveauté.

## À savoir

- Ruptures, migrations, changements de licence. Supprime la section si elle est vide.

## Installation

1. Télécharge `Aria-{{NOM}}-{VERSION}.zip` ci-dessous.
2. Décompresse d'abord le zip en entier (clic droit, « Extraire tout »).
<!-- PORTABLE:debut -->
   Choisis un dossier stable (pas Téléchargements).
<!-- PORTABLE:fin -->
   Lancé depuis l'aperçu du zip dans l'Explorateur, l'exe démarre sans les fichiers livrés à côté de lui.
3. Lance `Aria-{{NOM}}.exe`.
<!-- INSTALLATION:debut -->
   Choisis « Installer » pour le ranger dans `%USERPROFILE%\.Aria-{{NOM}}\programme`, ou « Lancer sans installer ».
<!-- INSTALLATION:fin -->
4. L'exe n'est pas signé : si Windows affiche « Windows a protégé votre ordinateur », clique sur « Informations
   complémentaires », puis sur « Exécuter quand même ».

## Mise à jour depuis une version précédente

L'appli te propose elle-même la nouvelle version : elle remplace l'exe là où il est et garde la précédente.

Pour le faire à la main : ferme l'appli, décompresse le nouveau zip en entier, puis
<!-- INSTALLATION:debut -->
lance son exe et choisis « Installer » : la version installée est remplacée, la précédente est gardée.
<!-- INSTALLATION:fin -->
<!-- PORTABLE:debut -->
lance son exe depuis un dossier à côté de l'ancienne version, que tu peux garder le temps de vérifier.
<!-- PORTABLE:fin -->

Tes réglages sont dans `%USERPROFILE%\.Aria-{{NOM}}` et sont repris tels quels.

## Contenu

| Composant | Version | Source |
| --- | --- | --- |
| Aria {{NOM}} | {VERSION} | [ce dépôt]({REPO_URL}) |

Le détail des composants embarqués est dans `LICENCES-TIERS.txt`, joint au zip.

## Comment vérifier ce que tu as téléchargé

L'empreinte du zip est dans `Aria-{{NOM}}-{VERSION}.zip.sha256`. Dans PowerShell :

```powershell
(Get-FileHash .\Aria-{{NOM}}-{VERSION}.zip -Algorithm SHA256).Hash
```

L'empreinte seule ne prouve pas d'où vient le fichier. Pour vérifier qu'il a bien été construit par ce dépôt :

```powershell
gh attestation verify .\Aria-{{NOM}}-{VERSION}.zip --repo ZedjeK/Aria-{{NOM}}
```
