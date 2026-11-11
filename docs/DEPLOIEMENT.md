# Déploiement

Comment Fight of Pions est mis en ligne, et comment l'installer sur l'OptiPlex (Ubuntu Server). Tâche E4h de la feuille de route.

## Le principe

```
git push (main)
   │
   ▼
GitHub Actions (.github/workflows/deployer.yml)
   ├─ verifier : format, clippy, tests du moteur et du serveur
   ├─ jeu-web  : tests du client, export web avec Godot, précompression
   └─ image    : l'image Docker complète (serveur + jeu web), si les deux passent
                 → ghcr.io/malikoush/fight_of_pions:latest  (et :sha-1a2b3c4)
   │
   ▼
OptiPlex : Watchtower voit la nouvelle image (toutes les 5 min),
           la télécharge et relance le serveur
```

- Sur l'OptiPlex, un seul fichier : `docker-compose.yml`. Ni le code source, ni Rust, ni Godot.
- Rien n'est compilé sur l'OptiPlex, et le serveur n'a jamais plus d'un cœur du processeur (`cpus: 1`).
- Une étape qui échoue (un test, une erreur de script dans l'export, la compilation) arrête tout : rien n'est publié, l'ancienne version continue de tourner, et GitHub envoie un mail.
- Une mise à jour redémarre le serveur : les parties finies sont enregistrées avant l'arrêt, **celles en cours sont perdues**. Les comptes et les cartes, dans le volume de la base, sont gardés.
- Compter 10 à 20 minutes entre le push et la mise en ligne : un changement du code Rust fait recompiler tout le serveur, dépendances comprises ; un changement du seul jeu web est plus rapide. Le déroulé se suit dans l'onglet **Actions** du dépôt.
- Watchtower reçoit le socket de Docker (`/var/run/docker.sock`) : c'est ce qui lui permet de remplacer le conteneur, et cela vaut un accès administrateur à la machine. C'est son fonctionnement normal ; il ne met à jour que le conteneur qui porte son étiquette, le serveur.

## Mise en place, une seule fois

### 1. Sur GitHub

1. Faire un premier push sur `main` (ou lancer le workflow à la main : onglet **Actions**, « Déployer », **Run workflow**).
2. Attendre qu'il soit vert. L'image apparaît dans les **Packages** du compte (`fight_of_pions`).
3. **Rendre l'image publique**, sinon l'OptiPlex ne peut pas la télécharger sans se connecter : page du package, **Package settings**, **Change visibility**, **Public**. Le code du jeu y est déjà public, puisque le dépôt l'est.

Si le dépôt redevient privé un jour, l'image peut rester privée : il faudra alors connecter l'OptiPlex au registre une fois (`docker login ghcr.io` avec un jeton GitHub qui a le droit `read:packages`), et le donner à Watchtower (voir sa documentation, « registry authentication »).

### 2. Sur l'OptiPlex

La machine (analysée le 2026-11-11) : `home-server`, Ubuntu 26.04, 8 threads, 7 Go de mémoire, presque au repos. Le disque système (NVMe, où Docker range ses volumes, donc la base) a 193 Go libres (agrandi à tout le NVMe le 2026-11-11) ; le disque des médias (`/mnt/media`) servira aux sauvegardes (E4i). Les services déjà là sont rangés dans `/opt/media-stack` (Jellyfin…) ; le port 3000 est libre ; Funnel sert déjà Jellyfin sur le port 443.

Docker est déjà installé. Pour s'en servir sans `sudo`, une fois (puis se déconnecter et se reconnecter) :

```
sudo usermod -aG docker $USER
```

Le jeu, rangé à côté des autres services :

```
sudo mkdir -p /opt/fight-of-pions
sudo chown $USER: /opt/fight-of-pions
cd /opt/fight-of-pions
curl -fsSL -o docker-compose.yml https://raw.githubusercontent.com/Malikoush/Fight_of_pions/main/docker-compose.yml
docker compose up -d
docker compose ps                  # le serveur doit passer « healthy » en moins d'une minute
curl -s http://127.0.0.1:3000/health
```

L'ouvrir sur internet avec Tailscale Funnel. Le port 443 sert déjà Jellyfin : le jeu prend le 8443 (Funnel n'accepte que 443, 8443 et 10000) :

```
sudo tailscale funnel --bg --https=8443 3000
tailscale funnel status            # https://home-server.tail663aab.ts.net:8443 : l'adresse à donner aux amis
```

