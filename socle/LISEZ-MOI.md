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
| notifications, raccourcis globaux, fichiers et associations, échanges entre applis, droits administrateur | voir le contrat, « Modules en option » | à venir |

Le démarrage avec Windows et l'icône de notification ne sont pas des fonctions : ils viennent de `en_fond` dans
l'identité de l'appli.

Les modules qui passent par un greffon Tauri (notifications, raccourcis, fenêtres « Ouvrir » et « Enregistrer »)
demandent que le greffon soit une dépendance directe de l'appli : Tauri ne lit les autorisations d'un greffon que
dans les dépendances directes. Ils restent donc à concevoir avec la première appli qui en a besoin.

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
