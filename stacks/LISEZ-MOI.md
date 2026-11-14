# Squelettes de stack

Un dossier par stack. `init.ps1` copie celui que tu choisis à la racine du dépôt, puis supprime `stacks/`.

Les squelettes arrivent un par un, chacun éprouvé sur une appli jetable : `.\tools\tester-squelettes.ps1` (et le
workflow `squelettes.yml`) passe chacun par `init.ps1`, le compile et en fait le zip. Ces deux fichiers ne servent
qu'au dépôt modèle : `init.ps1` les supprime dans l'appli créée.

| Dossier | Stack | État |
| --- | --- | --- |
| `go-wails/` | Go, Wails v3, Svelte 5 | à écrire |
| `dotnet-wpf/` | C# .NET 10, WPF, WPF-UI | socle posé : compile, tests, exe autonome ; le contrat vient ensuite |
| `python-fastapi/` | Python, FastAPI, exe empaqueté | à écrire |
| `extension-mv3/` | exe Electron et son extension Manifest V3 | à écrire |

## Ce qu'un squelette doit fournir

Trois scripts, toujours au même endroit, pour que la CI soit la même quelle que soit la stack :

| Fichier | Rôle |
| --- | --- |
| `build.ps1` | compile l'exe portable et pose tout ce qui doit être livré dans `.work\sortie\` |
| `tools\tester.ps1` | lance les tests de la stack ; code de retour non nul en cas d'échec |
| `tools\version.ps1` | affiche la version sur sa dernière ligne, sans le « v » (`1.2.3`) |

Et un dossier `_socle\`, que `init.ps1` lit puis supprime. Chaque fichier remplace la ligne `<<STACK:nom>>` du même
nom dans le socle ; un fichier absent supprime simplement la ligne.

| Fichier de `_socle\` | Remplace le marqueur dans |
| --- | --- |
| `gitignore.txt` | `.gitignore` |
| `gitattributes.txt` | `.gitattributes` |
| `dependabot.txt` | `.github\dependabot.yml` (blocs indentés de deux espaces) |
| `outils.txt` | `tests.yml` et `release.yml` (étapes d'installation des outils, indentées de six espaces) |
| `codeql.txt` | `codeql.yml` (une entrée de matrice, indentée de dix espaces) |
| `stack.json` | réglages lus par `init.ps1` : `{ "nom": "Go + Wails", "runsOn": "windows-latest" }` |

Les fichiers du squelette utilisent les mêmes marqueurs que le socle : `{{NOM}}`, `{{NOM_MIN}}`, `{{PROMESSE}}`,
`{{ELEMENT_ORANGE}}`, `{{PORT}}`, `{{ANNEE}}`, `{{STACK}}`, `{{DEMARRAGE_AUTO}}`, `{{INSTALLATION}}`. Un fichier ou un
dossier dont le nom contient `Aria-Modele` est renommé en `Aria-<Nom>`.

Les blocs entre `<!-- INSTALLATION:debut -->` et `<!-- INSTALLATION:fin -->` ne restent que si l'exe s'installe ; ceux
entre `<!-- PORTABLE:debut -->` et `<!-- PORTABLE:fin -->`, que s'il ne s'installe pas. Les balises sont retirées dans
les deux cas.
