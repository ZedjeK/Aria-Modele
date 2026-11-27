<!-- MODELE:debut -->
> **Ce dépôt est le modèle des applis Aria.** Il ne se lance pas tel quel.
> Crée ton appli avec « Use this template », clone-la, puis lance `.\init.ps1`
> dans le dossier du clone : le script demande le nom, la stack, la promesse,
> l'élément orange du logo, si l'appli sert en arrière-plan et son port local
> s'il y en a un, remplit tout et se supprime.
>
> Une stack : `rust-tauri` (Rust, Tauri 2, Svelte 5 en TypeScript). Elle tient
> tout le contrat des applis Aria (dossier de données, journal, secrets chiffrés,
> icône de notification, démarrage avec Windows, installation, mises à
> jour vérifiées), et son interface suit celle d'Aria Studio et d'Aria Cockpit.
> Le détail est dans [stacks/LISEZ-MOI.md](stacks/LISEZ-MOI.md). Pour construire :
> Rust (rustup) et Node.js 24 (version exacte dans `stacks/rust-tauri/.node-version`).
>
> Le socle partagé, dans `socle/`, n'est pas copié dans l'appli : elle le reçoit
> par une dépendance épinglée sur le tag de la version du modèle (décision 0007),
> et une PR de montée par semaine le met à jour.
>
> Le contrat des applis Aria, [docs/CONTRAT.md](docs/CONTRAT.md), fait foi : en
> cas de doute, c'est lui qui tranche, pour le modèle comme pour chaque appli.
>
> Ce qui suit est le squelette du README de la future appli.

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

Windows 11, ou Windows 10 22H2, en 64 bits. L'appli ne demande rien d'autre à
installer : WebView2, le moteur d'affichage de Windows, y est déjà présent (s'il
manque, voir « Si ça ne marche pas »). Droits, ports et matériel nécessaires.

## Installation

1. Télécharge `Aria-{{NOM}}-<version>.zip` depuis la page des versions.
2. Décompresse d'abord le zip en entier : clic droit, « Extraire tout ».
   Lancé depuis l'aperçu du zip dans l'Explorateur, l'exe ne s'installe pas : il
   te demande de décompresser d'abord.
3. Lance `Aria-{{NOM}}.exe` depuis le dossier obtenu. L'appli s'installe avant
   de s'ouvrir :
   - **Installer** range l'exe et ses fichiers dans
     `%USERPROFILE%\.Aria-{{NOM}}\programme`, garde la version précédente, puis
     relance l'appli depuis là. La case « Créer un raccourci sur le Bureau »,
     décochée d'office, ajoute un raccourci sur ton Bureau seulement ; un
     raccourci du même nom qui pointe vers un autre programme reste tel quel.
   - **Annuler** ferme l'appli sans rien installer.

   Le dossier décompressé ne sert plus ensuite : tu peux le supprimer.

Si Windows ou ton antivirus bloque l'exe, voir « Si ça ne marche pas ».

## Si ça ne marche pas

- **« WebView2 n'est pas installé »** : WebView2 est le moteur d'affichage de
  Windows qui dessine la fenêtre de l'appli. Il ne manque que rarement, sur
  certains Windows 10. L'appli propose d'ouvrir la page de Microsoft : installe
  « Microsoft Edge WebView2 Runtime », puis relance l'appli. La page :
  https://developer.microsoft.com/microsoft-edge/webview2/consumer/
- **« Windows a protégé votre ordinateur »** (SmartScreen) : l'exe n'est pas
  signé, et Windows se méfie d'un exe qu'il voit peu. Clique sur « Informations
  complémentaires », puis sur « Exécuter quand même ».
