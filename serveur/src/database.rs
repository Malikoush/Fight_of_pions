//! La base du serveur (E4a) : SQLite, un seul fichier. Elle garde les comptes, leurs cartes
//! et leur trio de cache ; les parties, elles, restent en mémoire. Le schéma est dans
//! `migrations/`, appliqué à l'ouverture.
//!
//! Une carte s'écrit en base comme dans le protocole : sa famille (`BeauteFatale`) et sa
//! rareté (`Epic`), jamais un numéro d'ordre, qui changerait avec le catalogue.

use crate::ServerError;
use crate::account::Account;
use crate::collection::Collection;
use crate::random::random_u64;
use crate::room::FinishedGame;
use fight_of_pions::{Attack, AttackKind, Player, Rarity};
use serde::Serialize;
use serde::de::DeserializeOwned;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::{SqlitePool, migrate::Migrator};
use std::str::FromStr;
use std::time::{Duration, UNIX_EPOCH};

/// Les migrations, comprises dans le serveur à la compilation.
static MIGRATIONS: Migrator = sqlx::migrate!();

/// Le temps qu'une écriture attend qu'une autre finisse avant d'abandonner.
const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

/// La base, partagée par toutes les connexions (elle se clone sans copier).
#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Ouvre la base de cette adresse (`sqlite://fight_of_pions.db`), la crée au besoin, et
    /// met son schéma à jour.
    pub async fn open(url: &str) -> Result<Database, sqlx::Error> {
        let options = SqliteConnectOptions::from_str(url)?
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal)
            .busy_timeout(BUSY_TIMEOUT);
        let pool = SqlitePoolOptions::new().connect_with(options).await?;
        Database::migrated(pool).await
    }

    /// Une base neuve en mémoire, perdue à la fin : pour les tests. Une seule connexion, gardée
    /// ouverte : chaque connexion à `:memory:` aurait sa propre base.
    pub async fn in_memory() -> Result<Database, sqlx::Error> {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")?.foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .min_connections(1)
            .idle_timeout(None)
            .max_lifetime(None)
            .connect_with(options)
            .await?;
        Database::migrated(pool).await
    }

    async fn migrated(pool: SqlitePool) -> Result<Database, sqlx::Error> {
        MIGRATIONS.run(&pool).await?;
        Ok(Database { pool })
    }

    /// Crée un compte avec ce pseudo et l'empreinte de son jeton, et lui donne ces cartes de
    /// départ, en une seule transaction.
    pub async fn create_account(
        &self,
        pseudo: &str,
        token_hash: &str,
        starter: &[Attack],
    ) -> Result<Account, ServerError> {
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO accounts (pseudo, token_hash) VALUES (?, ?) RETURNING id",
        )
        .bind(pseudo)
        .bind(token_hash)
        .fetch_one(&mut *transaction)
        .await
        .map_err(storage)?;
        for &card in starter {
            let (family, rarity) = card_names(card)?;
            sqlx::query(
                "INSERT INTO card_grants (account_id, family, rarity, source) \
                 VALUES (?, ?, ?, 'starter')",
            )
            .bind(id)
            .bind(family)
            .bind(rarity)
            .execute(&mut *transaction)
            .await
            .map_err(storage)?;
        }
        transaction.commit().await.map_err(storage)?;
        Ok(Account {
            id,
            pseudo: pseudo.to_string(),
        })
    }

    /// Le compte de ce jeton (par son empreinte), s'il existe ; sa dernière visite est notée.
    pub async fn account_by_token(&self, token_hash: &str) -> Result<Option<Account>, ServerError> {
        let row: Option<(i64, String)> = sqlx::query_as(
            "UPDATE accounts SET last_seen = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') \
             WHERE token_hash = ? RETURNING id, pseudo",
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(storage)?;
        Ok(row.map(|(id, pseudo)| Account { id, pseudo }))
    }

    /// Change le pseudo de ce compte (déjà vérifié).
    pub async fn set_pseudo(&self, account: i64, pseudo: &str) -> Result<(), ServerError> {
        sqlx::query("UPDATE accounts SET pseudo = ? WHERE id = ?")
            .bind(pseudo)
            .bind(account)
            .execute(&self.pool)
            .await
            .map_err(storage)?;
        Ok(())
    }

    /// Ce que possède ce compte. Une carte que le serveur ne connaît plus (renommée, retirée)
    /// est laissée en base mais n'est pas rendue.
    pub async fn collection(&self, account: i64) -> Result<Collection, ServerError> {
        let cards: Vec<(String, String)> =
            sqlx::query_as("SELECT family, rarity FROM card_grants WHERE account_id = ?")
                .bind(account)
                .fetch_all(&self.pool)
                .await
                .map_err(storage)?;
        let trio: Vec<(String, String)> = sqlx::query_as(
            "SELECT family, rarity FROM cache_trios WHERE account_id = ? ORDER BY slot",
        )
        .bind(account)
        .fetch_all(&self.pool)
        .await
        .map_err(storage)?;
        let mut cards = known_cards(cards);
        cards.sort_unstable();
        Ok(Collection {
            cards,
            cache_trio: known_cards(trio),
        })
    }

    /// Remplace le trio de cache de ce compte (déjà vérifié : des cartes qu'il possède).
    pub async fn set_cache_trio(&self, account: i64, trio: &[Attack]) -> Result<(), ServerError> {
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        sqlx::query("DELETE FROM cache_trios WHERE account_id = ?")
            .bind(account)
            .execute(&mut *transaction)
            .await
            .map_err(storage)?;
        for (slot, &card) in trio.iter().enumerate() {
            let (family, rarity) = card_names(card)?;
            sqlx::query(
                "INSERT INTO cache_trios (account_id, slot, family, rarity) VALUES (?, ?, ?, ?)",
            )
            .bind(account)
            .bind(i64::try_from(slot).unwrap_or(i64::MAX))
            .bind(family)
            .bind(rarity)
            .execute(&mut *transaction)
            .await
            .map_err(storage)?;
        }
        transaction.commit().await.map_err(storage)?;
        Ok(())
    }
}

