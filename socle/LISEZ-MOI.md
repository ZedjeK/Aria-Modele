# Socle partagé des applis Aria

Ce que toute appli Aria reçoit du dépôt modèle au lieu de le recopier (décision 0007). Le contrat qu'il tient est
[docs/CONTRAT.md](../docs/CONTRAT.md).

| Dossier | Contenu | Comment l'appli le reçoit |
| --- | --- | --- |
| `coeur/` | crate Rust `aria-coeur` | dépendance git du dépôt modèle, épinglée sur le tag `vX.Y.Z`, dans `appli/Cargo.toml` |
| `interface/` | paquet npm `aria-interface` | archive `aria-interface-X.Y.Z.tgz` jointe à la release `vX.Y.Z`, dans `interface/package.json` |

La crate et le paquet ont la version du dépôt modèle ; `tools\tester-squelettes.ps1` vérifie qu'elles sont égales.
Un tag `vX.Y.Z` lance `socle-release.yml`, qui emballe le paquet et crée la release en brouillon ; une fois la release
publiée, le workflow « Monter le socle » de chaque appli ouvre la PR qui la monte.

## Crate `aria-coeur`

Toujours là : identité, dossier de données, réglages (format versionné, lecture champ par champ), journal,
sauvegardes et restauration, secrets (DPAPI), diagnostic, démarrage avec Windows, installation, désinstallation,
mises à jour, reprise des données d'une ancienne version, dossier commun (`%USERPROFILE%\.Aria`).

Fonctions (`features`) à activer dans `appli/Cargo.toml` :

| Fonction | Ce qu'elle apporte | État |
| --- | --- | --- |
| `fenetre` | la fenêtre Tauri, le lancement, les commandes du socle, l'icône, les tâches longues ; toute appli l'active | disponible |
| `telechargement` | téléchargement vérifié : HTTPS, taille et SHA-256 contrôlés, reprise d'un téléchargement coupé | disponible |
| `moteur` | moteurs locaux partagés : versions côte à côte, plage acceptée, essai après mise à jour, registre d'usage, effacement sur demande | disponible |
| `carte-graphique` | mémoire vidéo occupée, déclarée dans le dossier commun | disponible |
| `fichiers` | fenêtres « Ouvrir », « Enregistrer » et « Choisir un dossier » de Windows, glisser-déposer (`ZoneDepot`), fichiers ouverts depuis l'Explorateur, copie avant réécriture (`fichiers::reecrire`), associations de types de fichiers réglées dans les Paramètres | disponible |
| notifications, raccourcis globaux, échanges entre applis, droits administrateur | voir le contrat, « Modules en option » | à venir |

Le démarrage avec Windows et l'icône de notification ne sont pas des fonctions : ils viennent de `en_fond` dans
l'identité de l'appli.

Les modules passent par l'API de Windows, sans greffon Tauri (décision 0011) : une appli n'ajoute que la fonction
dans `appli/Cargo.toml`. Les commandes et la capacité d'un module ne sont publiées que si sa fonction est activée
(`MODULES` dans `src/fenetre/liste.rs`).

### Fichiers et dossiers

```toml
aria-coeur = { …, features = ["fenetre", "fichiers"] }
```

- **Types de fichiers** : `Appli::new(…).types_de_fichiers(&[TypeFichier { extension: "md", description: "Document
  Markdown" }])`. Les Paramètres proposent de les associer, case par case ; rien n'est associé sans la case cochée.
- **Fenêtres de Windows** : `choisirFichiers`, `choisirDossier`, `choisirEnregistrement` dans l'interface.
- **Dépôt** : le composant `ZoneDepot` ; les fichiers ouverts depuis l'Explorateur arrivent par `fichiersOuverts()`
  au lancement, puis par l'événement `fichiers-ouverts`.
- **Chemins accordés** : une commande de l'appli qui touche un fichier de l'utilisateur vérifie
  `fenetre::fichiers::accorde(&app, &chemin)` et réécrit par `fichiers::reecrire`, qui copie d'abord l'ancien
  contenu dans `sauvegardes\`.

### Les commandes du socle

`src/fenetre/liste.rs` les liste. `build.rs` les publie à l'appli (`DEP_ARIA_COEUR_COMMANDES`), qui les autorise avec
les siennes ; les capacités des fenêtres du socle (`src/fenetre/capacite-*.json`) sont ajoutées au lancement. Une
commande ajoutée au socle s'ajoute dans `liste.rs`, dans `commandes_du_socle` (`src/fenetre/mod.rs`) et dans la
capacité de sa fenêtre ; les tests vérifient que les trois se suivent.

## Paquet `aria-interface`

Le thème, les polices, les composants (`Bouton`, `Panneau`, `SousOnglets`, `CarteChoix`, `Dialogue`, `Jauge`,
`TacheEnCours`…), la fenêtre principale (`Fenetre`), les dialogues (mise à jour, désinstallation, assistant), le
Journal (avec les sauvegardes à restaurer), les Paramètres, la proposition d'installation et le menu de l'icône.
L'appli monte sa vue par `demarrer(App)`.

## Changer le socle

Une PR dans le dépôt modèle, jamais dans une appli : une appli ne recopie ni ne modifie le code du socle. Une
fonction retirée est d'abord dépréciée dans une version mineure, puis retirée dans la version majeure suivante.