- **L'antivirus met l'exe en quarantaine** : un exe récent et non signé est
  parfois pris pour suspect. Vérifie d'abord qu'il vient de la page des versions
  de ce dépôt : l'empreinte du zip (`Get-FileHash`) doit être celle du fichier
  `.sha256` publié avec la version, et `gh attestation verify` confirme sa
  provenance (voir les notes de version). Restaure-le ensuite depuis la
  quarantaine, et signale le faux positif à l'éditeur de ton antivirus.
- **« Aria {{NOM}} doit d'abord être décompressé »** : l'exe a été lancé depuis
  l'aperçu du zip, et l'Explorateur n'en a extrait que l'exe, sans les fichiers
  livrés à côté de lui. Ferme le message, décompresse le zip en entier (clic
  droit, « Extraire tout »), puis lance l'exe du dossier obtenu.

Pour signaler un problème : dans l'appli, page « Journal », bouton « Exporter le
diagnostic ». Le texte copié (version, Windows, réglages, dernières lignes du
journal) ne contient aucun secret, et le chemin de ton profil Windows y devient
`%USERPROFILE%`. Relis-le avant de le coller dans le formulaire de bug du dépôt :
des noms de fichiers que tu as choisis peuvent y figurer. L'appli n'envoie rien
elle-même.

## Premiers pas

Au premier lancement, un assistant présente l'appli et règle l'essentiel ; il se
rouvre depuis les Paramètres. Ce que l'appli fait ensuite, étape par étape.

## Au quotidien

Section optionnelle : supprime-la si elle reste vide.

## Où sont tes données

Tout est dans `%USERPROFILE%\.Aria-{{NOM}}` :

