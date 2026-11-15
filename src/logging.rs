//! Diagnostics in `oxysol.log` (decision T-24): a plain-text `tracing` subscriber. The file is
//! emptied at start when the previous one grew above 1 MB.

use std::fs::{File, OpenOptions};
use std::io;
use std::path::Path;
use std::sync::Mutex;

use thiserror::Error;
use tracing::Level;
use tracing::subscriber::SetGlobalDefaultError;

use crate::paths::AppPaths;

/// Environment variable that sets the log level: `error`, `warn`, `info`, `debug` or `trace`.
pub const LEVEL_VARIABLE: &str = "OXYSOL_LOG";

/// A log file above this size is emptied when the application starts.
pub const MAX_LOG_BYTES: u64 = 1024 * 1024;

/// The log could not be started. The application continues without it.
#[derive(Debug, Error)]
pub enum LoggingError {
    #[error("the log file cannot be opened")]
    Open(#[source] io::Error),
    #[error("a tracing subscriber is already installed")]
    Subscriber(#[source] SetGlobalDefaultError),
}

/// Opens the log of `paths`, installs the subscriber at `requested` (default `info`) and returns
/// the file handle that the panic hook writes its record through.
pub fn start(paths: &AppPaths, requested: Option<&str>) -> Result<File, LoggingError> {
    let file = open_log(&paths.log_file()).map_err(LoggingError::Open)?;
    let writer = file.try_clone().map_err(LoggingError::Open)?;
    install(writer, requested)?;
    Ok(file)
}

/// Creates the directory of `path` if needed, then opens the file for appending, emptying it first
/// when it exceeds [`MAX_LOG_BYTES`].
pub fn open_log(path: &Path) -> io::Result<File> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let file = OpenOptions::new().create(true).append(true).open(path)?;
    if file.metadata()?.len() > MAX_LOG_BYTES {
        file.set_len(0)?;
    }
    Ok(file)
}

fn install(writer: File, requested: Option<&str>) -> Result<(), LoggingError> {
    let level = requested
        .and_then(|text| text.parse::<Level>().ok())
        .unwrap_or(Level::INFO);
    let subscriber = tracing_subscriber::fmt()
        .with_writer(Mutex::new(writer))
        .with_ansi(false)
        .with_max_level(level)
        .finish();
    tracing::subscriber::set_global_default(subscriber).map_err(LoggingError::Subscriber)?;
    let rejected = requested.filter(|text| !text.is_empty() && text.parse::<Level>().is_err());
    if let Some(text) = rejected {
        tracing::warn!(value = text, "OXYSOL_LOG is not a log level, using info");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_small_log_is_kept_for_appending() {
        let dir = tempfile::tempdir().ok();
        let Some(dir) = dir else {
            return;
        };
        let path = dir.path().join("nested").join("oxysol.log");
        std::fs::create_dir_all(dir.path().join("nested")).ok();
        std::fs::write(&path, b"previous run\n").ok();
        let file = open_log(&path);
        assert!(file.is_ok());
        let Ok(file) = file else {
            return;
        };
        assert_eq!(file.metadata().map(|m| m.len()).ok(), Some(13));
    }

    #[test]
    fn a_log_above_the_limit_is_emptied_at_start() {
        let dir = tempfile::tempdir().ok();
        let Some(dir) = dir else {
            return;
        };
        let path = dir.path().join("oxysol.log");
        let big = vec![b'x'; usize::try_from(MAX_LOG_BYTES + 1).unwrap_or(usize::MAX)];
        std::fs::write(&path, big).ok();
        let file = open_log(&path);
        let Ok(file) = file else {
            return;
        };
        assert_eq!(file.metadata().map(|m| m.len()).ok(), Some(0));
    }
}
