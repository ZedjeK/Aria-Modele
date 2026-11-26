# 0004 : les mises à jour : release GitHub, SHA-256, attestation, signature minisign

Date : 2026-11-26.

## Contexte

Un exe portable n'a pas de catalogue qui le met à jour : sans mise à jour intégrée, une faille corrigée resterait sur
les postes. Mais une mise à jour automatique est aussi la porte d'entrée idéale pour une version piégée.

## Décision

- L'appli lit `releases/latest` du dépôt de l'appli sur GitHub, propose la nouvelle version, et ne télécharge que
  depuis cette release, sur `github.com`, en HTTPS. Rien ne s'installe sans l'accord de l'utilisateur ; la version
  précédente est gardée dans `sauvegardes\versions\`.
- La taille et l'empreinte SHA-256 du zip sont vérifiées avant l'installation.
- Chaque zip publié porte une attestation de provenance (GitHub), vérifiable avec `gh attestation verify` ; l'appli
  ne la lit pas.
- Chaque zip publié sera signé par minisign avec la clé de la famille, gardée hors ligne par l'auteur, jamais sur
  GitHub ni dans la CI. L'appli embarquera la clé publique principale et une clé de secours, et refusera une mise à
  jour sans signature valide.

État au 2026-11-26 : les trois premiers points sont en place ; la signature minisign est décidée, pas encore en place
(elle attend la création des clés).

## Conséquences

- Tant que la signature minisign n'est pas en place, la provenance repose sur le compte GitHub de l'auteur et sur les
  releases immuables : un compte compromis pourrait proposer une version piégée. Le README de chaque appli le dit
  dans « Limites à connaître ».
- `releases/latest` ignore les prereleases : une version se teste en prerelease avant d'être annoncée à tous.
- Une clé privée volée sur le poste de l'auteur permettrait de signer une version piégée : la clé de secours sert à
  publier une version qui retire la clé compromise.

## À revoir si

- GitHub change le comportement de `releases/latest`, des releases immuables ou des attestations ;
- une clé minisign est perdue ou volée ;
- une autre source de mise à jour devient nécessaire.
