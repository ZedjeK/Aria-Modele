# 0010 : Windows x64 seulement

Date : 2026-11-27. Précise [0001](0001-stack-unique-rust-tauri.md).

## Contexte

La décision 0001 vise « Windows 11 et Windows 10 22H2, en 64 bits ». Des PC sous Windows 11 avec un processeur ARM
existent aussi : « 64 bits » ne dit pas s'ils sont visés. Les moteurs locaux de certaines applis (llama.cpp, ComfyUI)
devraient alors suivre en ARM64, et chaque release serait construite et testée deux fois.

## Décision

- La seule cible est `x86_64-pc-windows-msvc` : chaque release livre un exe x64, et rien pour ARM64.
- Les moteurs locaux et leurs versions sont choisis pour x64 seulement.
- Le README de chaque appli dit « Windows 10 22H2 ou Windows 11, x64 ».

## Conséquences

- Un seul build, une seule chaîne de tests et de release.
- Sur un PC ARM, l'exe x64 peut tourner par l'émulation de Windows 11, sans aucune garantie : ce cas n'est pas
  testé, et le README le dit dans « Limites à connaître ».

## À revoir si

- un utilisateur sur un PC ARM le demande ;
- un moteur local utilisé par la famille ne fonctionne pas sous émulation et qu'un utilisateur en a besoin ;
- Windows cesse d'émuler les applis x64 sur ARM.