- `reglages.json` : tes réglages ;
- `journal.json` : les 500 dernières opérations ;
- `sauvegardes\` : une copie datée avant chaque modification, et la version précédente de l'appli ; une copie
  se restaure depuis la page « Journal », après une copie de l'état actuel ;
- `secrets\` : ce que l'appli garde de secret, chiffré pour ton compte Windows (supprime la ligne si elle n'en garde pas) ;
- `cache\` : données de l'interface et fichiers de travail, recréés au besoin ;
- `programme\` : l'appli installée.

Supprimer ce dossier efface tes réglages et tes données : l'appli repart à neuf.
Pour retirer l'appli elle-même, passe d'abord par « Désinstaller » (voir plus bas).

## Désinstaller

Dans les paramètres, le panneau « Installation » a un bouton « Désinstaller ». Il
retire tout ce que l'appli a écrit hors de son dossier de données :

- le démarrage avec Windows, si l'appli en a un (ses entrées dans le registre de
  ton compte) ;
- le raccourci du Bureau, seulement s'il pointe vers l'appli ;
- les fichiers de l'appli dans `programme\`, une fois l'appli fermée ; ce que tu
  y as ajouté toi-même reste. Pour cela, l'appli laisse une copie de son exe
  dans `cache\` : elle part avec le dossier de données.

Tes données restent dans `%USERPROFILE%\.Aria-{{NOM}}`, et l'appli te dit où
elles sont : pour tout effacer, supprime ce dossier une fois l'appli fermée.

Avec une version qui n'a pas encore ce bouton : décoche d'abord « Démarrer avec
Windows » dans les paramètres (sinon son entrée reste dans le registre), quitte
l'appli, puis supprime le raccourci du Bureau s'il existe et le dossier
`programme\` du dossier de données.
Tes données restent tant que tu ne supprimes pas tout le dossier de données.

## Mises à jour

Au lancement, l'appli regarde s'il existe une nouvelle version : une seule
requête, vers la page des versions de ce dépôt sur GitHub, sans rien qui
t'identifie. Une case des paramètres coupe cette recherche.

Quand une version plus récente existe, un bandeau « Mise à jour disponible »
s'affiche en haut de la fenêtre ; « Voir » ouvre le détail et les notes de
version. Rien ne se télécharge ni ne s'installe sans ton clic sur « Installer la
mise à jour ». L'appli télécharge alors le zip de la version, vérifie sa taille
et son empreinte, garde ta version actuelle dans `sauvegardes\versions\`, met
la nouvelle à sa place et se relance.

Pour revenir à la version précédente : quitte l'appli, puis copie les fichiers
de `sauvegardes\versions\<version>\` à la place de ceux de l'appli.

## Sécurité

- **Droits** : l'appli se lance sans droits administrateur.
- **Connexions sortantes** : `api.github.com` pour chercher une mise à jour (si
  la case est cochée), puis `github.com` et son serveur de fichiers pour la
  télécharger, sur ton clic. Ajoute ici celles propres à l'appli, destination
  par destination.
- **Secrets** : chiffrés avec la protection des données de Windows (DPAPI) pour
  ton compte, un fichier par secret dans `secrets\`. Une autre appli lancée sous
  ton compte Windows peut les déchiffrer : cette protection vise la copie du
  dossier sur un autre compte ou un autre PC. Dis ici ce que l'appli garde.

Signature du code (code signing policy) : voir
[docs/POLITIQUE-SIGNATURE.md](docs/POLITIQUE-SIGNATURE.md).

Pour signaler une faille : voir [SECURITY.md](SECURITY.md).

## Limites à connaître

Ce que l'appli ne fait pas.

Mises à jour : l'empreinte SHA-256 vérifie que le téléchargement est intact, pas
d'où il vient. La provenance repose sur le compte GitHub de l'auteur, sur les
releases immuables (une version publiée ne change plus) et sur l'attestation de
provenance que GitHub enregistre pour chaque zip (`gh attestation verify`, voir
les notes de version). Si le compte de l'auteur était compromis, une version
piégée pourrait quand même être proposée.

Projet personnel, sans audit de sécurité indépendant.

## Provenance des composants

Section à garder seulement si l'appli embarque un composant tiers. Le détail
est dans [LICENCES-TIERS.txt](LICENCES-TIERS.txt).

## Compiler soi-même

Il faut Rust, installé par [rustup](https://rustup.rs), et Node.js 24 (la version
exacte, celle de la CI, est dans `.node-version`). Puis, dans
le dossier du dépôt :

```powershell
rustup toolchain install
.\tools\tester.ps1
.\build.ps1
```

`rustup toolchain install` prend la version de Rust écrite dans
`rust-toolchain.toml`.

Pour le contrôle des dépendances Rust que fait aussi la CI (licences, failles
connues), installe cargo-deny : `cargo install cargo-deny --version 0.20.2 --locked`.
Sans lui, `tester.ps1` saute ce contrôle et le dit.

`tester.ps1` lance les contrôles et les tests de l'interface et du Rust ;
`build.ps1` construit l'exe dans `.work\sortie\`. Comment vérifier que l'exe
obtenu est le bon.

## Structure

```text
appli/        l'exe Tauri : identité de l'appli et ses propres commandes, posées sur le socle partagé
  capabilities/   ce que la fenêtre principale a le droit d'appeler, en plus de ce que le socle autorise
interface/    l'interface en Svelte et TypeScript (pages, dialogues, composants)
tools/        contrôles, version, zip de release
docs/         logo, fiche du projet, contrat, notes de version
build.ps1     construction de l'exe
```

Les règles communes à toutes les applis Aria (données, réseau, installation, mises
à jour, interface) sont dans [docs/CONTRAT.md](docs/CONTRAT.md). Le socle partagé
qui les tient (crate `aria-coeur` : dossier de données, réglages, journal, secrets,
installation, mises à jour, icône, désinstallation) vient du dépôt modèle, à la
version épinglée dans `appli/Cargo.toml`. Chaque semaine, le workflow « Monter le
socle » ouvre une PR quand une version plus récente est publiée.

## Licence

MIT : voir [LICENSE](LICENSE). Les composants embarqués gardent leur propre
licence : voir [LICENCES-TIERS.txt](LICENCES-TIERS.txt).
