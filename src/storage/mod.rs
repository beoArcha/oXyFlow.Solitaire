//! SQLite implementation of [`oxysol_tui::Storage`]: settings, the saved game and the game history
//! in one `oxysol.db` file. Every write runs in a transaction. This module logs nothing; the TUI
//! decides how to react to an error and logs it there.

mod convert;
mod error;
mod history;
mod open;
mod schema;
mod sqlite;

use std::path::Path;

use oxysol_tui::StorageError;
use rusqlite::Connection;

use crate::paths::AppPaths;

pub use open::quarantine_path;
pub use oxysol_tui::GameRecord;

/// The persistent storage behind one open database file.
#[derive(Debug)]
pub struct SqliteStorage {
    conn: Connection,
}

impl SqliteStorage {
    /// Opens `oxysol.db` in the data directory of `paths`, creating the directory and the schema.
    pub fn open(paths: &AppPaths) -> Result<Self, StorageError> {
        Self::open_file(&paths.database_file())
    }

    /// Opens the database at `file`. A file that is not a database is renamed to `<file>.corrupt`
    /// and reported as [`StorageError::Corrupt`]; opening again then starts an empty database.
    pub fn open_file(file: &Path) -> Result<Self, StorageError> {
        let conn = open::open_database(file)?;
        Ok(Self { conn })
    }

    /// The schema version this build reads and writes.
    pub fn supported_schema_version() -> u32 {
        schema::supported_version()
    }
}
