# Contrat des applis Aria

Les règles que toute appli de la famille respecte. Ce fichier fait foi : en cas de doute, il passe avant une
préférence de style, et le code, le README et la fiche projet s'y conforment.

Il est le même dans toutes les applis : il fait partie du socle commun (`tools/socle.txt`). Pour le changer, passe
par le dépôt modèle. Les décisions d'où viennent ses règles sont dans `docs/decisions/`.

## Principes

1. **Tout en local.** Aucune télémétrie, rien n'est envoyé à l'auteur.
2. **Rien d'autre que l'appli** : chaque appli s'installe d'elle-même dans son dossier, sans installeur, sans
   service et sans droits administrateur, et se désinstalle par son propre bouton (décision 0008).
3. **Rien sans accord, et tout se restaure** : récapitulatif, sauvegarde avant modification, annulation, et un
   bouton « Restaurer » pour chaque sauvegarde. Une restauration ne supprime rien.
4. **Le README ne promet que ce que le code tient** (section « Limites à connaître »).
5. **Moindre privilège** : pas de droits administrateur si on peut s'en passer.
6. **Français et tutoiement** : interface, README, erreurs, commentaires, commits. Le contenu que l'appli traite peut
   être dans n'importe quelle langue ; son interface reste en français.
7. **Accessible** : tout se fait au clavier, le focus se voit, et le texte garde un contraste d'au moins 4,5 pour 1
   avec son fond.

**Sans dépendance** : l'appli ne demande rien à installer. Seule exception : WebView2 Evergreen, le moteur
d'affichage de Windows, présent d'office sur Windows 11 et sur Windows 10 22H2 à jour. S'il manque, l'appli le dit et
donne le lien de Microsoft. Le runtime Visual C++ est lié dans l'exe, et les tests le vérifient. Rien n'est extrait
dans `%TEMP%` au lancement.

**Plateformes** : Windows 11 ; Windows 10 22H2 tant que WebView2 y reçoit des mises à jour (au moins jusqu'en octobre
2028). En x64 seulement (décision 0010) : sur un PC ARM, l'exe peut tourner par l'émulation de Windows, sans
garantie, et le README le dit.

Toute question de paramétrage se tranche en restant fidèle au contrat ci-dessous.

## Socle partagé

Chaque appli est faite de son métier, posé sur le socle partagé du dépôt modèle (décision 0007) :

| Partie du socle | Ce qu'elle apporte | Comment l'appli la reçoit |
| --- | --- | --- |
| crate `aria-coeur` | ce que toute appli a toujours (ce contrat, sections suivantes), et les modules en option, à activer un par un | dépendance git du dépôt modèle, épinglée par tag, dans `appli/Cargo.toml` |
| paquet `aria-interface` | thème, composants, dialogues, fond animé, assistant de premier lancement | archive `.tgz` jointe à la release du dépôt modèle, à la même version |
| workflows | tests, release, CodeQL, montée du socle | posés par `init.ps1` dans `.github/workflows/` ; Dependabot tient leurs actions à jour |

- L'appli ne recopie ni ne modifie le code du socle. Un correctif du socle passe par le dépôt modèle, puis arrive
  dans chaque appli par la PR de montée.
- La crate et le paquet ont la même version que le dépôt modèle, `X.Y.Z`. Une fonction retirée est d'abord marquée
  dépréciée dans une version mineure, puis retirée dans la version majeure suivante.
- Le workflow « Monter le socle » ouvre chaque semaine une seule PR qui monte la crate et le paquet ensemble, à la
  dernière version publiée du dépôt modèle. La CI de l'appli la teste comme toute PR.

## Fiche projet

Chaque appli a sa fiche, `docs/FICHE.md` : ce qu'elle a de propre. Tu la lis avant toute modification et tu la
tiens à jour quand l'appli change.

- **Identité** : nom, promesse, élément orange du logo, fond animé (motif, ce qui est aléatoire, nombre par défaut).
- **Technique** : version du socle, modules en option activés, droits administrateur, moteurs et modèles utilisés,
  port local d'un service (s'il y en a un).
- **Données** : données de l'utilisateur et données secrètes, écritures hors du dossier de données (dont le raccourci
  du Bureau), données reprises d'une ancienne version.