Jellyfin reste sur `https://home-server.tail663aab.ts.net`, sans changement. Un réseau qui bloque les ports autres que 443 (certains Wi-Fi d'école ou d'entreprise) ne laissera pas passer le 8443 : sur un téléphone en 4G, aucun souci.

## Au quotidien

Les commandes `docker compose` se lancent depuis `/opt/fight-of-pions`.

| Je veux… | Je fais… |
|---|---|
| Mettre le jeu à jour | `git push` sur `main`. C'est tout. |
| Voir où en est une mise à jour | Onglet **Actions** du dépôt sur GitHub |
| Savoir si le serveur va bien | `docker compose ps` (« healthy ») |
| Lire ce qu'écrit le serveur | `docker compose logs -f serveur` |
| Savoir si Watchtower a mis à jour | `docker compose logs watchtower` |
| Redémarrer le serveur | `docker compose restart serveur` |
| Arrêter le jeu | `docker compose down` (sans `-v` : la base est gardée) |
| Donner une carte, voir les comptes, sauvegarder | `docker compose exec serveur serveur admin aide` (voir « La base ») |

**Revenir à une version précédente** : dans l'onglet **Actions** ou la page du package, repérer l'étiquette `sha-…` de la version voulue, et la mettre à la place de `latest` dans `docker-compose.yml` (`image: ghcr.io/malikoush/fight_of_pions:sha-1a2b3c4`), puis `docker compose up -d`. Watchtower ne touche pas à une étiquette `sha-…`, qui ne change jamais. Remettre `latest` pour reprendre les mises à jour.

**Quand `docker-compose.yml` lui-même change** dans le dépôt (rare), le retélécharger avec la même commande `curl`, puis `docker compose up -d`.

## La base

- Elle vit dans le volume Docker `fight-of-pions_donnees` (fichier `/data/fight_of_pions.db` dans le conteneur). Elle survit aux mises à jour, aux redémarrages et aux coupures de courant.
- **Ne jamais lancer `docker compose down -v`** ni `docker volume rm` : le volume, et avec lui tous les comptes et leurs cartes, serait effacé.

### L'administrer (E4i)

Le serveur a ses commandes d'administration : elles passent par son code (une carte doit être du catalogue en ligne, un pseudo suit les règles du jeu), jamais par du SQL tapé à la main, et marchent pendant qu'il tourne.

