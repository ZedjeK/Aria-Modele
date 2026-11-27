# Squelettes de stack

Un dossier par stack. `init.ps1` copie celui que tu choisis à la racine du dépôt, puis supprime `stacks/`.

Les squelettes arrivent un par un, chacun éprouvé sur une appli jetable : `.\tools\tester-squelettes.ps1` (et le
workflow `squelettes.yml`) passe chacun par `init.ps1`, le compile et en fait le zip. Ces deux fichiers ne servent
qu'au dépôt modèle : `init.ps1` les supprime dans l'appli créée.

| Dossier | Stack | État |
| --- | --- | --- |
| `rust-tauri/` | Rust, Tauri 2, Svelte 5 en TypeScript | socle posé : dossier de données, réglages, journal, sauvegardes, fond animé, instance unique, icône de notification et son menu, démarrage automatique, secrets (DPAPI), installation par l'exe, mises à jour intégrées (release GitHub, taille et SHA-256 vérifiées), interface sur le modèle d'Aria Studio et d'Aria Cockpit (logo et icône Windows de la famille tirés de `docs/logo.svg` par `tools/icone.ps1`, fenêtre sans cadre, menu latéral par familles, état en direct, bandeau de mise à jour, dialogues posés sur la page), exe en un seul fichier (WebView2 de Windows, données de WebView2 dans le dossier de l'appli) ; contrat complet |

## Socle partagé

Le code que toute appli a toujours vit hors des squelettes, dans `socle/` (décision 0007) : la crate `aria-coeur`
(`socle/coeur`) porte le contrat en Rust et, avec sa fonction `fenetre`, la fenêtre Tauri, ses commandes, le
lancement, l'installation, les mises à jour, l'icône et la désinstallation. Le squelette `rust-tauri` n'apporte que
l'identité de l'appli et ses propres commandes. Le paquet npm `aria-interface` (`socle/interface`) porte le thème, les
composants, la fenêtre principale (`Fenetre`), les dialogues, le Journal, les Paramètres, la proposition
d'installation et le menu de l'icône : l'interface du squelette n'apporte que son logo, ses familles de pages, ses
pastilles d'état et ses pages.

Dans le dépôt modèle, le squelette prend le socle par son chemin (`appli/Cargo.toml`). `init.ps1` le remplace par la
dépendance git du dépôt modèle, épinglée sur le tag de la version du socle (`socle/coeur/Cargo.toml`), ou, avec
`-Socle <clone>`, par le chemin du socle de ce clone (ce que font les tests). De même, `interface/package.json` prend
`aria-interface` par son chemin ; `init.ps1` le remplace par l'archive `aria-interface-X.Y.Z.tgz` jointe à la release
du dépôt modèle, ou par le dossier du socle du clone. La crate et le paquet ont la même version. Le code du socle n'est jamais copié
dans l'appli. Les commandes du socle et ses capacités viennent de lui (`DEP_ARIA_COEUR_COMMANDES`, capacités ajoutées
au lancement) : l'appli n'autorise que les siennes. `rust-toolchain.toml`, `clippy.toml` et `rustfmt.toml` sont les
mêmes dans le socle et dans le squelette, et `tester-squelettes.ps1` le vérifie.

## Ce qu'un squelette doit fournir

Trois scripts, toujours au même endroit, pour que la CI soit la même quelle que soit la stack :

| Fichier | Rôle |
| --- | --- |
| `build.ps1` | compile l'exe et pose tout ce qui doit être livré dans `.work\sortie\` |
| `tools\tester.ps1` | lance les tests de la stack ; code de retour non nul en cas d'échec |
| `tools\version.ps1` | affiche la version sur sa dernière ligne, sans le « v » (`1.2.3`) |
| `tools\icone.ps1` | facultatif : tire l'icône Windows et `docs\logo.png` de `docs\logo.svg` ; `tester-squelettes.ps1` le lance avant `build.ps1` |

Et un dossier `_socle\`, que `init.ps1` lit puis supprime. Chaque fichier remplace la ligne `<<STACK:nom>>` du même
nom dans le socle ; un fichier absent supprime simplement la ligne.

| Fichier de `_socle\` | Remplace le marqueur dans |
| --- | --- |
| `gitignore.txt` | `.gitignore` |
| `gitattributes.txt` | `.gitattributes` |
| `dependabot.txt` | `.github\dependabot.yml` (blocs indentés de deux espaces) |
| `outils.txt` | `tests.yml` et `release.yml` (étapes d'installation des outils, indentées de six espaces) ; chaque action y garde le SHA épinglé dans `squelettes.yml`, que Dependabot tient à jour (`tools\tester.ps1` refuse un écart) |
| `codeql.txt` | `codeql.yml` (une entrée de matrice, indentée de dix espaces) |
| `licences.txt` | `LICENCES-TIERS.txt` (un bloc par composant que l'exe de la stack embarque : polices, runtime, bibliothèques ; ce qui dépend du build reste « à renseigner », et `verifier-release.ps1` le fait compléter avant chaque version) |
| `stack.json` | réglages lus par `init.ps1` : `{ "nom": "Rust, Tauri 2, Svelte 5 (TypeScript)", "runsOn": "windows-latest" }` |

Les fichiers du squelette utilisent les mêmes marqueurs que le socle : `{{NOM}}`, `{{NOM_MIN}}`, `{{PROMESSE}}`,
`{{ELEMENT_ORANGE}}`, `{{PORT}}`, `{{ANNEE}}`, `{{STACK}}`, `{{DEMARRAGE_AUTO}}`, et `{{EN_FOND}}` (`true` ou
`false`, pour le code : l'appli sert en arrière-plan, avec icône de notification et démarrage automatique proposé).
Un fichier ou un dossier dont le nom contient `Aria-Modele` est renommé en `Aria-<Nom>`.

`tools\empaqueter.ps1` écrit dans le zip un `fichiers.txt` : la liste de ce que livre la version. L'installation par
l'exe et les mises à jour d'un squelette copient exactement cette liste. Tout ce que `build.ps1` pose dans
`.work\sortie\` doit donc être à la racine de ce dossier, sans sous-dossier.

