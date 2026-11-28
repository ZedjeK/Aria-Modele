# Fiche projet : Aria {{NOM}}

Ce que cette appli a de propre. À relire avant toute modification, et à tenir à
jour quand l'appli change.

| | |
| --- | --- |
| Nom | Aria {{NOM}} (dépôt et exe : `Aria-{{NOM}}`) |
| Promesse | {{PROMESSE}} |
| Élément orange du logo | {{ELEMENT_ORANGE}} |
| Fond animé | à renseigner : motif, ce qui est aléatoire, nombre par défaut |
| Stack | {{STACK}} |
| Socle | la version épinglée dans `appli/Cargo.toml` et `interface/package.json` |
| Modules en option | aucun, sauf mention ici : à renseigner (fonctions de `aria-coeur` activées) |
| Moteurs et modèles | aucun, sauf mention ici : à renseigner (moteur, plage de versions acceptée, modèles) |
| Droits administrateur | non, sauf mention ici : à renseigner |
| Démarrage automatique | {{DEMARRAGE_AUTO}} |
| Installation | dans `%USERPROFILE%\.Aria-{{NOM}}\programme`, comme toute appli Aria (décision 0008) |
| Port local (interface web) | {{PORT}} |
| Données de l'utilisateur | `%USERPROFILE%\.Aria-{{NOM}}` ; secrètes : à renseigner |
| Écritures hors du dossier de données | à renseigner : démarrage automatique, raccourci du Bureau, dossier commun `%USERPROFILE%\.Aria` s'il sert, associations de fichiers (`HKCU\Software\Classes`, module `fichiers`), inscription aux notifications (`HKCU\Software\Classes\AppUserModelId`, module `notifications`), et ce que l'appli modifie |
| Données reprises d'une ancienne version | aucune, sauf mention ici : à renseigner (ancien dossier, ce qui est repris) |
| Connexions sortantes | à renseigner : liste complète, destination par destination (les mises à jour intégrées en ajoutent deux : `api.github.com` pour chercher une nouvelle version, et `github.com` avec son hôte de téléchargement pour le zip) |
| Composants tiers embarqués | à renseigner |
| Réussite de la v0.1 | à renseigner : un critère mesurable |
| Prochaine version : objectif | à renseigner : ce que la prochaine version doit apporter, en une phrase |
| Non-objectifs | à renseigner : ce que l'appli ne fera pas, pour couper court aux demandes hors sujet |
| Plateformes | à renseigner : Windows 11 et Windows 10 22H2 (contrat), ou moins |
| Budget | à renseigner : taille de l'exe (Mo, relevée dans BUILDINFO.txt), mémoire au repos et délai d'ouverture (relevés dans la recette) |

Les lignes « à renseigner » se complètent avant la première ligne de code.
