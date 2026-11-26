//! Forward migration of the database: a version 1, 2, 3 or 4 file with rows opens as the current
//! version and keeps every row.

use oxysol::paths::AppPaths;
use oxysol::storage::SqliteStorage;
use oxysol_core::{DrawCount, Variant};
use oxysol_tui::{AssistCounters, Outcome, Proof, Storage};
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
        let id = storage.start_game(KLONDIKE, 10, oxysol_core::GameOptions::DEFAULT, false);
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
    assert_eq!(SqliteStorage::supported_schema_version(), 6);
    assert!(rows.iter().all(|row| row.counters.restarts == 0));
    assert!(rows.iter().all(|row| row.proof.is_none()));
}

const SCHEMA_V2_COLUMNS: &str = "
    ALTER TABLE games ADD COLUMN moves INTEGER;
    ALTER TABLE games ADD COLUMN undo_allowed INTEGER NOT NULL DEFAULT 1;
";

fn version_two_database() -> (TempDir, AppPaths) {
    let (dir, paths) = version_one_database();
    let conn = Connection::open(paths.database_file());
    let Ok(conn) = conn else {
        std::process::abort();
    };
    assert!(conn.execute_batch(SCHEMA_V2_COLUMNS).is_ok());
    assert!(conn.pragma_update(None, "user_version", 2).is_ok());
    (dir, paths)
}

#[test]
fn a_version_two_database_gains_the_proof_column_and_keeps_its_rows() {
    let (_dir, paths) = version_two_database();
    let storage = open(&paths);
    let rows = storage.history();
    assert!(matches!(rows, Ok(ref rows) if rows.len() == 3));
    let Ok(rows) = rows else {
        return;
    };
    assert!(rows.iter().all(|row| row.proof.is_none()));
    assert_eq!(rows[0].seed, 7);
    assert_eq!(rows[0].outcome, Some(Outcome::Won));
}

#[test]
fn a_recorded_proof_survives_a_reopen() {
    let (_dir, paths) = version_two_database();
    {
        let mut storage = open(&paths);
        let id = storage.start_game(KLONDIKE, 11, oxysol_core::GameOptions::DEFAULT, true);
        let Ok(id) = id else {
            std::process::abort();
        };
        assert!(storage.record_proof(id, Proof::Winnable).is_ok());
        assert!(
            storage
                .record_proof(oxysol_tui::GameId(999), Proof::Winnable)
                .is_err()
        );
    }
    let storage = open(&paths);
    let rows = storage.history();
    let Ok(rows) = rows else {
        std::process::abort();
    };
    assert_eq!(rows[3].proof, Some(Proof::Winnable));
    assert!(rows[..3].iter().all(|row| row.proof.is_none()));
}

const SCHEMA_V3_COLUMN: &str = "ALTER TABLE games ADD COLUMN proof TEXT;";

fn version_three_database() -> (TempDir, AppPaths) {
    let (dir, paths) = version_two_database();
    let conn = Connection::open(paths.database_file());
    let Ok(conn) = conn else {
        std::process::abort();
    };
    assert!(conn.execute_batch(SCHEMA_V3_COLUMN).is_ok());
    assert!(
        conn.execute(
            "INSERT INTO games (variant, seed, started_at, undo_allowed, proof)
             VALUES ('\"free_cell\"', 12, 1700003000000, 0, 'winnable')",
            [],
        )
        .is_ok()
    );
    assert!(conn.pragma_update(None, "user_version", 3).is_ok());
    (dir, paths)
}

#[test]
fn a_version_three_database_reads_its_old_rows_with_no_restarts() {
    let (_dir, paths) = version_three_database();
    let storage = open(&paths);
    let rows = storage.history();
    let Ok(rows) = rows else {
        std::process::abort();
    };
    assert_eq!(rows.len(), 4);
    assert!(rows.iter().all(|row| row.counters.restarts == 0));
    assert_eq!(rows[3].proof, Some(Proof::Winnable));
    assert!(!rows[3].undo_allowed);

    let conn = Connection::open(paths.database_file());
    let Ok(conn) = conn else {
        std::process::abort();
    };
    let version: Result<u32, _> = conn.pragma_query_value(None, "user_version", |row| row.get(0));
    assert!(matches!(version, Ok(v) if v == SqliteStorage::supported_schema_version()));
}

#[test]
fn a_restart_recorded_after_the_migration_survives_a_reopen() {
    let (_dir, paths) = version_three_database();
    {
        let mut storage = open(&paths);
        let id = oxysol_tui::GameId(4);
        assert!(storage.record_restart(id).is_ok());
        assert!(storage.record_restart(id).is_ok());
        assert!(storage.record_restart(oxysol_tui::GameId(999)).is_err());
    }
    let storage = open(&paths);
    let Ok(rows) = storage.history() else {
        std::process::abort();
    };
    assert_eq!(rows[3].counters.restarts, 2);
    assert!(rows[..3].iter().all(|row| row.counters.restarts == 0));
}

