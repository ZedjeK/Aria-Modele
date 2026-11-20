# Registre des ports locaux

Chaque appli Aria à interface web écoute sur la boucle locale (`127.0.0.1`, et `::1` si elle en a besoin), sur un port
fixe et différent de celui des autres. Ce fichier fait foi **dans le dépôt modèle** : `init.ps1` refuse un port qui y
figure, et tu y ajoutes une ligne à la main à chaque nouvelle appli. Il n'est pas copié dans les applis.

Relevé du 18 novembre 2026, dans le code de chaque appli, complété le 20 novembre 2026 (Aria Etabli). Une ligne par
port ; l'adresse d'écoute est notée quand elle a été lue dans le code.

| Port | Appli | Rôle |
| --- | --- | --- |
| 443 | Aria Vault | relais HTTPS du coffre, sur toutes les interfaces : service du réseau local, joint par le VPN hors de chez toi, jamais exposé sur Internet (exception au contrat : appli serveur du réseau local) |
| 8000 | Aria Vault | Vaultwarden, sur `127.0.0.1` seulement, derrière le relais HTTPS |
| 8075 | Aria Studio | interface web, sur `127.0.0.1` (`config.yaml`) |
| 8080 | Aria JDR | serveur llama du MJ, lancé par le jeu pendant une partie ou une qualification, sur `127.0.0.1` seulement ; pas une interface web |
| 8095 | Aria Studio | llama-server, sur `127.0.0.1` |
| 8195 | Aria Studio | ComfyUI, sur `127.0.0.1` |
| 8350 | Aria Etabli | interface web, sur `127.0.0.1` seulement (jeton tiré à chaque lancement, `Host` et `Origin` vérifiés) |
| 8420 | Aria JDR | interface web, sur `127.0.0.1` seulement (jeton tiré à chaque lancement, `Host` et `Origin` vérifiés) |
| 8700 | Aria Pilote | interface web, version installée (adresse d'écoute non relevée) |
| 8701 | Aria Pilote | serveur llama, version installée (adresse d'écoute non relevée) |
| 8710 | Aria Pilote | interface web, version de développement (`tools/build_portable.py`) |
| 8711 | Aria Pilote | serveur llama, version de développement |
| 8888 | Aria Pilote | SearXNG, version installée, sur `127.0.0.1` |
| 8898 | Aria Pilote | SearXNG, version de développement |
| 47823 | Aria Plume | pages HTTPS des compléments Word et Outlook, sur `127.0.0.1` et `::1` (certificat local, `Host` et `Origin` vérifiés) |

Ports retirés le 18 novembre 2026 : 3000 (simple exemple dans les tests de Pilote, pas un port de l'appli) et 8000 pour
Pilote (une constante de longueur de texte, pas un port).
