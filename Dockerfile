# Le serveur de Fight of Pions, en conteneur.
#
#   docker build -t fight-of-pions .
#   docker run -p 3000:3000 -v fight-of-pions-data:/data fight-of-pions
#
# La base des comptes est dans le volume /data : sans lui, elle disparaît à chaque nouvelle
# version du conteneur, avec les comptes et leurs cartes.

# 1. La compilation. Tout le workspace est copié (le serveur dépend du moteur et des bots, et
#    Cargo lit les manifestes de tous les membres), mais seul le serveur est compilé.
FROM rust:1-bookworm AS build
WORKDIR /src
COPY . .
RUN cargo build --release --locked -p serveur

# 2. L'image finale : le binaire seul. Les textes des cartes, le paquet de départ, les
#    migrations et SQLite y sont compris.
FROM debian:bookworm-slim
RUN useradd --system --uid 10001 pions \
    && mkdir /data \
    && chown pions /data
COPY --from=build /src/target/release/serveur /usr/local/bin/serveur
USER pions
ENV PORT=3000 \
    DATABASE_URL=sqlite:///data/fight_of_pions.db
VOLUME /data
EXPOSE 3000
CMD ["serveur"]
