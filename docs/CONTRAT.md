# Contrat des applis Aria

Les règles que toute appli de la famille respecte. Ce fichier fait foi : en cas de doute, il passe avant une
préférence de style, et le code, le README et la fiche projet s'y conforment.

Il est le même dans toutes les applis : il fait partie du socle commun (`tools/socle.txt`). Pour le changer, passe
par le dépôt modèle.

## Principes

1. **Tout en local.** Aucune télémétrie, rien n'est envoyé à l'auteur.
2. **Rien à installer** : exe portable, ni installeur ni service laissé en place. L'exe peut, avec accord, se ranger
   lui-même (voir « Installation par l'exe ») ; il reste utilisable là où il est.
3. **Rien sans accord, et tout se restaure** : récapitulatif, sauvegarde avant modification, annulation. Une
   restauration ne supprime rien.
4. **Le README ne promet que ce que le code tient** (section « Limites à connaître »).
5. **Moindre privilège** : pas de droits administrateur si on peut s'en passer.
6. **Français et tutoiement** : interface, README, erreurs, commentaires, commits.

**Sans dépendance** : l'appli ne demande rien à installer. Seule exception : WebView2 Evergreen, le moteur
d'affichage de Windows, présent d'office sur Windows 11 et sur Windows 10 22H2 à jour. S'il manque, l'appli le dit et
donne le lien de Microsoft. Le runtime Visual C++ est lié dans l'exe, et les tests le vérifient. Aucune bibliothèque
n'est extraite dans `%TEMP%`.

**Plateformes** : Windows 11 ; Windows 10 22H2 tant que WebView2 y reçoit des mises à jour (au moins jusqu'en octobre
2028).

Toute question de paramétrage se tranche en restant fidèle au contrat ci-dessous.

## Fiche projet

Chaque appli a sa fiche, `docs/FICHE.md` : ce qu'elle a de propre. Tu la lis avant toute modification et tu la
tiens à jour quand l'appli change.

- **Identité** : nom, promesse, élément orange du logo, fond animé (motif, ce qui est aléatoire, nombre par défaut).
- **Technique** : stack, droits administrateur, démarrage automatique, installation par l'exe (oui ou non), port
  local.
- **Données** : données de l'utilisateur et données secrètes, écritures hors du dossier de données (dont le raccourci
  du Bureau).
- **Ouverture** : connexions sortantes (liste complète, destination par destination), composants tiers embarqués.
- **Réussite** : critère mesurable de réussite de la v0.1.

Les lignes « à renseigner » se complètent **avant la première ligne de code**.

## Contrat de comportement

### Données et secrets

- **Dossier de données** : données, réglages, journal, sauvegardes, cache de l'interface, tout dans
  `%USERPROFILE%\.Aria-<Nom>`. Rien dans `%APPDATA%`, `%LOCALAPPDATA%` ni `%TEMP%`. Un exe en un seul fichier
  n'extrait rien dans `%TEMP%` à son lancement : ses bibliothèques natives sont livrées à côté de lui.
- **Hors de ce dossier**, l'appli n'écrit que ce que sa fiche projet annonce : l'entrée de démarrage automatique, et
  ce qui fait son travail, toujours avec accord et sauvegarde préalable.