fn version_four_database() -> (TempDir, AppPaths) {
    let (dir, paths) = version_three_database();
    let conn = Connection::open(paths.database_file());
    let Ok(conn) = conn else {
        std::process::abort();
    };
    assert!(
        conn.execute_batch("ALTER TABLE games ADD COLUMN restarts INTEGER NOT NULL DEFAULT 0;")
            .is_ok()
    );
    assert!(
        conn.execute("UPDATE games SET restarts = 2 WHERE id = 4", [])
            .is_ok()
    );
    assert!(conn.pragma_update(None, "user_version", 4).is_ok());
    (dir, paths)
}

#[test]
fn a_version_four_database_reads_its_old_rows_with_zero_assist_counters() {
    let (_dir, paths) = version_four_database();
    let storage = open(&paths);
    let Ok(rows) = storage.history() else {
        std::process::abort();
    };
    assert!(
        rows.iter()
            .all(|row| row.counters.assist == AssistCounters::default())
    );
    assert!(
        rows.iter()
            .all(|row| row.counters.assist.share_percent().is_none())
    );
    assert_eq!(rows[3].counters.restarts, 2);
}

#[test]
fn assist_counters_written_after_the_migration_survive_a_reopen() {
    let (_dir, paths) = version_four_database();
    let assist = AssistCounters {
        manual: 9,
        quick: 41,
        finish: 12,
        failed: 3,
        longest_streak: 17,
    };
    let target = oxysol_tui::GameId(2);
    {
        let mut storage = open(&paths);
        assert!(storage.record_assist(target, assist).is_ok());
        assert!(
            storage
                .record_assist(oxysol_tui::GameId(999), assist)
                .is_err()
        );
    }
    let storage = open(&paths);
    let Ok(rows) = storage.history() else {
        std::process::abort();
    };
    let target_row = rows.iter().find(|row| row.id == target);
    assert!(
        target_row
            .is_some_and(|row| row.counters.assist == assist && row.counters.assist.is_assisted())
    );
    assert!(
        rows.iter()
            .filter(|row| row.id != target)
            .all(|row| row.counters.assist == AssistCounters::default())
    );
    assert_eq!(rows[3].counters.restarts, 2);
}

fn version_five_database() -> (TempDir, AppPaths) {
    let (dir, paths) = version_four_database();
    let conn = Connection::open(paths.database_file());
    let Ok(conn) = conn else {
        std::process::abort();
    };
    assert!(
        conn.execute_batch(
            "ALTER TABLE games ADD COLUMN manual_moves INTEGER NOT NULL DEFAULT 0;
             ALTER TABLE games ADD COLUMN quick_moves INTEGER NOT NULL DEFAULT 0;
             ALTER TABLE games ADD COLUMN finish_moves INTEGER NOT NULL DEFAULT 0;
             ALTER TABLE games ADD COLUMN failed_quick INTEGER NOT NULL DEFAULT 0;
             ALTER TABLE games ADD COLUMN quick_streak INTEGER NOT NULL DEFAULT 0;"
        )
        .is_ok()
    );
    assert!(conn.pragma_update(None, "user_version", 5).is_ok());
    (dir, paths)
}

#[test]
fn a_version_five_database_reads_its_old_rows_with_the_default_rules() {
    let (_dir, paths) = version_five_database();
    let storage = open(&paths);
    let Ok(rows) = storage.history() else {
        std::process::abort();
    };
    assert!(!rows.is_empty());
    assert!(
        rows.iter()
            .all(|row| row.options == oxysol_core::GameOptions::DEFAULT)
    );
}

#[test]
fn the_options_of_a_game_survive_a_reopen_and_only_its_own_are_kept() {
    let (_dir, paths) = version_five_database();
    let chosen = oxysol_core::GameOptions {
        klondike_passes: oxysol_core::StockPasses::Three,
        golf_wrap: true,
        ..oxysol_core::GameOptions::DEFAULT
    };
    {
        let mut storage = open(&paths);
        assert!(storage.start_game(KLONDIKE, 12, chosen, true).is_ok());
    }
    let storage = open(&paths);
    let Ok(rows) = storage.history() else {
        std::process::abort();
    };
    let Some(last) = rows.last() else {
        std::process::abort();
    };
    assert_eq!(last.seed, 12);
    assert_eq!(last.options, chosen.for_variant(KLONDIKE));
    assert!(!last.options.golf_wrap);
}
