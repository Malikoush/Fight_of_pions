-- E4k : un compte sur plusieurs appareils, et son code de récupération.

-- Les jetons d'un compte, un par appareil : celui de sa création, puis un de plus à chaque
-- appareil qui le retrouve par son code. Comme avant, seule l'empreinte (SHA-256) est
-- gardée. `accounts.token_hash` reste, rempli à la création : c'est le premier jeton,
-- recopié ici ; c'est cette table qui fait foi pour se connecter.
CREATE TABLE account_tokens (
    token_hash TEXT PRIMARY KEY,
    account_id INTEGER NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

INSERT INTO account_tokens (token_hash, account_id, created_at)
SELECT token_hash, id, created_at FROM accounts;

-- L'empreinte du code de récupération du compte, s'il en a un : le code lui-même n'est
-- montré qu'une fois, à sa création. En créer un nouveau remplace l'ancien.
ALTER TABLE accounts ADD COLUMN recovery_hash TEXT;
CREATE UNIQUE INDEX accounts_by_recovery ON accounts (recovery_hash)
    WHERE recovery_hash IS NOT NULL;
