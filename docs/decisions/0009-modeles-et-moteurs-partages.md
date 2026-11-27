# 0009 : les modèles et les moteurs locaux partagés entre les applis

Date : 2026-11-27.

## Contexte

Le contrat range tout ce qu'une appli écrit dans son propre dossier, `%USERPROFILE%\.Aria-<Nom>`. Au 2026-11-27,
Etabli, JDR, Pilote et Studio utilisent des modèles GGUF et llama.cpp, et Studio aussi ComfyUI. Chaque appli garde
ses propres copies : un même modèle de plusieurs dizaines de Go peut exister en trois exemplaires, et chaque appli
installe et met à jour son propre moteur.

## Décision

- Les modèles et les moteurs locaux vivent dans un dossier commun à la famille, `%USERPROFILE%\.Aria` :
  `modeles\` pour les modèles, `moteurs\<moteur>\<version>\` pour les moteurs. C'est une exception écrite à la règle
  du dossier de données : ce dossier ne contient ni réglage, ni secret, ni donnée personnelle.
- Toute appli qui utilise un modèle ou un moteur passe par ce dossier, au moyen du module « moteur local » du socle.
  L'utilisateur peut placer ce dossier sur un autre disque, une fois, pour toute la famille.
- Un registre commun dans ce dossier dit quelle appli utilise quel modèle et quelle version de moteur. Un modèle ou
  un moteur ne s'efface que sur demande de l'utilisateur, après lui avoir dit quelles applis s'en servent.
- Chaque appli déclare la plage de versions d'un moteur qu'elle accepte. Une mise à jour du moteur n'est proposée que
  dans cette plage ; une version majeure attend une version de l'appli qui l'a validée. Après chaque mise à jour, un
  essai vérifie que le moteur démarre et répond ; s'il échoue, la version précédente reprend sa place.
- Les versions de moteur coexistent côte à côte : deux applis peuvent utiliser deux versions différentes.
- Tout fichier téléchargé dans ce dossier passe par le module « téléchargement vérifié » : source épinglée, taille
  et SHA-256 contrôlés avant usage.
- Une seule appli écrit à la fois dans le dossier commun, sous un verrou ; la lecture reste libre.
- Une appli qui tourne élevée ne lance aucun moteur depuis ce dossier, que n'importe quel programme de l'utilisateur
  peut modifier : elle garde ses moteurs dans son propre dossier protégé.

## Conséquences

- Un modèle n'est téléchargé et stocké qu'une fois pour toute la famille.
- Le contrat gagne une exception, et le guide utilisateur commun doit dire où sont les modèles et comment les
  déplacer.
- Les applis partagent aussi les risques : un moteur cassé touche toutes celles qui l'utilisent. La plage de
  versions déclarée et l'essai après mise à jour limitent ce risque.
- Un programme lancé sous le même compte peut modifier un modèle ou un moteur du dossier commun, comme il peut
  modifier le dossier d'une appli : les empreintes vérifiées au téléchargement ne protègent pas après coup.
- Les applis élevées (Cockpit, Shield) ne profitent pas du partage des moteurs.

## À revoir si

- un moteur ou un format de modèle ne peut pas être partagé entre deux applis ;
- les conflits de versions de moteur deviennent fréquents ;
- une appli a besoin de modifier un modèle partagé au lieu de le lire.
