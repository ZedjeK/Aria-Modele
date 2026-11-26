# 0005 : aucune télémétrie

Date : 2026-11-26.

## Contexte

Mesurer l'usage ou remonter les plantages automatiquement aiderait à corriger, mais enverrait des données de
l'utilisateur hors de son poste, sans qu'il le voie.

## Décision

Aucune donnée n'est envoyée à l'auteur ni à un tiers. Les seules connexions sortantes d'une appli sont celles de sa
fiche projet, destination par destination ; celles de toute la famille sont la recherche de mise à jour (qu'une case
des paramètres coupe) et le téléchargement d'une mise à jour, sur un clic. Pour signaler un bug, l'utilisateur copie
lui-même le diagnostic de l'appli (page Journal) et le colle dans le formulaire du dépôt.

## Conséquences

- Aucune mesure d'usage, aucun rapport de plantage automatique : un bug n'est connu que s'il est signalé.
- Le diagnostic ne contient aucun secret, et le chemin du profil y est masqué ; l'utilisateur le relit avant de
  l'envoyer.

## À revoir si

- une appli a besoin d'un service en ligne pour sa fonction même : la connexion entre alors dans sa fiche, avec
  l'accord de l'utilisateur. Jamais pour mesurer l'usage.