- **Fichiers du dossier** :

  | Fichier | Contenu |
  | --- | --- |
  | `reglages.json` | les réglages |
  | `journal.json` | les 500 dernières opérations |
  | `sauvegardes\` | les sauvegardes ; `sauvegardes\versions\` garde la version précédente du programme |
  | `secrets\` | les secrets, chiffrés par DPAPI, un fichier par secret (si l'appli en garde) |
  | `cache\` | le cache de l'interface |
  | `programme\` | pour une appli de bureau qui s'installe (voir « Installation par l'exe ») |

- **Secrets** (jetons, mots de passe, clés) : jamais en clair dans `reglages.json`, le journal ni les sauvegardes. Au
  minimum, ils sont chiffrés par la protection des données de Windows (DPAPI), liée au compte de l'utilisateur, et
  restent dans le dossier de données. Le README dit contre quoi cette protection ne joue pas : un programme lancé
  sous le même compte. DPAPI est en place dans le squelette (`coeur/src/secrets.rs`, entropie liée au nom de
  l'appli).

### Lancement et présence

- **Démarrage automatique** : désactivé par défaut, case dans les réglages et dans le menu de l'icône, lancement avec
  `--silencieux`, état relu du système.
- **Icône de notification** (appli en fond) : Ouvrir, Démarrage automatique, Quitter.
- **Instance unique** : relancer l'exe ramène la fenêtre existante.

### Installation et mises à jour

- **Installation par l'exe** (appli de bureau) :
  - **Proposition** : lancé hors de son dossier (zip décompressé), l'exe propose « Installer » ou « Lancer sans
    installer ».
  - **Installer** range l'exe, son banc d'essai éventuel et les licences dans `%USERPROFILE%\.Aria-<Nom>\programme`,
    garde la version précédente, puis relance depuis là.
  - **Raccourci** : une case « Créer un raccourci sur le Bureau », décochée d'office : Bureau de l'utilisateur
    seulement, jamais le Bureau public, cible stable d'une version à l'autre ; un raccourci déjà présent est proposé
    en mise à jour, jamais remplacé en silence.
  - **Droits du dossier** : une appli qui tourne élevée vérifie, avant de copier, que le dossier des données (et
    `programme\`) n'est modifiable que par le système et les administrateurs, propriétaire compris, et refuse sinon :
    on ne lance pas un exe élevé depuis un dossier qu'un programme sans droits peut modifier. Une appli lancée avec
    les droits de l'utilisateur ne fait pas ce contrôle : elle ne pourrait pas créer un tel dossier, et elle n'a pas
    de droits élevés à protéger.
  - **Couper la proposition** : case « Ne plus me proposer », réactivable dans les paramètres ; `--portable` la
    saute.
  - **Marque Internet** : les fichiers installés n'héritent pas de la marque « téléchargé depuis Internet » :
    l'utilisateur a lancé l'exe et accepté l'installation.
  - **Fin de l'installation** : la fenêtre se ferme d'elle-même une fois la copie finie, et l'appli se relance depuis
    son dossier ; le mode « occupé » qui interdit de fermer pendant la copie est levé avant. Une installation se teste
    par un vrai clic sur « Installer » avant la release, pas seulement par les tests du noyau.
  - **Décompresser d'abord** : le zip se décompresse en entier (clic droit, « Extraire tout ») avant de lancer l'exe.
    Lancé depuis l'aperçu du zip de l'Explorateur, l'exe est sans son banc d'essai, et seul l'exe est copié. Le README
    et les notes de chaque version le disent en toutes lettres (« décompresse d'abord le zip »), avec la raison, et la
    fenêtre signale le banc d'essai manquant.
  - **Mises à jour intégrées** : elles remplacent l'exe là où il est installé.
- **Mises à jour** : obligatoires pour toute appli, portable comprise. Lire `releases/latest` du dépôt de l'appli,
  proposer, ne télécharger que depuis cette release sur `github.com` en HTTPS, vérifier le SHA-256, garder l'ancienne
  version, ne rien installer sans accord. L'empreinte prouve que le téléchargement est intact, pas d'où il vient : la
  provenance repose sur le compte de l'auteur, les releases immuables et l'attestation jointe, et le README le dit
  dans « Limites à connaître ».
- **Désinstallation** : un bouton « Désinstaller » dans les Paramètres retire tout ce que l'appli a écrit hors de
  son dossier de données (démarrage automatique, raccourci du Bureau s'il pointe vers elle), puis supprime
  `programme\` une fois l'appli fermée. Les données restent, et l'appli dit où elles sont : les supprimer est un
  geste de l'utilisateur.

### Réseau et droits

- **Réseau local** : la boucle locale seulement, `127.0.0.1`, et `::1` si l'appli en a besoin ; jamais `0.0.0.0` ni
  `::`. Une interface web locale :
  - refuse toute requête dont l'en-tête `Host` n'est pas `127.0.0.1:<port>`, `localhost:<port>` (ou `[::1]:<port>`
    si elle écoute sur `::1`) ;
  - refuse toute requête dont l'`Origin`, quand il est présent, n'est pas le sien ;
  - ne fait passer aucune action par un GET, n'ouvre aucun CORS ;
  - exige à chaque requête un jeton tiré au hasard à chaque lancement : l'interface ne s'ouvre que par l'appli.

  Ports déjà pris : `docs/PORTS.md` du dépôt modèle.
- **Droits administrateur** : seulement si la fiche projet le dit, et autant que possible pour la seule opération qui
  l'exige. Une appli élevée traite son dossier de données comme une entrée non fiable : elle n'y charge ni code ni
  commande, et valide ce qu'elle y lit.

### Cas particuliers

- **Appli à extension de navigateur** : l'exe est l'appli. Il range l'extension dans
  `%USERPROFILE%\.Aria-<Nom>\extension` et l'y met à jour.
- **Signature de code** : pas prévue, elle n'est pas gratuite.
- **Appli serveur du réseau local** (exception, par exemple un coffre joint par les téléphones) : seulement si la
  fiche projet le dit, en TLS, jamais exposée sur Internet, et son port est noté dans `docs/PORTS.md` du dépôt
  modèle.
- **Squelette incomplet** : la clé `nonFournis` de `_socle/stack.json` dit ce que la stack ne tient pas encore
  (`installation`, `demarrageAuto`, `misesAJour`) ; `init.ps1` refuse les deux premiers et signale le troisième. Une
  appli créée avec un tel squelette le dit dans sa fiche et dans « Limites à connaître », et ne fait pas de release
  publique tant que les mises à jour manquent.

## Identité visuelle

### Bases

- **Logo** : « A » néon glitché (blanc, cyan, bleu) dans un cadre biseauté, avec un élément orange propre à l'appli.
- **Source du logo** : `docs/logo.svg`, seule source ; `tools\icone.ps1` en tire l'icône Windows (7 tailles, de 16 à
  256 px), `docs/logo.png` et le logo du menu latéral. Tant que l'élément orange n'est pas dessiné, le logo reste
  sans, pour qu'un oubli se voie.
- **Polices** : Space Grotesk (interface), IBM Plex Mono (code), embarquées. Jamais Cascadia Mono.
- **Palette** :

  | Rôle | Valeur |
  | --- | --- |
  | fond | `#030812` |
  | panneau | `#06101f` |
  | texte | `#ddebff` |
  | texte secondaire | `#93a8c8` |
  | cyan | `#7df9ff` |
  | bleu | `#5c8bff` |
  | orange | `#ff8a1f` |
  | rouge | `#ff5c7a` |
  | ambre | `#ffd166` |
  | ligne | `#1a2d4d` |

