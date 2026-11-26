# 0001 : une seule stack, Rust et Tauri, pour Windows seulement

Date : 2026-11-26.

## Contexte

Les applis de la famille étaient écrites dans plusieurs stacks (Python avec PyInstaller, .NET avec WPF, et d'autres).
Chacune avait ses règles propres : extraction de bibliothèques au lancement, casse des boutons, build, licences. Le
dépôt modèle portait autant de squelettes et d'exceptions au contrat, et chaque correctif se faisait plusieurs fois.

## Décision

Une seule stack, `rust-tauri` : Rust (édition 2024, version épinglée par `rust-toolchain.toml`), Tauri 2, interface
en Svelte 5 et TypeScript. Une seule plateforme : Windows 11 et Windows 10 22H2, en 64 bits. Toutes les applis y
migrent ; les règles propres aux autres stacks sortent du contrat.

## Conséquences

- Un seul squelette, un seul contrat sans exception de stack, une seule chaîne de tests et de release.
- L'exe dépend de WebView2 Evergreen, le moteur d'affichage de Windows (présent d'office sur Windows 11 et sur
  Windows 10 22H2 à jour) ; le runtime Visual C++ est lié dans l'exe (voir le contrat, « Sans dépendance »).
- Les applis existantes doivent migrer, une à la fois.
- Aucune version pour macOS ni Linux : les crates propres à ces systèmes ne comptent pas (`deny.toml`, cible
  Windows seulement).

## À revoir si

- WebView2 n'est plus mis à jour sur une plateforme visée (au moins jusqu'en octobre 2028 pour Windows 10) ;
- Tauri 3 sort ;
- une autre plateforme devient nécessaire.
