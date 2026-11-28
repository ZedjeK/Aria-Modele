# Recette d'une version

La liste à cocher sur la prerelease, avant de la rendre version courante. Copie-la dans une issue « Recette
vX.Y.Z » et coche au fur et à mesure ; une case qui ne se coche pas bloque la publication, ou se dit dans les notes
de la version.

Fais-la de préférence sur un Windows neuf, sans outils de développement : Windows Sandbox (Windows Pro) ou une machine
virtuelle.

## Téléchargement

- [ ] Le zip se télécharge depuis la prerelease, et son SHA-256 correspond au fichier `.sha256` joint.
- [ ] `gh attestation verify <zip> --repo <propriétaire>/<dépôt>` réussit.
- [ ] Les notes de la version disent « décompresse d'abord le zip », les limites, et l'avertissement SmartScreen tant
  que l'exe n'est pas signé.

## Installation

- [ ] Lancé depuis l'aperçu du zip dans l'Explorateur, l'exe demande de décompresser d'abord et ne s'installe pas.
- [ ] Zip décompressé : l'exe propose de s'installer ; refusé, l'appli se ferme.
- [ ] Installation acceptée : la fenêtre se ferme, l'appli se relance depuis `programme\` ; le raccourci du Bureau
  n'existe que si la case a été cochée.
- [ ] Un raccourci du même nom qui pointe ailleurs est laissé tel quel.
- [ ] L'assistant de premier lancement se déroule jusqu'au bout, et se rouvre depuis les Paramètres.

## Promesse

- [ ] Le critère de réussite de la fiche (ou l'objectif de la version) se vérifie, sur des données réelles.
- [ ] Une tâche longue montre sa progression et s'annule ; la fenêtre ne fige pas.

## Mise à jour

- [ ] Depuis la version courante précédente, réellement installée : la mise à jour est proposée, s'installe et
  garde l'ancienne version dans `sauvegardes\versions\`.
- [ ] Réglages, journal et secrets sont retrouvés après la mise à jour.
- [ ] Pour la première version sur le socle : les données de l'ancienne version sont reprises, l'ancien dossier
  n'a pas changé, et ce qui n'a pas pu être repris est dit.

## Restauration et désinstallation

- [ ] Un réglage changé laisse au journal une ligne qui dit quel champ a changé, et de quoi à quoi.
- [ ] Une sauvegarde se restaure depuis l'interface, après une sauvegarde de l'état actuel ; avant de restaurer, le
  dialogue montre les réglages qui changeraient.
- [ ] « Désinstaller » retire le démarrage avec Windows, le raccourci et `programme\` ; les données restent, et
  l'appli dit où elles sont.
- [ ] « Désinstaller » avec « Supprimer aussi mes données » cochée : une fois l'appli fermée,
  `%USERPROFILE%\.Aria-<Nom>` a disparu, et `%USERPROFILE%\.Aria` est intact.

## Accessibilité

- [ ] Le parcours principal se fait entièrement au clavier, et le focus se voit partout.
- [ ] Windows réglé sur « moins d'animations » : le fond animé devient une image fixe.

## Modules activés

Seulement ceux que la fiche déclare.

- [ ] Démarrage avec Windows : la case suit l'état réel ; `--silencieux` démarre l'appli dans l'icône de notification.
- [ ] Moteur local : le moteur démarre, répond, et une mise à jour hors de la plage acceptée n'est pas proposée.
- [ ] Notifications : elles se coupent dans les réglages.
- [ ] Raccourcis globaux : un conflit avec un autre programme est signalé.
- [ ] Fichiers : un fichier réécrit a sa copie dans `sauvegardes\` ; une association n'est posée qu'avec accord et
  part à la désinstallation.