| Commande | Ce qu'elle fait |
|---|---|
| `comptes` | Tous les comptes, le plus récemment vu en premier : numéro, pseudo, dernière visite, cartes, parties |
| `collection 3` | Les cartes et la cache du compte n° 3 |
| `donner 3 MassacreDivin Legendaire` | Donne une carte (majuscules et accents indifférents ; un nom en deux mots s'écrit attaché, ou entre guillemets) |
| `donner-tout 3` | Tout le catalogue en ligne |
| `retirer 3 Ours Epique` | Retire une carte (et de la cache) |
| `renommer 3 Ana` | Change le pseudo |
| `code 3` | Un code de récupération neuf pour le compte n° 3 (il remplace l'ancien) : à donner au joueur qui a perdu son compte, qui le tape dans Réglages, « Retrouver mon compte » |
| `parties` ou `parties 50` | Les dernières parties finies : qui, contre qui, qui gagne, comment |
| `sauvegarder` | Une sauvegarde tout de suite |
| `aide` | La liste des commandes |

Un joueur connecté voit ce qui change (cartes, pseudo) en rechargeant la page.

**Sur l'OptiPlex** :

```
cd /opt/fight-of-pions
docker compose exec serveur serveur admin comptes
```

Pour taper moins, une fois, dans `~/.bashrc` : `alias fop='docker compose -f /opt/fight-of-pions/docker-compose.yml exec serveur serveur admin'`, puis `fop comptes`, `fop donner 3 Ours Rare`…

**Depuis le PC**, avec `outils/admin.ps1` (par SSH). Une fois, dans PowerShell, puis rouvrir le terminal :

```
[Environment]::SetEnvironmentVariable("FIGHT_SERVEUR", "utilisateur@adresse-du-serveur", "User")
```

Ensuite, depuis le dépôt : `.\outils\admin.ps1 comptes`, `.\outils\admin.ps1 donner 3 Ours Rare`… (un compte par son seul numéro : dans PowerShell, `#3` est un commentaire). Sans clé SSH, le mot de passe est demandé à chaque commande ; pour ne plus le taper, une fois : `type $env:USERPROFILE\.ssh\id_rsa.pub | ssh utilisateur@adresse-du-serveur "cat >> ~/.ssh/authorized_keys"`.

### Les sauvegardes

- **Chaque nuit à 2 h UTC** (3 ou 4 h à Paris), le serveur copie la base dans `/mnt/media/sauvegardes/fight-of-pions` : un autre disque que la base, qui survit si le NVMe meurt. Les **7 dernières** sont gardées (`fight_of_pions-2026-11-12_020000.db`…), les plus vieilles effacées. La copie est cohérente même pendant une partie (SQLite, `VACUUM INTO`). Réglages : `BACKUP_HOUR_UTC` et `BACKUPS_KEPT` dans `serveur/src/limits.rs`, le dossier dans `docker-compose.yml`.
- **Une fois, avant la mise à jour qui l'apporte**, créer le dossier et le donner à l'utilisateur du conteneur (n° 10001), puis reprendre le `docker-compose.yml` du dépôt :

```
sudo mkdir -p /mnt/media/sauvegardes/fight-of-pions
sudo chown 10001:10001 /mnt/media/sauvegardes/fight-of-pions
cd /opt/fight-of-pions
curl -fsSL -o docker-compose.yml https://raw.githubusercontent.com/Malikoush/Fight_of_pions/main/docker-compose.yml
docker compose up -d
docker compose exec serveur serveur admin sauvegarder    # une première copie, pour vérifier
ls -l /mnt/media/sauvegardes/fight-of-pions
```

  Avant le `mkdir`, vérifier que le disque des médias est bien monté (`findmnt /mnt/media` doit répondre) : sinon le dossier serait créé sur le disque système, sans erreur. Il doit aussi accepter les droits Linux : `df -T /mnt/media` dit `ext4` ; un disque `ntfs` ou `exfat` ignore `chown`, il faut alors un autre dossier. Si `sauvegarder` échoue, `docker compose logs serveur` dit pourquoi (dossier absent, interdit).
- Les commandes d'administration se lancent avec l'utilisateur du conteneur, comme ci-dessus : **jamais `docker compose exec -u root`**, qui laisserait à la base des fichiers que le serveur ne pourrait plus écrire.
- **Revenir à une sauvegarde** (la base actuelle est gardée à côté, en `fight_of_pions.db.avant-restauration`) :

```
cd /opt/fight-of-pions
docker compose stop serveur
docker run --rm -v fight-of-pions_donnees:/data -v /mnt/media/sauvegardes/fight-of-pions:/sauvegardes:ro debian:bookworm-slim \
  sh -c 'mv /data/fight_of_pions.db /data/fight_of_pions.db.avant-restauration && rm -f /data/fight_of_pions.db-wal /data/fight_of_pions.db-shm && cp /sauvegardes/fight_of_pions-2026-11-12_020000.db /data/fight_of_pions.db && chown 10001:10001 /data/fight_of_pions.db'
docker compose start serveur
```

  (en remplaçant le nom par celui de la sauvegarde voulue : `ls /mnt/media/sauvegardes/fight-of-pions`).

## Si ça coince

| Ce qu'on voit | Où regarder |
|---|---|
| GitHub envoie un mail d'échec | Onglet **Actions**, l'étape en rouge : son journal dit quel test ou quelle commande a échoué. Rien n'a changé sur l'OptiPlex. |
| `docker compose ps` dit « unhealthy » ou redémarre en boucle | `docker compose logs serveur` : la base illisible, un port déjà pris… |
| La page ne s'ouvre pas de l'extérieur | `tailscale funnel status`, et `curl -s http://127.0.0.1:3000/health` sur l'OptiPlex |
| Watchtower ne met pas à jour | `docker compose logs watchtower` ; l'image est-elle bien publique ? |
| Pas de sauvegarde de la nuit | `docker compose logs serveur` (« sauvegarde ratée… ») ; le dossier existe-t-il, et appartient-il à 10001 ? |
| Le jeu affiche une ancienne version | Recharger la page sans le cache du navigateur (Ctrl+F5) |

## Construire l'image à la main

Rarement utile, puisque GitHub le fait. Depuis la racine du dépôt, avec Docker :

```
docker build -t fight-of-pions .                   # export web déjà fait et précompressé
docker build --target serveur -t fight-of-pions .  # le serveur seul, sans le jeu web
```