impl Database {
    /// Enregistre cette partie finie de cette variante et, si son gagnant le mérite
    /// ([`FinishedGame::rewarded`]), lui donne une carte du catalogue de la partie qu'il
    /// n'a pas encore, tirée au hasard ; le tout en une seule transaction. Rend la carte
    /// gagnée : aucune pour un gagnant qui a déjà tout.
    pub async fn record_game(
        &self,
        game: &FinishedGame,
        variant: &str,
    ) -> Result<Option<Attack>, ServerError> {
        let record = serde_json::to_string(&game.record).map_err(|_| ServerError::Storage)?;
        let ending = serde_json::to_string(&game.end).map_err(|_| ServerError::Storage)?;
        let started_at = game
            .started_at
            .duration_since(UNIX_EPOCH)
            .map_or(0, |since| {
                i64::try_from(since.as_secs()).unwrap_or(i64::MAX)
            });
        let [first, second] = game.players;
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO games (variant, seed, player1_kind, player1_account, player2_kind, \
             player2_account, winner, ending, moves, record, started_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, strftime('%Y-%m-%dT%H:%M:%SZ', ?, 'unixepoch')) \
             RETURNING id",
        )
        .bind(variant)
        .bind(game.record.seed.to_string())
        .bind(first.kind())
        .bind(first.account())
        .bind(second.kind())
        .bind(second.account())
        .bind(game.end.winner().map(|winner| match winner {
            Player::One => 1_i64,
            Player::Two => 2,
        }))
        .bind(ending)
        .bind(i64::try_from(game.record.actions.len()).unwrap_or(i64::MAX))
        .bind(record)
        .bind(started_at)
        .fetch_one(&mut *transaction)
        .await
        .map_err(storage)?;

        let mut reward = None;
        if let Some((_, account)) = game.rewarded() {
            let owned: Vec<(String, String)> =
                sqlx::query_as("SELECT family, rarity FROM card_grants WHERE account_id = ?")
                    .bind(account)
                    .fetch_all(&mut *transaction)
                    .await
                    .map_err(storage)?;
            let owned = known_cards(owned);
            let missing: Vec<Attack> = game
                .catalog
                .iter()
                .filter(|card| !owned.contains(card))
                .copied()
                .collect();
            let left = u64::try_from(missing.len()).unwrap_or(u64::MAX);
            if left > 0 {
                let card = missing[usize::try_from(random_u64() % left).unwrap_or(0)];
                let (family, rarity) = card_names(card)?;
                sqlx::query(
                    "INSERT INTO card_grants (account_id, family, rarity, source, game_id) \
                     VALUES (?, ?, ?, 'victory', ?)",
                )
                .bind(account)
                .bind(family)
                .bind(rarity)
                .bind(id)
                .execute(&mut *transaction)
                .await
                .map_err(storage)?;
                reward = Some(card);
            }
        }
        transaction.commit().await.map_err(storage)?;
        Ok(reward)
    }
}

