//! `SqliteStorage` against real database files in a temporary directory.

use std::error::Error;
use std::path::PathBuf;

use oxysol::paths::{AppPaths, DATABASE_FILE};
use oxysol::storage::{SqliteStorage, quarantine_path};
use oxysol_core::{DrawCount, Move, Session, Variant};
use oxysol_tui::{GameId, Outcome, SavedGame, Settings, Storage, StorageError};
use rusqlite::Connection;
use tempfile::TempDir;

const KLONDIKE: Variant = Variant::Klondike(DrawCount::One);

fn temp_paths() -> (TempDir, AppPaths) {
    let dir = tempfile::tempdir();
    assert!(dir.is_ok(), "temporary directory");
    let Ok(dir) = dir else {
        std::process::abort();
    };
    let paths = AppPaths::in_dir(dir.path());
    (dir, paths)
}

fn open(paths: &AppPaths) -> SqliteStorage {
    let storage = SqliteStorage::open(paths);
    assert!(storage.is_ok(), "open failed: {:?}", storage.as_ref().err());
    let Ok(storage) = storage else {
        std::process::abort();
    };
    storage
}

fn saved_game(id: GameId, seed: u64, elapsed_ms: u64) -> SavedGame {
    let mut session = Session::new(KLONDIKE, seed);
    assert!(session.apply(Move::Stock).is_ok());
    SavedGame {
        id,
        record: session.record(),
        elapsed_ms,
        undo_allowed: true,
    }
}

fn database_file(paths: &AppPaths) -> PathBuf {
    paths.database_file()
}

#[test]
fn missing_rows_are_none() {
    let (_dir, paths) = temp_paths();
    let storage = open(&paths);
    assert!(matches!(storage.load_settings(), Ok(None)));
    assert!(matches!(storage.load_game(), Ok(None)));
    assert!(matches!(storage.history(), Ok(ref rows) if rows.is_empty()));
}

#[test]
fn settings_round_trip_and_replace() {
    let (_dir, paths) = temp_paths();
    let mut storage = open(&paths);
    let mut settings = Settings::default();
    assert!(storage.save_settings(&settings).is_ok());
    assert!(matches!(storage.load_settings(), Ok(Some(ref s)) if *s == settings));

    settings.volume = 12;
    settings.music = true;
    assert!(storage.save_settings(&settings).is_ok());
    assert!(matches!(storage.load_settings(), Ok(Some(ref s)) if *s == settings));
}

#[test]
fn game_round_trip_replace_and_clear() {
    let (_dir, paths) = temp_paths();
    let mut storage = open(&paths);
    let id = storage.start_game(KLONDIKE, 7, true);
    assert!(id.is_ok());
    let Ok(id) = id else {
        return;
    };

    let first = saved_game(id, 7, 1_500);
    assert!(storage.save_game(&first).is_ok());
    assert!(matches!(storage.load_game(), Ok(Some(ref g)) if *g == first));

    let second = saved_game(id, 7, 9_000);
    assert!(storage.save_game(&second).is_ok());
    assert!(matches!(storage.load_game(), Ok(Some(ref g)) if *g == second));

    assert!(storage.clear_game().is_ok());
    assert!(matches!(storage.load_game(), Ok(None)));
    assert!(storage.clear_game().is_ok());
}

#[test]
fn history_keeps_exact_seeds_variants_and_outcomes() {
    let (_dir, paths) = temp_paths();
    let mut storage = open(&paths);
    let seeds = [0, 1, u64::from(u32::MAX), 1 << 63, u64::MAX];
    let mut ids = Vec::new();
    for seed in seeds {
        let id = storage.start_game(KLONDIKE, seed, true);
        assert!(id.is_ok());
        if let Ok(id) = id {
            ids.push(id);
        }
    }
    assert!(storage.start_game(Variant::FreeCell, 99, true).is_ok());
    assert!(
        storage
            .finish_game(ids[4], Outcome::Won, 61_000, 87)
            .is_ok()
    );
    assert!(
        storage
            .finish_game(ids[0], Outcome::Abandoned, 5, 0)
            .is_ok()
    );

    let rows = storage.history();
    assert!(rows.is_ok());
    let Ok(rows) = rows else {
        return;
    };
    let stored: Vec<u64> = rows.iter().map(|row| row.seed).collect();
    assert_eq!(stored, [0, 1, u64::from(u32::MAX), 1 << 63, u64::MAX, 99]);
    assert_eq!(rows[5].variant, Variant::FreeCell);
    assert_eq!(rows[4].outcome, Some(Outcome::Won));
    assert_eq!(rows[4].elapsed_ms, Some(61_000));
    assert_eq!(rows[4].moves, Some(87));
    assert_eq!(rows[1].moves, None);
    assert!(rows.iter().all(|row| row.undo_allowed));
    assert_eq!(rows[0].outcome, Some(Outcome::Abandoned));
    assert_eq!(rows[1].outcome, None);
    assert!(rows[1].finished_at_ms.is_none());
    assert!(
        rows[4]
            .finished_at_ms
            .is_some_and(|end| end >= rows[4].started_at_ms)
    );
    assert!(rows[0].started_at_ms > 0);
}