- **Couleurs** (sens) :

  | Couleur | Sens |
  | --- | --- |
  | cyan | actif ou choisi |
  | orange | action principale ou alerte |
  | rouge | erreur |
  | bleu | information |

### Fond animé

- **Il dit ce que fait l'appli.** Zone principale seulement, jamais sous le menu latéral ; panneaux opaques ; couleurs
  de la palette ; éléments tirés au hasard à chaque passage. Le fond de chaque appli est décrit dans sa fiche projet.
- **Réglages obligatoires**, enregistrés dans `reglages.json` : `fond.actif`, `fond.vitesse` (lente, normale,
  rapide), `fond.minimum` et `fond.maximum`.
- **En pause** quand la fenêtre est cachée, **coupé** si Windows demande moins d'animations.

### Mise en page et formes

- **Référence** : Aria Studio et Aria Cockpit. Une forme ou une disposition nouvelle part de leur façon de faire, ou
  de ce que le squelette rust-tauri du dépôt modèle en a repris (`interface/src/lib/theme.css`,
  `interface/src/lib/composants/`, les dialogues).
- **Fenêtre** : sans cadre de Windows ; la barre de titre est celle de l'appli (bande à attraper pour déplacer la
  fenêtre, réduire, agrandir et fermer seuls en haut à droite).
