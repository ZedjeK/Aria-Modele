# Politique de signature du code (Code signing policy)

Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by
[SignPath Foundation](https://signpath.org).

L'exe d'Aria {{NOM}} est signé par Authenticode, avec un certificat de la SignPath Foundation, une fois l'appli
acceptée par la fondation. Windows affiche alors « SignPath Foundation » comme éditeur. Tant que la signature n'est pas
active pour cette appli, l'exe n'est pas signé et Windows peut afficher un avertissement au premier lancement : voir
« Si ça ne marche pas » dans le [README](../README.md).

## Ce qui est signé

- Seulement `Aria-{{NOM}}.exe`, construit par le workflow `Release` de ce dépôt sur les machines de GitHub, depuis le
  code source du tag de la version (jamais sur un essai à blanc). Aucun exe construit sur un poste n'est signé.
- La signature est faite avant le zip, son empreinte SHA-256 et l'attestation de provenance : les trois portent sur
  l'exe signé.
- Chaque demande de signature est approuvée à la main dans SignPath par un approbateur avant d'aboutir.

## Rôles

| Rôle | Qui | Ce qu'il fait |
| --- | --- | --- |
| Auteurs (committers) | ZedjeK | écrivent et fusionnent le code, par des pull requests |
| Relecteurs (reviewers) | ZedjeK | relisent toute modification proposée par une autre personne avant sa fusion |
| Approbateurs (approvers) | ZedjeK | approuvent chaque demande de signature dans SignPath |

Chaque membre de l'équipe utilise l'authentification à plusieurs facteurs sur GitHub et sur SignPath.

## Vie privée (privacy policy)

Aria {{NOM}} n'envoie aucune donnée à son auteur ni à un tiers : aucune télémétrie. Ses connexions sortantes sont
listées une à une dans [docs/FICHE.md](FICHE.md) et dans le README ; pour toute appli de la famille :

- au lancement, une requête à `api.github.com` pour chercher une nouvelle version : elle ne transmet que le nom et la
  version de l'appli (GitHub voit aussi, comme pour toute requête, l'adresse IP) ; une case des paramètres la coupe ;
- le téléchargement d'une mise à jour depuis `github.com`, seulement sur un clic de l'utilisateur.

Privacy: this program sends no telemetry and no user data. Its outgoing connections are listed one by one in
docs/FICHE.md; every app of the family checks GitHub for updates at startup (sending only the app name and version;
this can be turned off in the settings) and downloads an update only on the user's click.

## Comportement de l'appli

- Aucune modification du système sans l'accord de l'utilisateur : le démarrage avec Windows, l'installation et le
  raccourci du Bureau sont proposés, jamais imposés.
- Désinstallation : un bouton « Désinstaller » dans les paramètres retire ce que l'appli a écrit hors de son dossier
  de données ; voir « Désinstaller » dans le README.
