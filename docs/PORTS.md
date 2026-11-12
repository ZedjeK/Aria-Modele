# Registre des ports locaux

Chaque appli Aria à interface web écoute sur `127.0.0.1`, sur un port fixe et différent de celui des autres.
Ce fichier fait foi **dans le dépôt modèle** : `init.ps1` refuse un port qui y figure, et tu y ajoutes une ligne à la
main à chaque nouvelle appli. Il n'est pas copié dans les applis.

Relevé du 12 novembre 2026, fait en cherchant les ports dans le code de chaque appli. Le rôle exact de chaque port
reste à préciser.

| Port | Appli | Rôle |
| --- | --- | --- |
| 3000 | Aria Pilote | à préciser |
| 8000 | Aria Vault et Aria Pilote | à préciser : cité dans les deux, conflit possible à vérifier |
| 8075 | Aria Studio | à préciser |
| 8095 | Aria Studio | à préciser |
| 8195 | Aria Studio | à préciser |
| 8710 | Aria Pilote | à préciser |
| 8888 | Aria Pilote | à préciser |