- **Menu latéral**, à gauche, opaque : le logo de la famille (A néon, élément orange de l'appli), ARIA et le nom de
  l'appli en orange ; les pages groupées en familles, chaque famille titrée en cyan avec un filet ; l'entrée active
  marquée d'une barre cyan à gauche ; pour une appli en arrière-plan, « Quitter » en rouge sous un filet ; la version
  en bas.
- **Haut de la zone principale** : le chemin de la page (famille / page) à gauche, l'état en direct à droite (une
  pastille par élément suivi : point de couleur, nom, détail) ; dessous, un bandeau teinté d'orange quand une mise à
  jour est disponible.
- **Décor** : nuit profonde, grille fine, halo bleu au centre et halo orange en bas à droite, sous le fond animé ;
  largeur des pages limitée.
- **Dialogues** : posés sur la page (voile sombre, cadre à coins coupés), pas dans une fenêtre à part, sauf la
  proposition d'installation, qui vient avant la fenêtre de l'appli.
- **Pages de l'appli elle-même** : « Journal » et « Paramètres » (démarrage, installation, mises à jour, affichage)
  dans leur propre famille, à part du travail de l'appli.
- **Sous-onglets** : en haut de la page, sous son titre.
- **Formes**, une par sens :

  | Forme | Sens |
  | --- | --- |
  | en biais | un bouton qui agit |
  | encoche (coin haut droit coupé) | un sous-onglet ; actif, contour cyan et barre épaisse dessous |
  | carte à onglet | un choix exclusif qui se coche (un seul parmi plusieurs, comme un bouton radio) ; le nom est dans l'onglet, qui se remplit en cyan quand il est choisi. Un choix multiple se fait avec des panneaux à coins coupés munis d'une case |
  | coins coupés | un panneau |

- **Boutons** : même forme, même taille.

  | Style | Usage |
  | --- | --- |
  | plein orange | l'action principale, une seule par page |
  | contour cyan | les actions secondaires |
  | contour discret | annuler ou fermer |

- **Casse du texte des boutons** : les boutons sont en majuscules par la feuille de style. Leur texte s'écrit en casse
  normale dans le code et dans les messages, et ne se met jamais en majuscules à la main.
- **Liseré** : tous les panneaux ont un liseré cyan fin, sans lueur intérieure. Le liseré accentué (épais, titre en
  cyan) marque le panneau sur lequel agit le bouton orange de la page, ou ce qu'il produit. **Un seul par page ;
  aucun sur une page sans bouton orange.**
- **Commentaire d'onglet** : sous les sous-onglets, un encadré bleu « À savoir » donne le commentaire de l'onglet
  actif.
- **Fenêtre de dialogue** (installation, mise à jour, assistant) : fond de l'appli, titre en cyan en haut, encadré
  « À savoir » s'il y a un commentaire, puis les panneaux. La fenêtre elle-même n'est jamais accentuée : le liseré
  accentué va au panneau sur lequel agit le bouton orange. En bas, la ligne d'état, puis les boutons alignés à
  droite, l'orange en dernier ; « Plus tard » ou « Annuler » en contour discret dans cette rangée. La fenêtre posée
  sur la page a un cadre à coins coupés au liseré fin. Modèle : `interface/src/Installation.svelte` et
  `interface/src/dialogues/MiseAJour.svelte` du squelette rust-tauri.
- **Assistant à étapes** : les étapes en encoches, comme les sous-onglets mais non cliquables, avec une coche sur
  celles qui sont faites. L'encadré « À savoir » donne le commentaire de l'étape. Chaque étape a un panneau accentué.
  Le bouton orange va à l'action à faire maintenant (« Installer » tant qu'un élément obligatoire manque), puis
  revient sur « Suivant » ; jamais deux à la fois.
- **Mesure pas encore faite** : jauge vide et « à tester », jamais un chiffre supposé.

### Icône et menu

- **Icône de notification** : le logo de l'appli. Délavée quand rien n'est actif, pleine quand quelque chose l'est.
  Une seule pastille à la fois, par ordre de priorité :

  | Pastille | Sens |
  | --- | --- |
  | orange | à regarder |
  | cyan | l'appli agit en ce moment |
  | bleue | information, par exemple une mise à jour disponible |

  L'infobulle dit la même chose en une ligne.
