//! Les sauvegardes de la base (E4i) : une copie complète et cohérente, faite pendant que le
//! serveur tourne (`VACUUM INTO`), rangée dans un dossier sous un nom daté
//! (`fight_of_pions-2026-11-11_020000.db`) ; seules les plus récentes sont gardées. Le
//! serveur en fait une chaque nuit si on lui donne un dossier (`BACKUP_DIR`) ; `serveur admin
//! sauvegarder` en fait une tout de suite.
//!
//! Le dossier gagne à être sur un autre disque que la base : si celui de la base meurt, les
//! copies survivent. Pour revenir à une copie : arrêter le serveur, la mettre à la place de
//! la base, relancer (docs/DEPLOIEMENT.md).

use crate::ServerError;
use crate::database::Database;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Le nom d'une sauvegarde : ce début, l'heure de la copie, puis cette fin. Seuls les
/// fichiers ainsi nommés sont comptés, et effacés quand il y en a trop.
const FILE_PREFIX: &str = "fight_of_pions-";
const FILE_SUFFIX: &str = ".db";
const SECONDS_PER_HOUR: u64 = 60 * 60;
const HOURS_PER_DAY: u64 = 24;
const SECONDS_PER_DAY: u64 = HOURS_PER_DAY * SECONDS_PER_HOUR;

/// Copie la base dans ce dossier, puis n'y garde que les `kept` sauvegardes les plus
/// récentes. Rend le fichier écrit.
pub async fn back_up(
    database: &Database,
    folder: &Path,
    kept: usize,
) -> Result<PathBuf, ServerError> {
    let stamp = database.file_timestamp().await?;
    let file = folder.join(format!("{FILE_PREFIX}{stamp}{FILE_SUFFIX}"));
    database.back_up_into(&file).await?;
    if let Err(error) = prune(folder, kept) {
        eprintln!("sauvegarde : anciennes copies pas effacées ({error})");
    }
    Ok(file)
}

/// Efface les sauvegardes de ce dossier au-delà des `kept` plus récentes (le nom daté se
/// trie comme l'heure). Rend celles qui ont été effacées.
fn prune(folder: &Path, kept: usize) -> std::io::Result<Vec<PathBuf>> {
    let mut copies: Vec<PathBuf> = std::fs::read_dir(folder)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(FILE_PREFIX) && name.ends_with(FILE_SUFFIX))
        })
        .collect();
    copies.sort();
    let extra = copies.len().saturating_sub(kept);
    let removed: Vec<PathBuf> = copies.into_iter().take(extra).collect();
    for path in &removed {
        std::fs::remove_file(path)?;
    }
    Ok(removed)
}

/// Le temps à attendre, à cet instant (compté depuis l'époque Unix), jusqu'à la prochaine
/// fois où il est cette heure en UTC.
pub fn delay_until_hour(now: Duration, hour_utc: u64) -> Duration {
    let into_day = now.as_secs() % SECONDS_PER_DAY;
    let target = (hour_utc % HOURS_PER_DAY) * SECONDS_PER_HOUR;
    let wait = if into_day < target {
        target - into_day
    } else {
        SECONDS_PER_DAY - into_day + target
    };
    Duration::from_secs(wait)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_next_backup_waits_for_the_hour_today_or_tomorrow() {
        let two = 2 * SECONDS_PER_HOUR;
        let at = |seconds: u64| Duration::from_secs(10 * SECONDS_PER_DAY + seconds);
        assert_eq!(delay_until_hour(at(0), 2), Duration::from_secs(two));
        assert_eq!(delay_until_hour(at(two - 1), 2), Duration::from_secs(1));
        // À l'heure pile ou après : le lendemain.
        assert_eq!(
            delay_until_hour(at(two), 2),
            Duration::from_secs(SECONDS_PER_DAY)
        );
        assert_eq!(
            delay_until_hour(at(23 * SECONDS_PER_HOUR), 2),
            Duration::from_secs(3 * SECONDS_PER_HOUR)
        );
    }

    #[tokio::test]
    async fn a_backup_is_a_readable_copy_and_only_the_latest_are_kept() {
        let folder = std::env::temp_dir().join(format!(
            "fight_of_pions_sauvegardes_{}",
            crate::random::random_key()
        ));
        std::fs::create_dir_all(&folder).unwrap();
        // De vieilles copies, et un fichier étranger qui ne doit jamais être touché.
        for old in [
            "2000-01-01_000000",
            "2000-01-02_000000",
            "2000-01-03_000000",
        ] {
            std::fs::write(folder.join(format!("{FILE_PREFIX}{old}{FILE_SUFFIX}")), b"").unwrap();
        }
        std::fs::write(folder.join("notes.txt"), b"a garder").unwrap();
        // La base d'un serveur : un fichier, comme en vrai.
        let base_folder = folder.with_extension("base");
        std::fs::create_dir_all(&base_folder).unwrap();
        let database = Database::open_file(&base_folder.join("base.db"))
            .await
            .unwrap();
        database
            .create_account("Ana", "empreinte", &[])
            .await
            .unwrap();

        let file = back_up(&database, &folder, 2).await.unwrap();

        let copy = Database::open_file(&file).await.unwrap();
        let accounts = copy.accounts().await.unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].pseudo, "Ana");
        drop(copy);
        // Les fichiers de travail de SQLite (`-wal`, `-shm`), laissés par la relecture, ne
        // comptent pas.
        let mut left: Vec<String> = std::fs::read_dir(&folder)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|name| !name.ends_with("-wal") && !name.ends_with("-shm"))
            .collect();
        left.sort();
        let newest = file.file_name().unwrap().to_string_lossy().into_owned();
        assert_eq!(
            left,
            [
                format!("{FILE_PREFIX}2000-01-03_000000{FILE_SUFFIX}"),
                newest,
                "notes.txt".to_string()
            ]
        );
        let _ = std::fs::remove_dir_all(&folder);
        let _ = std::fs::remove_dir_all(&base_folder);
    }
}
