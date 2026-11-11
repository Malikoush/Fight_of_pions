# syntax=docker/dockerfile:1
# Le serveur de Fight of Pions, en conteneur.
#
#   docker build -t fight-of-pions .
#   docker run -p 3000:3000 -v fight-of-pions-data:/data \
#     -v "$PWD/client-godot/export/web:/web:ro" fight-of-pions
#
# La base des comptes est dans le volume /data : sans lui, elle disparaît à chaque nouvelle
# version du conteneur, avec les comptes et leurs cartes. L'export web du client est monté
# sur /web et servi sur `/` : le changer ne demande pas de reconstruire l'image ; sans lui,
# le serveur ne sert que la WebSocket. Avant de le monter, `python outils/precompresser_web.py`
# compresse une fois pour toutes ses gros fichiers.

# 1. La compilation. Tout le workspace est copié (le serveur dépend du moteur et des bots, et
#    Cargo lit les manifestes de tous les membres), mais seul le serveur est compilé. Les
#    dépendances téléchargées et déjà compilées sont gardées d'une construction à l'autre
#    (caches de BuildKit) : une mise à jour ne recompile que ce qui a changé.
FROM rust:1-bookworm AS build
WORKDIR /src
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked -p serveur \
    && cp target/release/serveur /usr/local/bin/serveur

# 2. L'image finale : le binaire seul. Les textes des cartes, le paquet de départ, les
#    migrations et SQLite y sont compris.
FROM debian:bookworm-slim
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