- **Ouverture** : connexions sortantes (liste complète, destination par destination), composants tiers embarqués.
- **Réussite** : critère mesurable de réussite de la v0.1.
- **Cap** : objectif de la prochaine version, non-objectifs, plateformes, budget (taille de l'exe, mémoire au repos,
  délai d'ouverture). La taille de l'exe est relevée dans `BUILDINFO.txt` à chaque zip ; un budget dépassé se dit
  dans les notes de la version.

Les lignes « à renseigner » se complètent **avant la première ligne de code**.

## Contrat de comportement

Tout ce qui suit est fourni par le socle : l'appli l'active ou le règle, elle ne le réécrit pas.

### Données et secrets

- **Dossier de données** : données, réglages, journal, sauvegardes, cache de l'interface, tout dans
  `%USERPROFILE%\.Aria-<Nom>`. Rien dans `%APPDATA%`, `%LOCALAPPDATA%` ni `%TEMP%`.
- **Dossier commun** : les modèles et les moteurs locaux partagés vivent dans `%USERPROFILE%\.Aria` (décision 0009),
  et rien d'autre : ni réglage, ni secret, ni donnée personnelle. L'utilisateur peut le placer sur un autre disque,
  une fois, pour toute la famille.
- **Hors de ces dossiers**, l'appli n'écrit que ce que sa fiche projet annonce : l'entrée de démarrage automatique,
  le raccourci du Bureau, et ce qui fait son travail, toujours avec accord et sauvegarde préalable.
- **Fichiers du dossier** :

  | Fichier | Contenu |
  | --- | --- |
  | `reglages.json` | les réglages, avec le numéro de leur format |
  | `journal.json` | les 500 dernières opérations |
  | `sauvegardes\` | les sauvegardes ; `sauvegardes\versions\` garde la version précédente du programme |
  | `secrets\` | les secrets, chiffrés par DPAPI, un fichier par secret (si l'appli en garde) |
  | `cache\` | le cache de l'interface |
  | `programme\` | l'appli installée |

- **Réglages** : `reglages.json` porte le numéro de son format, et chaque changement de format a sa migration. La
  lecture est tolérante, champ par champ : un champ illisible reprend sa valeur par défaut, sans effacer les autres.
  Avant de réécrire un fichier qui ne se relisait pas tel quel, l'appli le sauvegarde et le note au journal.
- **Restauration** : chaque sauvegarde se restaure depuis l'interface. Une restauration commence par sauvegarder
  l'état actuel, puis remet l'ancien ; elle ne supprime rien.
- **Secrets** (jetons, mots de passe, clés) : jamais en clair dans `reglages.json`, le journal ni les sauvegardes. Au
  minimum, ils sont chiffrés par la protection des données de Windows (DPAPI), liée au compte de l'utilisateur, avec
  une entropie propre à l'appli, et restent dans le dossier de données. Le README dit contre quoi cette protection ne
  joue pas : un programme lancé sous le même compte. Un secret chiffré ne se relit pas sur un autre PC ni après une
  réinstallation de Windows : l'appli le redemande alors, sans planter.
- **Reprise des données** : la première version d'une appli sur le socle reprend les données de son ancienne version
  (autre dossier, autre format). Elle copie avant de reprendre, ne modifie jamais l'ancien dossier, et dit ce qu'elle
  n'a pas pu reprendre.
- **Diagnostic** : la page « Journal » a un bouton « Exporter le diagnostic » qui copie dans le presse-papiers la
  version, le build, les versions de Windows et de WebView2, les réglages et les 50 dernières lignes du journal. Aucun
  secret, aucun envoi réseau ; le chemin du profil y devient `%USERPROFILE%`. Le formulaire de bug du dépôt demande ce
  texte.

### Lancement et présence

- **Instance unique** : relancer l'exe ramène la fenêtre existante.
- **Assistant de premier lancement** : au premier lancement après l'installation, un assistant à étapes présente
  l'appli et règle l'essentiel. Il se rouvre depuis les Paramètres.
- **Tâches longues** : une opération qui peut durer montre sa progression, s'annule, et ne fige jamais la fenêtre.
  Son état apparaît dans les pastilles du haut de la page, et dans l'icône de notification si l'appli en a une.

### Installation et mises à jour

- **Installation** (décision 0008) :
  - **Proposition** : lancé hors de son dossier (zip décompressé), l'exe propose de s'installer. S'il est refusé,
    l'appli se ferme. Il n'y a ni lancement sans installation, ni option `--portable`.
  - **Installer** range l'exe, ce qu'il livre à côté de lui (`fichiers.txt`) et les licences dans
    `%USERPROFILE%\.Aria-<Nom>\programme`, garde la version précédente, puis relance depuis là.
  - **Raccourci** : une case « Créer un raccourci sur le Bureau », décochée d'office : Bureau de l'utilisateur
    seulement, jamais le Bureau public, cible stable d'une version à l'autre. Un raccourci déjà présent est proposé en
    mise à jour s'il pointe vers l'appli, et laissé tel quel sinon : il n'est jamais remplacé en silence.
  - **Droits du dossier** : une appli qui tourne élevée vérifie, avant de copier, que le dossier des données (et
    `programme\`) n'est modifiable que par le système et les administrateurs, propriétaire compris, et refuse sinon :
    on ne lance pas un exe élevé depuis un dossier qu'un programme sans droits peut modifier. Une appli lancée avec
    les droits de l'utilisateur ne fait pas ce contrôle : elle ne pourrait pas créer un tel dossier, et elle n'a pas
    de droits élevés à protéger.
  - **Marque Internet** : les fichiers installés n'héritent pas de la marque « téléchargé depuis Internet » :
    l'utilisateur a lancé l'exe et accepté l'installation.
  - **Fin de l'installation** : la fenêtre se ferme d'elle-même une fois la copie finie, et l'appli se relance depuis
    son dossier. L'installation est testée automatiquement par la CI, et par un vrai clic avant la release.
  - **Décompresser d'abord** : le zip se décompresse en entier (clic droit, « Extraire tout ») avant de lancer l'exe.
    Lancé depuis l'aperçu du zip de l'Explorateur, l'exe ne s'installe pas : il demande de décompresser d'abord. Le
    README et les notes de chaque version le disent en toutes lettres (« décompresse d'abord le zip »).
- **Mises à jour** : lire `releases/latest` du dépôt de l'appli, proposer, ne télécharger que depuis cette release
  sur `github.com` en HTTPS, vérifier la taille (plafonnée) et le SHA-256, plafonner aussi la taille décompressée,
  garder l'ancienne version, ne rien installer sans accord. Elles remplacent l'exe dans `programme\`. L'empreinte
  prouve que le téléchargement est intact, pas d'où il vient : la provenance repose sur le compte de l'auteur, les
  releases immuables et l'attestation jointe, et le README le dit dans « Limites à connaître ».
- **Désinstallation** : un bouton « Désinstaller » dans les Paramètres retire tout ce que l'appli a écrit hors de
  son dossier de données (démarrage automatique, raccourci du Bureau s'il pointe vers elle, association de fichiers
  qu'elle a posée), puis supprime `programme\` une fois l'appli fermée. Les données restent, et l'appli dit où elles
  sont. Une case « Supprimer aussi mes données », décochée d'office, dit ce qui part (réglages, journal, sauvegardes,
  secrets) et que cela ne se récupère pas ; cochée, la copie de fin efface tout `%USERPROFILE%\.Aria-<Nom>` une fois
  l'appli fermée, elle comprise. Le dossier commun `%USERPROFILE%\.Aria` n'est jamais touché.

### Modules en option

Chaque module s'active dans `appli/Cargo.toml` (fonction de la crate `aria-coeur`), sauf le démarrage avec Windows
et l'icône, que donne `en_fond` dans l'identité de l'appli, et se déclare dans la fiche. Les modules disponibles sont
dans `socle/LISEZ-MOI.md` du dépôt modèle.

| Module | Règles |
| --- | --- |
| Démarrage avec Windows | désactivé par défaut ; case dans les réglages et dans le menu de l'icône ; lancement avec `--silencieux` ; état relu du système |
| Icône de notification | avec le démarrage avec Windows : Ouvrir, Démarrer avec Windows, Quitter (voir « Icône et menu ») |
| Moteur local (décision 0009) | moteurs dans le dossier commun, une version par dossier ; l'appli déclare la plage de versions qu'elle accepte ; une mise à jour n'est proposée que dans cette plage ; essai après mise à jour, retour à la version précédente s'il échoue ; un moteur ou un modèle ne s'efface que sur demande, après avoir dit quelles applis s'en servent |
| Téléchargement vérifié | source épinglée et annoncée dans la fiche, HTTPS, taille et SHA-256 contrôlés avant usage, reprise d'un téléchargement coupé |
| Partage de la carte graphique | l'appli qui charge un modèle déclare ce qu'elle occupe dans le dossier commun, et avertit quand une autre appli occupe déjà la mémoire vidéo |
| Notifications | désactivables dans les réglages ; une notification par événement que l'utilisateur doit voir, jamais pour attirer l'attention |
| Raccourcis clavier globaux | modifiables et désactivables ; un conflit avec un autre programme est signalé, jamais forcé |
| Fichiers et dossiers | fenêtre « Ouvrir » et « Enregistrer » de Windows, glisser-déposer ; un fichier de l'utilisateur réécrit est copié avant ; une association de type de fichier seulement avec accord, retirée à la désinstallation |
| Échanges entre applis | sur la boucle locale seulement, avec un jeton (voir « Service local ») ; en lecture par défaut ; une action qui modifie passe par l'accord de l'utilisateur dans l'appli |
| Droits administrateur | seulement si la fiche le dit, et pour la seule opération qui l'exige (voir « Réseau et droits ») |

### Réseau et droits

- **Réseau local** : la boucle locale seulement, `127.0.0.1`, et `::1` si l'appli en a besoin ; jamais `0.0.0.0` ni
  `::`. L'interface de l'appli est sa fenêtre : elle ne s'ouvre pas dans le navigateur.
- **Service local** (échanges entre applis, complément d'un autre logiciel) :
  - refuse toute requête dont l'en-tête `Host` n'est pas `127.0.0.1:<port>`, `localhost:<port>` (ou `[::1]:<port>`
    s'il écoute sur `::1`) ;
  - refuse toute requête dont l'`Origin`, quand il est présent, n'est pas le sien ;
  - ne fait passer aucune action par un GET, n'ouvre aucun CORS ;
  - exige à chaque requête un jeton tiré au hasard à chaque lancement.

  Ports déjà pris : `docs/PORTS.md` du dépôt modèle.
- **Droits administrateur** : seulement si la fiche projet le dit, et autant que possible pour la seule opération qui
  l'exige. Une appli élevée traite son dossier de données comme une entrée non fiable : elle n'y charge ni code ni
  commande, et valide ce qu'elle y lit. Elle ne lance aucun moteur depuis le dossier commun : elle garde ses moteurs
  dans son propre dossier protégé.

### Cas particuliers

Ce que le socle ne fournit pas, et que l'appli fait elle-même :

- **Appli à extension de navigateur** : l'exe est l'appli. Il range l'extension dans
  `%USERPROFILE%\.Aria-<Nom>\extension` et l'y met à jour.
- **Appli serveur du réseau local** (exception, par exemple un coffre joint par les téléphones) : seulement si la
  fiche projet le dit, en TLS, jamais exposée sur Internet, et son port est noté dans `docs/PORTS.md` du dépôt
  modèle.
- **Signature de code** : Authenticode par SignPath Foundation, appli par appli, une fois l'appli acceptée par la
  fondation ; Windows affiche alors « SignPath Foundation » comme éditeur. Tant qu'une appli n'est pas signée, son
  README et les notes de ses versions expliquent l'avertissement SmartScreen. La politique de signature de chaque
  appli est dans `docs/POLITIQUE-SIGNATURE.md` ; l'étape de signature reste coupée tant que la variable de dépôt
  `SIGNPATH_ACTIF` ne vaut pas `true`.

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

- **Accessibilité** : le texte et les éléments qui agissent gardent un contraste d'au moins 4,5 pour 1 avec leur
  fond, mesuré par les tests du paquet d'interface ; le focus clavier se voit sur tout ce qui agit ; une couleur
  n'est jamais le seul porteur d'un sens (un mot ou une forme l'accompagne).

### Fond animé

- **Il dit ce que fait l'appli.** Zone principale seulement, jamais sous le menu latéral ; panneaux opaques ; couleurs
  de la palette ; éléments tirés au hasard à chaque passage. Le fond de chaque appli est décrit dans sa fiche projet.
- **Réglages obligatoires**, enregistrés dans `reglages.json` : `fond.actif`, `fond.vitesse` (lente, normale,
  rapide), `fond.minimum` et `fond.maximum`.
- **En pause** quand la fenêtre est cachée, **image fixe** si Windows demande moins d'animations.

### Mise en page et formes

- **Référence** : le paquet `aria-interface` du socle (`socle/interface/` dans le dépôt modèle). Une forme ou une
  disposition nouvelle part de ce qu'il a déjà posé, et y entre si une deuxième appli en a besoin.
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
- **Pages de l'appli elle-même** : « Journal » et « Paramètres » (démarrage, installation, mises à jour, affichage,
  assistant) dans leur propre famille, à part du travail de l'appli.
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
  sur la page a un cadre à coins coupés au liseré fin.
- **Assistant à étapes** : les étapes en encoches, comme les sous-onglets mais non cliquables, avec une coche sur
  celles qui sont faites. L'encadré « À savoir » donne le commentaire de l'étape. Chaque étape a un panneau accentué.
  Le bouton orange va à l'action à faire maintenant (« Installer » tant qu'un élément obligatoire manque), puis
  revient sur « Suivant » ; jamais deux à la fois.
- **Tâche en cours** : une barre de progression dans le panneau de la tâche, avec le bouton « Annuler » en contour
  discret ; la pastille de la tâche reste cyan tant qu'elle tourne.
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
- **Dépendances Rust** : `cargo deny` (`deny.toml`) passe à chaque test : licences d'une liste explicite, aucune
  faille connue, crates.io et le dépôt modèle pour seules origines. La liste des crates de `LICENCES-TIERS.txt` est
  générée par `.\tools\licences-rust.ps1` (cargo-about) ; `verifier-release.ps1` refuse une liste qui ne correspond
  plus à `Cargo.lock`.
- **Licences des composants embarqués** : tout ce que l'exe embarque (crates, paquets npm, polices, données) est sous
  une licence de la liste de `deny.toml`, ou sous la SIL Open Font License pour les polices. Un moteur lancé à part
  (processus séparé, téléchargé, non lié à l'exe) peut avoir une autre licence : la fiche la nomme et
  `LICENCES-TIERS.txt` la reprend.
- **Branches, commits, PR, tags, releases** : voir [CONTRIBUTING.md](../CONTRIBUTING.md), sans exception pour les
  applis.

### Release d'une appli

La chaîne générique (tag, workflow, brouillon, publication) est décrite dans [CONTRIBUTING.md](../CONTRIBUTING.md) ;
voici ce que toute appli y ajoute.

| Point | Règle |
| --- | --- |
| Version | à un seul endroit, lue par `.\tools\version.ps1` ; le tag `vX.Y.Z` lui est égal |
| Notes | `docs/releases/<tag>.md` obligatoire et complet, avec « décompresse d'abord le zip » ; `.\tools\verifier-release.ps1 -Tag vX.Y.Z` vérifie avant le tag les notes, la fiche et les licences |
| Recette | `docs/RECETTE.md`, cochée sur la prerelease avant de la rendre courante |
| `release.yml` | refait le contrôle des notes, refuse un tag dont le commit n'est pas dans la branche par défaut, construit, joint l'empreinte et l'attestation de provenance, s'arrête au brouillon |
| Essai à blanc | `gh workflow run Release --ref main` : build, zip et contrôle des notes, sans publication |
| Zip | contient `fichiers.txt`, la liste exacte que copient l'installation et les mises à jour |
| Tags `v*` | verrouillés par un ruleset ; un tag poussé par erreur suit le retour arrière décrit dans [CONTRIBUTING.md](../CONTRIBUTING.md) |
| Publication | annonce la version à tous les utilisateurs (l'appli lit `releases/latest`) : une prerelease d'abord pour tester sans prévenir |
| Protections du dépôt | posées et relues par `.\tools\proteger-depot.ps1`, qui garde un état d'avant daté |

## Interdits

### Réseau et données

- Télémétrie, ou une connexion sortante absente de la fiche projet.
- Écoute ailleurs que sur la boucle locale (`127.0.0.1`, `::1`), hors appli serveur annoncée par sa fiche.
- Un secret en clair dans le dossier de données, le dépôt, un workflow ou un journal.

### Système et droits

- Écriture hors du dossier de données et du dossier commun, ou élévation de droits, absentes de la fiche projet.
- Service Windows laissé en place ; raccourci ou association de fichiers créés sans accord.
- Remplacer un raccourci ou une association qui ne pointe pas vers l'appli.
- Lancer un exe élevé depuis un dossier qu'un programme sans droits peut modifier, ou un moteur depuis le dossier
  commun dans une appli élevée.
- Suppression d'un fichier de l'utilisateur sans copie préalable.

### Dépôt

- Une promesse dans le README que le code ne tient pas.
- Recopier ou modifier dans une appli le code du socle.
