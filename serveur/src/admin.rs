//! L'administration (E4i) : `serveur admin <commande>`, lancé à côté du serveur qui tourne
//! (dans le conteneur : `docker compose exec serveur serveur admin …`, docs/DEPLOIEMENT.md).
//! Il ouvre la même base (`DATABASE_URL`) : SQLite laisse les deux écrire chacun leur tour.
//!
//! Il passe par le code du serveur, jamais par du SQL tapé à la main : une carte doit être
//! du catalogue en ligne, un pseudo suit les mêmes règles que dans le jeu. Ce qu'il change
//! (cartes, pseudo), un joueur connecté le voit en rechargeant la page.

use crate::account::{checked_pseudo, new_recovery_code, recovery_hash};
use crate::backup;
use crate::database::{Database, GameSummary};
use crate::limits::BACKUPS_KEPT;
use fight_of_pions::{Attack, AttackKind, EndReason, Rarity};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;

/// Les parties montrées par `parties` quand on n'en demande pas un nombre.
const GAMES_SHOWN: i64 = 20;
/// La provenance d'une carte donnée par l'administration, écrite avec elle en base.
const ADMIN_SOURCE: &str = "admin";

/// Les commandes, telles que `serveur admin aide` les montre.
fn help() -> String {
    format!(
        "\
Commandes de serveur admin :
  comptes                               tous les comptes, le plus récemment vu en premier
  collection <compte>                   les cartes et la cache d'un compte
  donner <compte> <famille> <rareté>    donne une carte (ex. donner 3 MassacreDivin Légendaire)
  donner-tout <compte>                  donne tout le catalogue en ligne
  retirer <compte> <famille> <rareté>   retire une carte (et de la cache)
  renommer <compte> <pseudo>            change le pseudo
  code <compte>                         un code de récupération neuf (remplace l'ancien)
  parties [nombre]                      les dernières parties finies ({GAMES_SHOWN} par défaut)
  sauvegarder                           une sauvegarde de la base, tout de suite
Un compte se désigne par son numéro (3 ou #3). Familles et raretés s'écrivent sans souci
des majuscules ni des accents (massacredivin, legendaire) ; un nom en plusieurs mots
s'écrit attaché, ou entre guillemets (\"massacre divin\").
Un joueur connecté voit ce qui change en rechargeant la page."
    )
}

/// Lance cette commande (`args` : les mots après `admin`) sur cette base ; `backups` : le
/// dossier des sauvegardes, s'il y en a un. Rend le texte à afficher, ou l'erreur.
pub async fn administer(
    database: &Database,
    args: &[String],
    backups: Option<&Path>,
) -> Result<String, String> {
    let words: Vec<&str> = args.iter().map(String::as_str).collect();
    match words.as_slice() {
        [] | ["aide"] => Ok(help()),
        ["comptes"] => accounts(database).await,
        ["collection", id] => collection(database, account_id(id)?).await,
        ["donner", id, family, rarity] => {
            give(database, account_id(id)?, card(family, rarity)?).await
        }
        ["donner-tout", id] => give_all(database, account_id(id)?).await,
        ["retirer", id, family, rarity] => {
            take(database, account_id(id)?, card(family, rarity)?).await
        }
        ["renommer", id, pseudo @ ..] if !pseudo.is_empty() => {
            rename(database, account_id(id)?, &pseudo.join(" ")).await
        }
        ["code", id] => recovery_code(database, account_id(id)?).await,
        ["parties"] => games(database, GAMES_SHOWN).await,
        ["parties", count] => {
            let count = count
                .parse()
                .ok()
                .filter(|count: &i64| *count > 0)
                .ok_or_else(|| format!("« {count} » n'est pas un nombre de parties"))?;
            games(database, count).await
        }
        ["sauvegarder"] => back_up(database, backups).await,
        _ => Err(format!(
            "commande inconnue : {}\n\n{}",
            words.join(" "),
            help()
        )),
    }
}

async fn accounts(database: &Database) -> Result<String, String> {
    let accounts = database.accounts().await.map_err(text)?;
    if accounts.is_empty() {
        return Ok("Aucun compte.".to_string());
    }
    let mut lines = vec![format!(
        "{}, le plus récemment vu en premier :",
        counted(accounts.len(), "compte")
    )];
    for account in accounts {
        lines.push(format!(
            "  #{:<4} {:<20} vu {}  créé {}  · {} · {}",
            account.id,
            account.pseudo,
            account.last_seen,
            account.created_at,
            counted(account.cards, "carte"),
            counted(account.games, "partie")
        ));
    }
    Ok(lines.join("\n"))
}

async fn collection(database: &Database, id: i64) -> Result<String, String> {
    let who = who(database, id).await?;
    let owned = database.collection(id).await.map_err(text)?;
    let mut lines = vec![format!("{who} : {}", counted(owned.cards.len(), "carte"))];
    let mut by_family: BTreeMap<AttackKind, Vec<Rarity>> = BTreeMap::new();
    for card in &owned.cards {
        by_family.entry(card.kind).or_default().push(card.rarity);
    }
    for (kind, rarities) in by_family {
        let rarities: Vec<String> = rarities.iter().map(ToString::to_string).collect();
        lines.push(format!("  {kind} : {}", rarities.join(", ")));
    }
    if owned.cache_trio.is_empty() {
        lines.push("Cache : aucune carte rangée (tirée au hasard à chaque partie)".to_string());
    } else {
        let trio: Vec<String> = owned.cache_trio.iter().map(ToString::to_string).collect();
        lines.push(format!("Cache : {}", trio.join(", ")));
    }
    Ok(lines.join("\n"))
}

async fn give(database: &Database, id: i64, card: Attack) -> Result<String, String> {
    let who = who(database, id).await?;
    let added = database
        .grant(id, &[card], ADMIN_SOURCE)
        .await
        .map_err(text)?;
    Ok(if added > 0 {
        format!("{card} donnée à {who}.")
    } else {
        format!("{who} a déjà {card}.")
    })
}

async fn give_all(database: &Database, id: i64) -> Result<String, String> {
    let who = who(database, id).await?;
    let catalog = online_catalog()?;
    let added = database
        .grant(id, &catalog, ADMIN_SOURCE)
        .await
        .map_err(text)?;
    Ok(format!(
        "{who} reçoit {} : il a tout le catalogue en ligne ({}).",
        counted(added, "carte nouvelle"),
        counted(catalog.len(), "carte")
    ))
}

async fn take(database: &Database, id: i64, card: Attack) -> Result<String, String> {
    let who = who(database, id).await?;
    let removed = database.revoke(id, card).await.map_err(text)?;
    Ok(if removed {
        format!("{card} retirée à {who} (et de sa cache).")
    } else {
        format!("{who} n'avait pas {card}.")
    })
}

async fn rename(database: &Database, id: i64, pseudo: &str) -> Result<String, String> {
    let who = who(database, id).await?;
    let pseudo = checked_pseudo(pseudo).map_err(text)?;
    database.set_pseudo(id, &pseudo).await.map_err(text)?;
    Ok(format!("{who} s'appelle maintenant {pseudo}."))
}

/// Un code de récupération neuf pour ce compte, à donner au joueur qui l'a perdu : il le
/// tape dans Réglages, « Retrouver mon compte ».
async fn recovery_code(database: &Database, id: i64) -> Result<String, String> {
    let who = who(database, id).await?;
    let code = new_recovery_code();
    let hash = recovery_hash(&code).ok_or_else(|| "code de récupération mal formé".to_string())?;
    database.set_recovery(id, &hash).await.map_err(text)?;
    Ok(format!(
        "Code de récupération de {who} : {code}\n\
         (il remplace l'ancien ; à taper dans Réglages, « Retrouver mon compte »)"
    ))
}

async fn games(database: &Database, count: i64) -> Result<String, String> {
    let games = database.recent_games(count).await.map_err(text)?;
    if games.is_empty() {
        return Ok("Aucune partie finie.".to_string());
    }
    let lines: Vec<String> = games.iter().map(game_line).collect();
    Ok(lines.join("\n"))
}

/// « #12 · 2026-11-11T16:20:00Z · Ana contre bot mcts · Ana gagne (6 pions alignés) · 84 coups ».
fn game_line(game: &GameSummary) -> String {
    let [first, second] = &game.players;
    let names = [player_name(first), player_name(second)];
    let result = match game.winner {
        Some(1) => format!("{} gagne", names[0]),
        Some(2) => format!("{} gagne", names[1]),
        _ => "match nul".to_string(),
    };
    format!(
        "#{} · {} · {} contre {} · {} ({}) · {} coups",
        game.id,
        game.ended_at,
        names[0],
        names[1],
        result,
        end_reason(&game.ending),
        game.moves
    )
}

/// Qui tenait une place : le pseudo d'un humain, ou ce qu'était la place.
fn player_name((kind, pseudo): &(String, Option<String>)) -> String {
    match (kind.as_str(), pseudo) {
        ("human", Some(pseudo)) => pseudo.clone(),
        ("human", None) => "humain sans compte".to_string(),
        ("nobody", _) => "personne".to_string(),
        (bot, _) => format!("bot {bot}"),
    }
}

/// La raison de la fin, en français, lue dans la fin enregistrée (JSON).
fn end_reason(ending: &str) -> String {
    serde_json::from_str::<serde_json::Value>(ending)
        .ok()
        .and_then(|ending| ending.get("reason").cloned())
        .and_then(|reason| serde_json::from_value::<EndReason>(reason).ok())
        .map_or_else(|| ending.to_string(), |reason| reason.to_string())
}

async fn back_up(database: &Database, backups: Option<&Path>) -> Result<String, String> {
    let folder = backups.ok_or_else(|| {
        "pas de dossier de sauvegarde : la variable BACKUP_DIR n'est pas réglée".to_string()
    })?;
    let file = backup::back_up(database, folder, BACKUPS_KEPT)
        .await
        .map_err(|_| format!("sauvegarde ratée dans {}", folder.display()))?;
    Ok(format!(
        "Sauvegarde : {} ({BACKUPS_KEPT} gardées au plus dans ce dossier).",
        file.display()
    ))
}

/// Une erreur, en texte à afficher.
fn text(error: impl std::fmt::Display) -> String {
    error.to_string()
}

/// « 1 carte », « 3 cartes », « 2 cartes nouvelles » : au pluriel, chaque mot prend un s.
fn counted(count: impl TryInto<i64>, word: &str) -> String {
    let count: i64 = count.try_into().unwrap_or(i64::MAX);
    if count > 1 {
        let plural: Vec<String> = word.split(' ').map(|part| format!("{part}s")).collect();
        format!("{count} {}", plural.join(" "))
    } else {
        format!("{count} {word}")
    }
}

/// Un compte, par son numéro (`3` ou `#3`).
fn account_id(text: &str) -> Result<i64, String> {
    text.trim_start_matches('#')
        .parse()
        .map_err(|_| format!("« {text} » n'est pas un numéro de compte (voir : comptes)"))
}

/// « #3 Ana », si ce compte existe.
async fn who(database: &Database, id: i64) -> Result<String, String> {
    match database.pseudo_of(id).await.map_err(text)? {
        Some(pseudo) => Ok(format!("#{id} {pseudo}")),
        None => Err(format!("aucun compte n'a le numéro {id} (voir : comptes)")),
    }
}

/// Les cartes du jeu en ligne : seules celles-là se donnent.
fn online_catalog() -> Result<Vec<Attack>, String> {
    bots::variants()
        .into_iter()
        .find(|variant| variant.name == bots::ONLINE_VARIANT)
        .map(|variant| variant.config.catalog_attacks())
        .ok_or_else(|| "la variante du jeu en ligne est introuvable".to_string())
}

/// La carte du catalogue en ligne de cette famille et de cette rareté, écrites par leur nom
/// du protocole (`MassacreDivin`, `Legendary`) ou leur nom affiché (« Massacre divin »,
/// « Légendaire »), sans souci des majuscules, des espaces ni des accents.
fn card(family: &str, rarity: &str) -> Result<Attack, String> {
    let catalog = online_catalog()?;
    let mut kinds: Vec<AttackKind> = Vec::new();
    for attack in &catalog {
        if !kinds.contains(&attack.kind) {
            kinds.push(attack.kind);
        }
    }
    let kind = kinds
        .iter()
        .copied()
        .find(|kind| same_name(family, &[wire_name(kind), kind.to_string()]))
        .ok_or_else(|| {
            let names: Vec<String> = kinds.iter().map(wire_name).collect();
            format!(
                "aucune famille en ligne ne s'appelle « {family} » ; les familles : {}",
                names.join(", ")
            )
        })?;
    let rarities: Vec<Rarity> = catalog
        .iter()
        .filter(|attack| attack.kind == kind)
        .map(|attack| attack.rarity)
        .collect();
    let rarity = rarities
        .iter()
        .copied()
        .find(|found| same_name(rarity, &[wire_name(found), found.to_string()]))
        .ok_or_else(|| {
            let names: Vec<String> = rarities.iter().map(ToString::to_string).collect();
            format!(
                "{kind} n'a pas de rareté « {rarity} » en ligne ; ses raretés : {}",
                names.join(", ")
            )
        })?;
    Ok(Attack { kind, rarity })
}

/// Le nom du protocole (`MassacreDivin`, `Epic`).
fn wire_name(value: &impl Serialize) -> String {
    match serde_json::to_value(value) {
        Ok(serde_json::Value::String(name)) => name,
        _ => String::new(),
    }
}

fn same_name(typed: &str, names: &[String]) -> bool {
    let typed = folded(typed);
    names.iter().any(|name| folded(name) == typed)
}

/// Un nom sans majuscules, espaces, tirets ni accents, pour comparer ce qui est tapé.
fn folded(text: &str) -> String {
    text.chars()
        .filter(|letter| !letter.is_whitespace() && *letter != '-' && *letter != '_')
        .flat_map(char::to_lowercase)
        .map(|letter| match letter {
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'à' | 'â' | 'ä' => 'a',
            'î' | 'ï' => 'i',
            'ô' | 'ö' => 'o',
            'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            other => other,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn run(database: &Database, line: &str) -> Result<String, String> {
        let args: Vec<String> = line.split_whitespace().map(str::to_string).collect();
        administer(database, &args, None).await
    }

    async fn base_with_ana() -> (Database, i64) {
        let database = Database::in_memory().await.unwrap();
        let ana = database
            .create_account("Ana", "empreinte", &[])
            .await
            .unwrap();
        (database, ana.id)
    }

    #[tokio::test]
    async fn the_help_comes_first_and_an_unknown_command_is_refused() {
        let (database, _) = base_with_ana().await;
        assert_eq!(run(&database, "").await.unwrap(), help());
        assert_eq!(run(&database, "aide").await.unwrap(), help());
        assert!(help().contains(&format!("({GAMES_SHOWN} par défaut)")));
        assert!(run(&database, "effacer tout").await.is_err());
        assert!(run(&database, "parties zéro").await.is_err());
    }

    #[tokio::test]
    async fn accounts_are_listed_and_an_unknown_one_is_refused() {
        let (database, ana) = base_with_ana().await;
        let listed = run(&database, "comptes").await.unwrap();
        assert!(
            listed.contains(&format!("#{ana}")) && listed.contains("Ana"),
            "{listed}"
        );
        assert!(run(&database, "collection 999").await.is_err());
        assert!(run(&database, "collection Ana").await.is_err());
        assert!(run(&database, &format!("collection #{ana}")).await.is_ok());
    }

    #[tokio::test]
    async fn a_card_is_given_once_shown_then_taken_back() {
        let (database, ana) = base_with_ana().await;
        let given = run(
            &database,
            &format!("donner {ana} massacre divin legendaire"),
        )
        .await;
        // Sans guillemets, « massacre divin » fait deux mots : la commande n'est pas comprise.
        assert!(given.is_err());
        let given = run(&database, &format!("donner {ana} MassacreDivin légendaire"))
            .await
            .unwrap();
        assert!(given.contains("donnée"), "{given}");
        let again = run(&database, &format!("donner {ana} massacredivin Legendary"))
            .await
            .unwrap();
        assert!(again.contains("déjà"), "{again}");
        let shown = run(&database, &format!("collection {ana}")).await.unwrap();
        assert!(
            shown.contains(
                "1 carte
"
            ) && shown.contains("Légendaire"),
            "{shown}"
        );
        // Rangée dans la cache, elle en sort avec elle.
        let legendary = card("MassacreDivin", "Legendary").unwrap();
        database.set_cache_trio(ana, &[legendary]).await.unwrap();
        let taken = run(&database, &format!("retirer {ana} MassacreDivin Legendary"))
            .await
            .unwrap();
        assert!(taken.contains("retirée"), "{taken}");
        let left = database.collection(ana).await.unwrap();
        assert!(left.cards.is_empty() && left.cache_trio.is_empty());
    }

    #[tokio::test]
    async fn only_cards_of_the_online_catalog_are_given() {
        let (database, ana) = base_with_ana().await;
        assert!(
            run(&database, &format!("donner {ana} Licorne Rare"))
                .await
                .is_err()
        );
        // Les Meilleurs amis n'existent qu'en Épique.
        assert!(
            run(&database, &format!("donner {ana} MeilleursAmis Commune"))
                .await
                .is_err()
        );
        let all = run(&database, &format!("donner-tout {ana}")).await.unwrap();
        let catalog = online_catalog().unwrap();
        assert!(all.contains(&catalog.len().to_string()), "{all}");
        let mut owned = database.collection(ana).await.unwrap().cards;
        let mut expected = catalog.clone();
        owned.sort_unstable();
        expected.sort_unstable();
        assert_eq!(owned, expected);
    }

    #[tokio::test]
    async fn a_pseudo_follows_the_rules_of_the_game() {
        let (database, ana) = base_with_ana().await;
        let renamed = run(&database, &format!("renommer {ana} Ana la Grande"))
            .await
            .unwrap();
        assert!(renamed.contains("Ana la Grande"), "{renamed}");
        assert_eq!(
            database.pseudo_of(ana).await.unwrap().as_deref(),
            Some("Ana la Grande")
        );
        let too_long = "x".repeat(200);
        assert!(
            run(&database, &format!("renommer {ana} {too_long}"))
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn games_and_backups_say_when_there_is_nothing_to_do() {
        let (database, _) = base_with_ana().await;
        assert_eq!(
            run(&database, "parties").await.unwrap(),
            "Aucune partie finie."
        );
        assert!(run(&database, "sauvegarder").await.is_err());
    }

    #[tokio::test]
    async fn a_backup_into_a_missing_folder_is_an_error_not_a_panic() {
        // Une base sur disque, comme celle du serveur (celle en mémoire ne se copie pas).
        let folder = std::env::temp_dir().join(format!(
            "fight_of_pions_admin_{}",
            crate::random::random_key()
        ));
        std::fs::create_dir_all(&folder).unwrap();
        let database = Database::open_file(&folder.join("base.db")).await.unwrap();
        let absent = folder.join("dossier_absent");
        let args = ["sauvegarder".to_string()];
        assert!(administer(&database, &args, Some(&absent)).await.is_err());
        drop(database);
        let _ = std::fs::remove_dir_all(&folder);
    }

    #[test]
    fn a_finished_game_reads_as_one_line() {
        let game = GameSummary {
            id: 12,
            ended_at: "2026-11-11T16:20:00Z".to_string(),
            players: [
                ("human".to_string(), Some("Ana".to_string())),
                ("mcts".to_string(), None),
            ],
            winner: Some(1),
            ending: r#"{"outcome":{"Won":"One"},"reason":"Forfeit"}"#.to_string(),
            moves: 84,
        };
        assert_eq!(
            game_line(&game),
            format!(
                "#12 · 2026-11-11T16:20:00Z · Ana contre bot mcts · Ana gagne ({}) · 84 coups",
                EndReason::Forfeit
            )
        );
    }

    #[tokio::test]
    async fn a_recovery_code_made_by_the_admin_opens_the_account() {
        let (database, ana) = base_with_ana().await;
        let shown = run(&database, &format!("code {ana}")).await.unwrap();
        let code = shown
            .split_whitespace()
            .find(|word| word.matches('-').count() == 3)
            .unwrap();
        let hash = recovery_hash(code).unwrap();
        let found = database
            .recover(&hash, "jeton de l'appareil")
            .await
            .unwrap();
        assert_eq!(found.map(|account| account.id), Some(ana));
        assert!(run(&database, "code 999").await.is_err());
    }
}
