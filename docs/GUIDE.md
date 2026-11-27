# Guide des applis Aria

Ce qui est pareil dans toutes les applis Aria : les installer, les mettre à jour, retrouver ses données, revenir en
arrière, les désinstaller. Ce qui est propre à une appli est dans son README.

Dans ce guide, `<Nom>` est le nom de l'appli : `Aria-Atlas` pour Aria Atlas, par exemple.

## Installer

1. Télécharge le zip de la version depuis la page des versions du dépôt de l'appli.
2. Décompresse-le en entier : clic droit, « Extraire tout ». Lancé depuis l'aperçu du zip dans l'Explorateur, l'exe
   ne s'installe pas : il te demande de décompresser d'abord.
3. Lance l'exe du dossier obtenu et choisis « Installer ». L'appli se range dans `%USERPROFILE%\.Aria-<Nom>\programme`
   et se relance de là. « Annuler » la ferme sans rien installer.
4. Au premier lancement, un assistant présente l'appli et règle l'essentiel. Il se rouvre depuis les Paramètres.

Si Windows affiche « Windows a protégé votre ordinateur », l'exe n'est pas encore signé : « Informations
complémentaires », puis « Exécuter quand même ». L'empreinte du zip (`Get-FileHash`) doit être celle du fichier
`.sha256` publié avec la version.

## Mettre à jour

Au lancement, l'appli regarde s'il existe une nouvelle version, si la case des Paramètres le permet. Un bandeau le dit ;
rien ne s'installe sans ton clic. La version remplacée est gardée dans `sauvegardes\versions\`.

## Où sont tes données

Tout ce qu'une appli garde est dans `%USERPROFILE%\.Aria-<Nom>` :

| Dossier ou fichier | Contenu |
| --- | --- |
| `reglages.json` | tes réglages |
| `journal.json` | les 500 dernières opérations |
| `sauvegardes\` | une copie datée avant chaque modification, et les versions précédentes de l'appli |
| `secrets\` | ce que l'appli garde de secret, chiffré pour ton compte Windows |
| `cache\` | ce que l'appli recrée au besoin |
| `programme\` | l'appli installée |

Les applis qui utilisent des modèles ou des moteurs locaux (IA, génération d'images) les partagent dans
`%USERPROFILE%\.Aria` : un modèle n'est téléchargé et gardé qu'une fois pour toute la famille. Ce dossier ne contient
ni réglage, ni secret, ni donnée personnelle. Une appli qui le propose peut le placer sur un autre disque, une seule
fois pour toutes les applis : l'ancien contenu reste en place, à supprimer toi-même une fois le déplacement vérifié.
Un modèle ou un moteur ne s'efface que sur ta demande, après que l'appli t'a dit quelles autres applis s'en servent.

## Revenir en arrière

- **Un fichier** : page « Journal », panneau « Sauvegardes », « Restaurer ». L'état actuel est d'abord sauvegardé :
  rien n'est perdu.
- **L'appli** : quitte-la, puis copie les fichiers de `sauvegardes\versions\<version>\` à la place de ceux de
  `programme\`.

## Signaler un problème

Page « Journal », « Exporter le diagnostic » : le texte copié (version, Windows, réglages, dernières lignes du
journal) ne contient aucun secret, et le chemin de ton profil y devient `%USERPROFILE%`. Relis-le, puis colle-le dans
le formulaire de bug du dépôt de l'appli. Aucune appli Aria n'envoie rien d'elle-même.

## Désinstaller

Paramètres, « Désinstaller » : l'appli retire le démarrage avec Windows (s'il existe), le raccourci du Bureau s'il
pointe vers elle, puis `programme\` une fois fermée. Tes données restent dans `%USERPROFILE%\.Aria-<Nom>` ; pour tout
effacer, supprime ce dossier une fois l'appli fermée.
