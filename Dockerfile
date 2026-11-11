# syntax=docker/dockerfile:1
# Fight of Pions en conteneur : le serveur, et le jeu web qu'il sert sur `/`.
#
# L'image publiée est construite par GitHub à chaque push sur `main`
# (.github/workflows/deployer.yml, docs/DEPLOIEMENT.md) : le workflow exporte d'abord le
# jeu web avec Godot dans client-godot/export/web, puis construit cette image, jeu compris.
#
# À la main :
#   docker build -t fight-of-pions .                   le serveur et le jeu (export déjà fait)
#   docker build --target serveur -t fight-of-pions .  le serveur seul, sans le jeu web
#
# La base des comptes est dans le volume /data : sans lui, elle disparaît à chaque nouvelle
# version du conteneur, avec les comptes et leurs cartes.

# 1. La compilation. Seuls les manifestes et les sources Rust du workspace sont copiés (le
#    serveur dépend du moteur et des bots, et Cargo lit les manifestes de tous les
#    membres) : un changement du jeu web ne recompile pas le serveur. Seul le serveur est
#    compilé. Les dépendances téléchargées et déjà compilées sont gardées d'une
#    construction à l'autre sur une même machine (caches de BuildKit). Sur GitHub, ces
#    caches ne sont pas gardés : un changement du code Rust recompile tout le serveur,
#    dépendances comprises (quelques minutes) ; un changement du seul jeu web, rien.
FROM rust:1-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY moteur moteur
COPY bots bots
COPY serveur serveur
COPY terminal terminal
COPY labo labo
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked -p serveur \
    && cp target/release/serveur /usr/local/bin/serveur

# 2. Le serveur seul : le binaire. Les textes des cartes, le paquet de départ, les
#    migrations et SQLite y sont compris. Sans jeu web dans /web, il ne sert que la
#    WebSocket (un dossier peut y être monté : `-v …/export/web:/web:ro`).
FROM debian:bookworm-slim AS serveur
RUN useradd --system --uid 10001 pions \
    && mkdir /data \
    && chown pions /data
COPY --from=build /usr/local/bin/serveur /usr/local/bin/serveur
USER pions
ENV PORT=3000 \
    DATABASE_URL=sqlite:///data/fight_of_pions.db \
    WEB_DIR=/web
VOLUME /data
EXPOSE 3000
# Le serveur répond-il ? Le binaire le vérifie lui-même : aucun outil de plus dans l'image.
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD ["serveur", "--health"]
# `docker stop` envoie SIGTERM : le serveur ferme les connexions et finit d'enregistrer les
# parties avant de s'arrêter.
STOPSIGNAL SIGTERM
CMD ["serveur"]

# 3. L'image complète (celle par défaut, celle qui est publiée) : le serveur et le jeu web,
#    déjà exporté et précompressé (outils/precompresser_web.py) dans client-godot/export/web.
#    Les fichiers appartiennent à root : le serveur les lit, il ne peut pas les modifier.
FROM serveur AS complet
COPY client-godot/export/web /web
