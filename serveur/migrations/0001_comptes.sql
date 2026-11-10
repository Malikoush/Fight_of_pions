-- E4a, E4b : les comptes invités, leurs cartes et leur trio de cache.
-- Les dates sont en texte ISO 8601, en UTC. Une carte s'écrit comme dans le protocole :
-- sa famille (`BeauteFatale`) et sa rareté (`Epic`).

-- Un compte invité : un pseudo, et l'empreinte (SHA-256) de son jeton secret. Le jeton
-- lui-même ne quitte jamais le client après sa création.
CREATE TABLE accounts (
    id         INTEGER PRIMARY KEY,
    pseudo     TEXT NOT NULL,
    token_hash TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    last_seen  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- Les cartes obtenues, une ligne par carte : la collection d'un compte, c'est l'ensemble
-- de ses lignes. `source` dit d'où elle vient (`starter` : le paquet de départ ; plus
-- tard, les récompenses).
CREATE TABLE card_grants (
    account_id INTEGER NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    family     TEXT NOT NULL,
    rarity     TEXT NOT NULL,
    source     TEXT NOT NULL,
    granted_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    PRIMARY KEY (account_id, family, rarity)
);

-- Le trio de cache : jusqu'à 3 cartes de sa collection, préparées avant la partie. Au
-- draft, la cache se choisit parmi elles.
CREATE TABLE cache_trios (
    account_id INTEGER NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    slot       INTEGER NOT NULL CHECK (slot BETWEEN 0 AND 2),
    family     TEXT NOT NULL,
    rarity     TEXT NOT NULL,
    PRIMARY KEY (account_id, slot),
    UNIQUE (account_id, family, rarity),
    FOREIGN KEY (account_id, family, rarity)
        REFERENCES card_grants (account_id, family, rarity) ON DELETE CASCADE
);
