//! Opening the database file: directory creation, schema upgrade and quarantine of a corrupt file.

use std::path::{Path, PathBuf};

use rusqlite::Connection;

use super::error::Fault;
use super::schema::migrate;

/// Where a corrupt database is moved to (`<name>.corrupt`) so the next open starts empty.
pub fn quarantine_path(database: &Path) -> PathBuf {
    let mut name = database.as_os_str().to_owned();
    name.push(".corrupt");
    PathBuf::from(name)
}

pub(super) fn open_database(database: &Path) -> Result<Connection, Fault> {
    if let Some(parent) = database.parent() {
        std::fs::create_dir_all(parent)?;
    }
    match connect(database) {
        Err(fault) if fault.is_corrupt_file() => {
            std::fs::rename(database, quarantine_path(database))?;
            Err(fault)
        }
        other => other,
    }
}

fn connect(database: &Path) -> Result<Connection, Fault> {
    let mut conn = Connection::open(database)?;
    migrate(&mut conn)?;
    Ok(conn)
}
