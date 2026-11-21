# Squelettes de stack

Un dossier par stack. `init.ps1` copie celui que tu choisis à la racine du dépôt, puis supprime `stacks/`.

Les squelettes arrivent un par un, chacun éprouvé sur une appli jetable : `.\tools\tester-squelettes.ps1` (et le
workflow `squelettes.yml`) passe chacun par `init.ps1`, le compile et en fait le zip. Ces deux fichiers ne servent
qu'au dépôt modèle : `init.ps1` les supprime dans l'appli créée.

| Dossier | Stack | État |
| --- | --- | --- |
| `rust-tauri/` | Rust, Tauri 2, Svelte 5 en TypeScript | socle posé : dossier de données, réglages, journal, sauvegardes, fond animé, instance unique, icône de notification et son menu, démarrage automatique, secrets (DPAPI), installation par l'exe, mises à jour intégrées (release GitHub, taille et SHA-256 vérifiées), interface sur le modèle d'Aria Studio et d'Aria Cockpit (logo et icône Windows de la famille tirés de `docs/logo.svg` par `tools/icone.ps1`, fenêtre sans cadre, menu latéral par familles, état en direct, bandeau de mise à jour, dialogues posés sur la page), exe en un seul fichier (WebView2 de Windows, données de WebView2 dans le dossier de l'appli) ; contrat complet |

## Ce qu'un squelette doit fournir

Trois scripts, toujours au même endroit, pour que la CI soit la même quelle que soit la stack :

| Fichier | Rôle |
| --- | --- |
| `build.ps1` | compile l'exe portable et pose tout ce qui doit être livré dans `.work\sortie\` |
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
| `stack.json` | réglages lus par `init.ps1` : `{ "nom": "Rust, Tauri 2, Svelte 5 (TypeScript)", "runsOn": "windows-latest" }`, et `nonFournis` si le squelette ne tient pas encore tout le contrat |

`nonFournis` liste ce que le squelette ne tient pas encore : `installation`, `demarrageAuto` (démarrage automatique et
icône de notification), `misesAJour`. `init.ps1` refuse de créer une appli qui demande les deux premiers, et prévient pour
le troisième : les mises à jour intégrées sont obligatoires pour toute appli Aria, donc pas de release publique tant
qu'elles manquent. La clé disparaît quand le squelette tient tout.

Les fichiers du squelette utilisent les mêmes marqueurs que le socle : `{{NOM}}`, `{{NOM_MIN}}`, `{{PROMESSE}}`,
`{{ELEMENT_ORANGE}}`, `{{PORT}}`, `{{ANNEE}}`, `{{STACK}}`, `{{DEMARRAGE_AUTO}}`, `{{INSTALLATION}}`, et `{{EN_FOND}}`
(`true` ou `false`, pour le code : l'appli sert en arrière-plan, avec icône de notification et démarrage automatique
proposé) et `{{INSTALLABLE}}` (`true` ou `false` : l'exe propose de s'installer). Un fichier ou un dossier dont le
nom contient `Aria-Modele` est renommé en `Aria-<Nom>`.

`tools\empaqueter.ps1` écrit dans le zip un `fichiers.txt` : la liste de ce que livre la version. L'installation par
l'exe et les mises à jour d'un squelette copient exactement cette liste. Tout ce que `build.ps1` pose dans
`.work\sortie\` doit donc être à la racine de ce dossier, sans sous-dossier.

Les blocs entre `<!-- INSTALLATION:debut -->` et `<!-- INSTALLATION:fin -->` ne restent que si l'exe s'installe ; ceux
entre `<!-- PORTABLE:debut -->` et `<!-- PORTABLE:fin -->`, que s'il ne s'installe pas. Les balises sont retirées dans
les deux cas.