- **Menu de l'icône** : un menu Windows aux couleurs de l'appli, en blocs titrés.
  - **Ossature commune** : « Ouvrir » en gras, « État » (du texte, avec un point de couleur), puis en bas « Démarrer
    avec Windows » avec sa case et « Quitter ».
  - **Entre les deux**, les blocs propres à l'appli : « Agir sans ouvrir » et « Aller à ».
  - « Voir la mise à jour » n'apparaît que si une version est disponible.
- **Sous-menus** : en volet à côté de leur ligne. Chaque choix a un bouton radio, son nom, un détail, et « EN PLACE »
  sur celui qui tourne.

## Interface commune des dépôts

| Commande | Rôle |
| --- | --- |
| `.\build.ps1` | compile (sortie dans `.work\sortie\`) |
| `.\tools\tester.ps1` | teste |
| `.\tools\version.ps1` | affiche la version |
| `.\tools\verifier-release.ps1 -Tag vX.Y.Z` | vérifie, avant le tag, les notes, la fiche et les licences |
| `.\tools\proteger-depot.ps1` | pose et relit les protections GitHub du dépôt |

- Les deux premiers **se lancent avant tout commit**.
- `tools\empaqueter.ps1` écrit `fichiers.txt` dans le zip : la liste exacte de ce que livre la version.
- La version vit à un seul endroit ; le tag `vX.Y.Z` doit lui correspondre.
- Le job de CI s'appelle `tests` et **ne se renomme jamais** : le ruleset l'exige.
- **À tenir à jour** : `README.md`, `docs/FICHE.md`, `LICENCES-TIERS.txt`, `CHANGELOG.md`, `docs/releases/<tag>.md`.
- **Branches, commits, PR, tags, releases** : voir `CONTRIBUTING.md`, sans exception pour les applis.

### Release d'une appli

La chaîne générique (tag, workflow, brouillon, publication) est décrite dans `CONTRIBUTING.md` ; voici ce que toute
appli y ajoute.

| Point | Règle |
| --- | --- |
| Version | à un seul endroit, lue par `.\tools\version.ps1` ; le tag `vX.Y.Z` lui est égal |
| Notes | `docs/releases/<tag>.md` obligatoire et complet, avec « décompresse d'abord le zip » ; `.\tools\verifier-release.ps1 -Tag vX.Y.Z` vérifie avant le tag les notes, la fiche et les licences |
| `release.yml` | refait le contrôle des notes, refuse un tag dont le commit n'est pas dans la branche par défaut, construit, joint l'empreinte et l'attestation de provenance, s'arrête au brouillon |
| Essai à blanc | `gh workflow run Release --ref main` : build, zip et contrôle des notes, sans publication |
| Zip | contient `fichiers.txt`, la liste exacte que copient l'installation et les mises à jour |
| Tags `v*` | verrouillés par un ruleset ; un tag poussé par erreur suit le retour arrière décrit dans `CONTRIBUTING.md` |
| Publication | annonce la version à tous les utilisateurs (l'appli lit `releases/latest`) : une prerelease d'abord pour tester sans prévenir |
| Protections du dépôt | posées et relues par `.\tools\proteger-depot.ps1`, qui garde un état d'avant daté |

## Interdits

### Réseau et données

- Télémétrie, ou une connexion sortante absente de la fiche projet.
- Écoute ailleurs que sur la boucle locale (`127.0.0.1`, `::1`), hors appli serveur annoncée par sa fiche.
- Un secret en clair dans le dossier de données, le dépôt, un workflow ou un journal.

### Système et droits

- Écriture hors du dossier de données, ou élévation de droits, absentes de la fiche projet.
- Service Windows laissé en place ; installation ou raccourci créés sans accord.
- Lancer un exe élevé depuis un dossier qu'un programme sans droits peut modifier.
- Suppression d'un fichier de l'utilisateur sans copie préalable.

### Dépôt

- Une promesse dans le README que le code ne tient pas.
