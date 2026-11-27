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
2. Décompresse d'abord le zip en entier (clic droit, « Extraire tout ») : lancé depuis l'aperçu du zip dans
   l'Explorateur, l'exe ne s'installe pas et te demande de décompresser d'abord.
3. Lance `Aria-{{NOM}}.exe` depuis le dossier obtenu, et choisis « Installer » : l'appli se range dans
   `%USERPROFILE%\.Aria-{{NOM}}\programme` et se relance de là. « Annuler » la ferme sans rien installer.
4. Si Windows affiche « Windows a protégé votre ordinateur » (exe pas encore signé), clique sur « Informations
   complémentaires », puis sur « Exécuter quand même ».

## Mise à jour depuis une version précédente

L'appli te propose elle-même la nouvelle version : elle remplace l'exe installé et garde la précédente.

Pour le faire à la main : ferme l'appli, décompresse le nouveau zip en entier, puis lance son exe et choisis
« Installer » : la version installée est remplacée, la précédente est gardée.

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
