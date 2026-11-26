//! The schema as an ordered list of migrations. `PRAGMA user_version` holds how many were applied.

use rusqlite::Connection;

use super::error::Fault;

/// Forward-only migrations; the index plus one is the schema version they produce.
pub(super) const MIGRATIONS: &[&str] = &[
    "
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
",
    "
    ALTER TABLE games ADD COLUMN moves INTEGER;
    ALTER TABLE games ADD COLUMN undo_allowed INTEGER NOT NULL DEFAULT 1;
",
    "
    ALTER TABLE games ADD COLUMN proof TEXT;
",
    "
    ALTER TABLE games ADD COLUMN restarts INTEGER NOT NULL DEFAULT 0;
",
    "
    ALTER TABLE games ADD COLUMN manual_moves INTEGER NOT NULL DEFAULT 0;
    ALTER TABLE games ADD COLUMN quick_moves INTEGER NOT NULL DEFAULT 0;
    ALTER TABLE games ADD COLUMN finish_moves INTEGER NOT NULL DEFAULT 0;
    ALTER TABLE games ADD COLUMN failed_quick INTEGER NOT NULL DEFAULT 0;
    ALTER TABLE games ADD COLUMN quick_streak INTEGER NOT NULL DEFAULT 0;
",
    "
    ALTER TABLE games ADD COLUMN options TEXT NOT NULL DEFAULT '{}';
",
];

/// The schema version this build writes.
pub(super) fn supported_version() -> u32 {
    u32::try_from(MIGRATIONS.len()).unwrap_or(u32::MAX)
}

/// Brings the database to the current schema in one transaction, or reports a newer one.
pub(super) fn migrate(conn: &mut Connection) -> Result<(), Fault> {
    let found: u32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    let supported = supported_version();
    if found > supported {
        return Err(Fault::Incompatible { found, supported });
    }
    if found == supported {
        return Ok(());
    }
    let tx = conn.transaction()?;
    for migration in MIGRATIONS.iter().skip(found as usize) {
        tx.execute_batch(migration)?;
    }
    tx.pragma_update(None, "user_version", supported)?;
    tx.commit()?;
    Ok(())
}
