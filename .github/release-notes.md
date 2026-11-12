**Aria {{NOM}}** {VERSION} : {{PROMESSE}}

<!-- Modèle des notes de version. À chaque version : copie ce fichier vers docs/releases/v{VERSION}.md dans la PR
     de version, remplace {VERSION} et {REPO_URL}, remplis les sections, supprime ce commentaire. Le workflow de
     release publie ce fichier tel quel. -->

## Nouveautés

- Ce qui change pour toi, une ligne par nouveauté.

## À savoir

- Ruptures, migrations, changements de licence. Supprime la section si elle est vide.

## Installation

1. Télécharge `Aria-{{NOM}}-{VERSION}.zip` ci-dessous et décompresse-le dans un dossier stable (pas Téléchargements).
2. Lance `Aria-{{NOM}}.exe`.
3. L'exe n'est pas signé : si Windows affiche « Windows a protégé votre ordinateur », clique sur « Informations
   complémentaires », puis sur « Exécuter quand même ».

## Mise à jour depuis une version précédente

Ferme l'appli, décompresse la nouvelle version à côté de l'ancienne, lance-la. Tes réglages sont dans
`%USERPROFILE%\.Aria-{{NOM}}` et sont repris tels quels. L'ancienne version peut être gardée le temps de vérifier.

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