/// Une erreur de la base : notée dans la sortie du serveur, annoncée sans détail au client.
fn storage(error: sqlx::Error) -> ServerError {
    eprintln!("base : {error}");
    ServerError::Storage
}

/// La famille et la rareté d'une carte, telles que le protocole les écrit.
fn card_names(card: Attack) -> Result<(String, String), ServerError> {
    Ok((name_of(&card.kind)?, name_of(&card.rarity)?))
}

fn name_of(value: &impl Serialize) -> Result<String, ServerError> {
    match serde_json::to_value(value) {
        Ok(serde_json::Value::String(name)) => Ok(name),
        _ => Err(ServerError::Storage),
    }
}

/// Les cartes de ces lignes, sauf celles que le serveur ne connaît plus.
fn known_cards(rows: Vec<(String, String)>) -> Vec<Attack> {
    rows.into_iter()
        .filter_map(|(family, rarity)| {
            match (parsed::<AttackKind>(&family), parsed::<Rarity>(&rarity)) {
                (Some(kind), Some(rarity)) => Some(Attack { kind, rarity }),
                _ => {
                    eprintln!("base : carte inconnue ignorée ({family} {rarity})");
                    None
                }
            }
        })
        .collect()
}

fn parsed<T: DeserializeOwned>(name: &str) -> Option<T> {
    serde_json::from_value(serde_json::Value::String(name.to_string())).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::token_hash;
    use crate::collection::starter_cards;

    #[tokio::test]
    async fn an_account_is_created_with_the_starter_cards_and_found_by_its_token() {
        let database = Database::in_memory().await.unwrap();
        let starter = starter_cards().unwrap();
        let created = database
            .create_account("Martin", &token_hash("secret"), starter)
            .await
            .unwrap();
        assert_eq!(created.pseudo, "Martin");
        let found = database
            .account_by_token(&token_hash("secret"))
            .await
            .unwrap();
        assert_eq!(found, Some(created.clone()));
        assert_eq!(
            database
                .account_by_token(&token_hash("autre"))
                .await
                .unwrap(),
            None
        );

        let collection = database.collection(created.id).await.unwrap();
        let mut expected = starter.to_vec();
        expected.sort_unstable();
        assert_eq!(collection.cards, expected);
        assert!(collection.cache_trio.is_empty());

        database.set_pseudo(created.id, "Martin 2").await.unwrap();
        let found = database
            .account_by_token(&token_hash("secret"))
            .await
            .unwrap();
        assert_eq!(
            found.map(|account| account.pseudo).as_deref(),
            Some("Martin 2")
        );
    }

    #[tokio::test]
    async fn a_trio_is_kept_in_order_and_replaced_whole() {
        let database = Database::in_memory().await.unwrap();
        let starter = starter_cards().unwrap();
        let account = database
            .create_account("Martin", &token_hash("secret"), starter)
            .await
            .unwrap();
        let trio = [starter[2], starter[0], starter[1]];
        database.set_cache_trio(account.id, &trio).await.unwrap();
        assert_eq!(
            database.collection(account.id).await.unwrap().cache_trio,
            trio
        );
        database
            .set_cache_trio(account.id, &[starter[3]])
            .await
            .unwrap();
        assert_eq!(
            database.collection(account.id).await.unwrap().cache_trio,
            [starter[3]]
        );
    }

    #[tokio::test]
    async fn the_base_refuses_a_trio_card_that_is_not_owned() {
        // La vérification est faite avant (`Collection::check_trio`) ; la base la refait.
        let database = Database::in_memory().await.unwrap();
        let account = database
            .create_account("Martin", &token_hash("secret"), &[])
            .await
            .unwrap();
        let card = starter_cards().unwrap()[0];
        assert_eq!(
            database.set_cache_trio(account.id, &[card]).await,
            Err(ServerError::Storage)
        );
        assert!(
            database
                .collection(account.id)
                .await
                .unwrap()
                .cache_trio
                .is_empty()
        );
    }

    #[tokio::test]
    async fn two_accounts_cannot_share_a_token() {
        let database = Database::in_memory().await.unwrap();
        database
            .create_account("Un", &token_hash("secret"), &[])
            .await
            .unwrap();
        assert_eq!(
            database
                .create_account("Deux", &token_hash("secret"), &[])
                .await,
            Err(ServerError::Storage)
        );
    }

    #[tokio::test]
    async fn a_deserved_victory_gives_one_missing_card_with_the_game() {
        use crate::room::{Room, Seat};
        let database = Database::in_memory().await.unwrap();
        let starter = starter_cards().unwrap();
        let winner = database
            .create_account("Gagnant", &token_hash("un"), starter)
            .await
            .unwrap();
        let loser = database
            .create_account("Perdant", &token_hash("deux"), starter)
            .await
            .unwrap();
        let seat = |account: &Account| Seat::Human {
            key: account.pseudo.clone(),
            account: Some(account.id),
        };
        let game = Room::won_by_the_first_player([seat(&winner), seat(&loser)])
            .take_finished()
            .unwrap();
        let card = database
            .record_game(&game, "essai")
            .await
            .unwrap()
            .expect("une carte gagnée");
        assert!(!starter.contains(&card), "une carte qu'il n'avait pas");
        assert!(game.catalog.contains(&card));
        assert!(
            database
                .collection(winner.id)
                .await
                .unwrap()
                .cards
                .contains(&card)
        );
        assert!(
            !database
                .collection(loser.id)
                .await
                .unwrap()
                .cards
                .contains(&card)
        );

        let row: (String, i64, Option<i64>, i64, String, i64, String) = sqlx::query_as(
            "SELECT g.variant, g.player1_account, g.winner, g.moves, c.source, c.game_id,              g.record FROM games g JOIN card_grants c ON c.game_id = g.id",
        )
        .fetch_one(&database.pool)
        .await
        .unwrap();
        assert_eq!(row.0, "essai");
        assert_eq!(row.1, winner.id);
        assert_eq!(row.2, Some(1));
        assert_eq!(row.3, i64::try_from(game.record.actions.len()).unwrap());
        assert_eq!(row.4, "victory");
        let record: fight_of_pions::GameRecord = serde_json::from_str(&row.6).unwrap();
        assert_eq!(record, game.record, "la partie se relit telle quelle");
        // La base refuse une deuxième carte pour la même partie.
        let second = sqlx::query(
            "INSERT INTO card_grants (account_id, family, rarity, source, game_id)              VALUES (?, 'Ours', 'Legendary', 'victory', ?)",
        )
        .bind(winner.id)
        .bind(row.5)
        .execute(&database.pool)
        .await;
        assert!(second.is_err());
    }

    #[tokio::test]
    async fn a_game_is_recorded_even_without_a_reward() {
        use crate::room::{Room, Seat};
        use bots::BotKind;
        let database = Database::in_memory().await.unwrap();
        let player = database
            .create_account("Joueur", &token_hash("un"), &[])
            .await
            .unwrap();
        let seat = Seat::Human {
            key: "un".to_string(),
            account: Some(player.id),
        };
        let game = Room::won_by_the_first_player([seat.clone(), Seat::Bot(BotKind::Random)])
            .take_finished()
            .unwrap();
        assert_eq!(database.record_game(&game, "essai").await.unwrap(), None);
        // Qui possède déjà tout le catalogue ne gagne rien non plus.
        let mut owned_all = game.clone();
        owned_all.players[1] = crate::room::Contestant::Bot(BotKind::Mcts);
        for &card in &game.catalog {
            let (family, rarity) = card_names(card).unwrap();
            sqlx::query(
                "INSERT INTO card_grants (account_id, family, rarity, source) VALUES (?, ?, ?, 'admin')",
            )
            .bind(player.id)
            .bind(family)
            .bind(rarity)
            .execute(&database.pool)
            .await
            .unwrap();
        }
        assert_eq!(
            database.record_game(&owned_all, "essai").await.unwrap(),
            None
        );
        let games: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM games")
            .fetch_one(&database.pool)
            .await
            .unwrap();
        assert_eq!(games, 2);
    }

    #[test]
    fn a_card_is_written_with_its_protocol_names_and_read_back() {
        let card = Attack {
            kind: AttackKind::BeauteFatale,
            rarity: Rarity::Epic,
        };
        let (family, rarity) = card_names(card).unwrap();
        assert_eq!((family.as_str(), rarity.as_str()), ("BeauteFatale", "Epic"));
        assert_eq!(
            known_cards(vec![(family, rarity), ("Dragon".into(), "Epic".into())]),
            [card]
        );
    }
}