#[test]
fn reopening_keeps_everything() {
    let (_dir, paths) = temp_paths();
    let settings = Settings {
        volume: 33,
        ..Settings::default()
    };
    let saved;
    {
        let mut storage = open(&paths);
        assert!(storage.save_settings(&settings).is_ok());
        let id = storage.start_game(KLONDIKE, u64::MAX, true);
        assert!(id.is_ok());
        let Ok(id) = id else {
            return;
        };
        saved = saved_game(id, u64::MAX, 4_200);
        assert!(storage.save_game(&saved).is_ok());
        assert!(
            storage
                .finish_game(id, Outcome::Abandoned, 4_200, 3)
                .is_ok()
        );
    }

    let storage = open(&paths);
    assert!(matches!(storage.load_settings(), Ok(Some(ref s)) if *s == settings));
    assert!(matches!(storage.load_game(), Ok(Some(ref g)) if *g == saved));
    let rows = storage.history();
    assert!(matches!(
        rows,
        Ok(ref r) if r.len() == 1 && r[0].seed == u64::MAX && r[0].outcome == Some(Outcome::Abandoned)
    ));
}

#[test]
fn finishing_an_unknown_game_is_an_error() {
    let (_dir, paths) = temp_paths();
    let mut storage = open(&paths);
    let result = storage.finish_game(GameId(404), Outcome::Won, 1, 1);
    assert!(matches!(result, Err(StorageError::Corrupt { .. })));
}

#[test]
fn a_newer_schema_version_is_incompatible_and_left_untouched() {
    let (_dir, paths) = temp_paths();
    drop(open(&paths));
    let newer = SqliteStorage::supported_schema_version() + 1;
    {
        let conn = Connection::open(database_file(&paths));
        assert!(conn.is_ok());
        let Ok(conn) = conn else {
            return;
        };
        assert!(conn.pragma_update(None, "user_version", newer).is_ok());
    }

    let result = SqliteStorage::open(&paths);
    assert!(matches!(
        result,
        Err(StorageError::Incompatible { found, supported })
            if found == newer && supported == SqliteStorage::supported_schema_version()
    ));
    assert!(database_file(&paths).exists());
    assert!(!quarantine_path(&database_file(&paths)).exists());
}

#[test]
fn a_fresh_database_is_stamped_with_the_supported_version() {
    let (_dir, paths) = temp_paths();
    drop(open(&paths));
    drop(open(&paths));
    let conn = Connection::open(database_file(&paths));
    assert!(conn.is_ok());
    let Ok(conn) = conn else {
        return;
    };
    let version: Result<u32, _> = conn.pragma_query_value(None, "user_version", |row| row.get(0));
    assert!(matches!(version, Ok(v) if v == SqliteStorage::supported_schema_version()));
}

#[test]
fn a_corrupt_file_is_renamed_and_the_next_open_starts_empty() {
    let (_dir, paths) = temp_paths();
    let file = database_file(&paths);
    let garbage = b"this is certainly not a sqlite database file".repeat(40);
    assert!(std::fs::write(&file, &garbage).is_ok());

    let first = SqliteStorage::open(&paths);
    assert!(matches!(first, Err(StorageError::Corrupt { .. })));
    assert!(!file.exists());
    let quarantined = quarantine_path(&file);
    assert!(quarantined.ends_with(format!("{DATABASE_FILE}.corrupt")));
    assert!(matches!(std::fs::read(&quarantined), Ok(ref bytes) if *bytes == garbage));

    let storage = open(&paths);
    assert!(matches!(storage.load_settings(), Ok(None)));
    assert!(matches!(storage.load_game(), Ok(None)));
    assert!(matches!(storage.history(), Ok(ref rows) if rows.is_empty()));
}

#[test]
fn an_unwritable_location_is_an_io_error_with_a_source() {
    let (dir, _paths) = temp_paths();
    let blocker = dir.path().join("occupied");
    assert!(std::fs::write(&blocker, b"a file, not a directory").is_ok());
    let paths = AppPaths::in_dir(blocker.join("data"));

    let result = SqliteStorage::open(&paths);
    assert!(matches!(result, Err(StorageError::Io(_))));
    let Err(error) = result else {
        return;
    };
    assert!(error.source().is_some());
}

#[test]
fn a_damaged_saved_game_is_corrupt_without_losing_the_rest() {
    let (_dir, paths) = temp_paths();
    let mut storage = open(&paths);
    let settings = Settings::default();
    assert!(storage.save_settings(&settings).is_ok());
    let id = storage.start_game(KLONDIKE, 5, true);
    assert!(id.is_ok());
    drop(storage);
    {
        let conn = Connection::open(database_file(&paths));
        assert!(conn.is_ok());
        let Ok(conn) = conn else {
            return;
        };
        let inserted = conn.execute(
            "INSERT INTO saved_game (id, game_id, record, elapsed_ms, app_version)
             VALUES (1, 1, 'not json', 0, '0')",
            [],
        );
        assert!(inserted.is_ok());
    }

    let storage = open(&paths);
    assert!(matches!(
        storage.load_game(),
        Err(StorageError::Corrupt { .. })
    ));
    assert!(matches!(storage.load_settings(), Ok(Some(ref s)) if *s == settings));
}
