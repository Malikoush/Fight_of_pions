-- E4c : les parties finies, et la carte qu'une victoire fait gagner.

-- Une partie finie. `record` est la partie entière en JSON (configuration, graine, coups :
-- le `GameRecord` du moteur), de quoi la rejouer à l'identique ; `ending` est sa fin en JSON
-- (résultat et raison). Une place est tenue par `human` (avec son compte, s'il en a un), un
-- bot (`mcts`, `random`…) ou `nobody`. `winner` vaut 1 ou 2, NULL pour un nul. La graine est
-- en texte : elle peut dépasser un entier signé de SQLite.
CREATE TABLE games (
    id              INTEGER PRIMARY KEY,
    variant         TEXT NOT NULL,
    seed            TEXT NOT NULL,
    player1_kind    TEXT NOT NULL,
    player1_account INTEGER REFERENCES accounts (id) ON DELETE SET NULL,
    player2_kind    TEXT NOT NULL,
    player2_account INTEGER REFERENCES accounts (id) ON DELETE SET NULL,
    winner          INTEGER CHECK (winner IN (1, 2)),
    ending          TEXT NOT NULL,
    moves           INTEGER NOT NULL,
    record          TEXT NOT NULL,
    started_at      TEXT NOT NULL,
    ended_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX games_by_player1 ON games (player1_account);
CREATE INDEX games_by_player2 ON games (player2_account);

-- La partie qui a fait gagner une carte (`source` = `victory`). Une partie en fait gagner
-- au plus une : l'index unique l'assure, même si l'enregistrement était tenté deux fois.
ALTER TABLE card_grants ADD COLUMN game_id INTEGER REFERENCES games (id);
CREATE UNIQUE INDEX card_grants_one_per_game ON card_grants (game_id) WHERE game_id IS NOT NULL;
