//! Forward migration of the database: a version 1 file with rows opens as version 2 and keeps
//! every row.

use oxysol::paths::AppPaths;
use oxysol::storage::SqliteStorage;
use oxysol_core::{DrawCount, Variant};
use oxysol_tui::{Outcome, Storage};
use rusqlite::Connection;
use tempfile::TempDir;

const KLONDIKE: Variant = Variant::Klondike(DrawCount::One);

const SCHEMA_V1: &str = "
    CREATE TABLE settings (
        id INTEGER PRIMARY KEY CHECK (id = 1),
        json TEXT NOT NULL
    );
    CREATE TABLE games (
        id INTEGER PRIMARY KEY,
        variant TEXT NOT NULL,
        seed INTEGER NOT NULL,
        started_at INTEGER NOT NULL,
        finished_at INTEGER,
        outcome TEXT,
        elapsed_ms INTEGER
    );
    CREATE TABLE saved_game (
        id INTEGER PRIMARY KEY CHECK (id = 1),
        game_id INTEGER NOT NULL,
        record TEXT NOT NULL,
        elapsed_ms INTEGER NOT NULL,
        app_version TEXT NOT NULL
    );
    INSERT INTO games (variant, seed, started_at, finished_at, outcome, elapsed_ms)
        VALUES ('{\"klondike\":\"one\"}', 7, 1700000000000, 1700000090000, 'won', 90000);
    INSERT INTO games (variant, seed, started_at, finished_at, outcome, elapsed_ms)
        VALUES ('\"free_cell\"', 8, 1700001000000, 1700001005000, 'abandoned', 5000);
    INSERT INTO games (variant, seed, started_at)
        VALUES ('\"free_cell\"', 9, 1700002000000);
";

fn version_one_database() -> (TempDir, AppPaths) {
    let dir = tempfile::tempdir();
    assert!(dir.is_ok(), "temporary directory");
    let Ok(dir) = dir else {
        std::process::abort();
    };
    let paths = AppPaths::in_dir(dir.path());
    assert!(std::fs::create_dir_all(dir.path()).is_ok());
    let conn = Connection::open(paths.database_file());
    assert!(conn.is_ok());
    let Ok(conn) = conn else {
        std::process::abort();
    };
    assert!(conn.execute_batch(SCHEMA_V1).is_ok());
    assert!(conn.pragma_update(None, "user_version", 1).is_ok());
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

#[test]
fn a_version_one_database_keeps_its_rows_and_gains_the_new_columns() {
    let (_dir, paths) = version_one_database();
    let storage = open(&paths);
    let rows = storage.history();
    assert!(rows.is_ok());
    let Ok(rows) = rows else {
        return;
    };
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].variant, KLONDIKE);
    assert_eq!(rows[0].seed, 7);
    assert_eq!(rows[0].outcome, Some(Outcome::Won));
    assert_eq!(rows[0].elapsed_ms, Some(90_000));
    assert_eq!(rows[1].outcome, Some(Outcome::Abandoned));
    assert_eq!(rows[2].outcome, None);
    assert!(rows.iter().all(|row| row.moves.is_none()));
    assert!(rows.iter().all(|row| row.undo_allowed));
}

#[test]
fn a_migrated_database_is_stamped_with_the_current_version_and_records_new_games() {
    let (_dir, paths) = version_one_database();
    {
        let mut storage = open(&paths);
        let id = storage.start_game(KLONDIKE, 10, false);
        assert!(id.is_ok());
        let Ok(id) = id else {
            return;
        };
        assert!(storage.finish_game(id, Outcome::Won, 61_000, 52).is_ok());
    }
    let storage = open(&paths);
    let rows = storage.history();
    assert!(rows.is_ok());
    let Ok(rows) = rows else {
        return;
    };
    assert_eq!(rows.len(), 4);
    assert_eq!(rows[3].moves, Some(52));
    assert!(!rows[3].undo_allowed);
    assert!(rows[0].undo_allowed);

    let conn = Connection::open(paths.database_file());
    assert!(conn.is_ok());
    let Ok(conn) = conn else {
        return;
    };
    let version: Result<u32, _> = conn.pragma_query_value(None, "user_version", |row| row.get(0));
    assert!(matches!(version, Ok(v) if v == SqliteStorage::supported_schema_version()));
    assert_eq!(SqliteStorage::supported_schema_version(), 2);
}
