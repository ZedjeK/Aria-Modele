# 0006 : la signature Authenticode par SignPath Foundation

Date : 2026-11-26.

## Contexte

Un exe non signé déclenche l'avertissement SmartScreen de Windows (« Windows a protégé votre ordinateur ») et inquiète
davantage les antivirus. Un certificat de signature de code au nom de l'auteur est payant chaque année.

## Décision

Les exe sont signés par Authenticode avec un certificat de la SignPath Foundation, gratuit pour les projets open
source, appli par appli, une fois l'appli acceptée par la fondation, en commençant par une appli pilote. La signature
se fait dans le workflow de release, sur les machines de GitHub, avant le zip, l'empreinte et l'attestation, et
chaque demande est approuvée à la main dans SignPath. Le dépôt modèle prépare l'étape, coupée tant que la variable de
dépôt `SIGNPATH_ACTIF` ne vaut pas `true`. Chaque appli publie sa politique de signature
(`docs/POLITIQUE-SIGNATURE.md`).

## Conséquences

- Windows affiche « SignPath Foundation » comme éditeur, et non l'auteur.
- Les conditions de la fondation s'appliquent : licence open source, build vérifiable depuis le code source,
  politique de signature publiée, rôles de l'équipe, authentification à plusieurs facteurs.
- Le jeton SignPath vit dans les secrets du dépôt de l'appli ; il ne sert qu'à envoyer des demandes. Avant
  l'activation, la signature doit passer dans un job séparé, qui n'exécute aucun code du dépôt avant elle.
- Tant qu'une appli n'est pas signée, son README explique l'avertissement SmartScreen.

## À revoir si

- la fondation change ses conditions ou refuse une appli ;
- un certificat au nom de l'auteur devient accessible à un coût raisonnable ;
- Windows change le traitement des exe signés par SmartScreen.
