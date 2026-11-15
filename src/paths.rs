//! Where the application keeps its files.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use thiserror::Error;

/// Environment variable that points every file at one directory (tests, portable use).
pub const HOME_VARIABLE: &str = "OXYSOL_HOME";

/// File name of the SQLite database.
pub const DATABASE_FILE: &str = "oxysol.db";

/// File name of the plain-text log.
pub const LOG_FILE: &str = "oxysol.log";

/// The operating system offers no home directory to derive the data directory from.
#[derive(Debug, Error)]
#[error("no home directory is available to place the oXySol data directory in")]
pub struct PathsError;

/// Resolved locations of the application's files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    data_dir: PathBuf,
}

impl AppPaths {
    /// Everything under `data_dir`.
    pub fn in_dir(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            data_dir: data_dir.into(),
        }
    }

    /// Resolves the locations: `OXYSOL_HOME` when set and not empty, else the platform data
    /// directory of `ProjectDirs::from("", "oXyFlow", "oXySol")`.
    pub fn detect() -> Result<Self, PathsError> {
        Self::resolve(std::env::var_os(HOME_VARIABLE))
    }

    /// The same as [`AppPaths::detect`] with the override passed explicitly.
    pub fn resolve(home: Option<OsString>) -> Result<Self, PathsError> {
        match home.filter(|value| !value.is_empty()) {
            Some(home) => Ok(Self::in_dir(PathBuf::from(home))),
            None => ProjectDirs::from("", "oXyFlow", "oXySol")
                .map(|dirs| Self::in_dir(dirs.data_dir()))
                .ok_or(PathsError),
        }
    }

    /// The directory that holds the database and the log.
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    /// The SQLite database file.
    pub fn database_file(&self) -> PathBuf {
        self.data_dir.join(DATABASE_FILE)
    }

    /// The plain-text log file.
    pub fn log_file(&self) -> PathBuf {
        self.data_dir.join(LOG_FILE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_home_puts_every_file_in_one_directory() {
        let paths = AppPaths::resolve(Some(OsString::from("portable")));
        assert!(matches!(paths, Ok(ref p) if p.data_dir() == Path::new("portable")));
        let Ok(paths) = paths else {
            return;
        };
        assert_eq!(
            paths.database_file(),
            Path::new("portable").join(DATABASE_FILE)
        );
        assert_eq!(paths.log_file(), Path::new("portable").join(LOG_FILE));
    }

    #[test]
    fn empty_home_is_ignored() {
        let paths = AppPaths::resolve(Some(OsString::new()));
        assert!(!matches!(paths, Ok(ref p) if p.data_dir().as_os_str().is_empty()));
    }

    #[test]
    fn platform_directory_is_used_without_an_override() {
        let paths = AppPaths::resolve(None);
        if let Ok(paths) = paths {
            assert!(paths.data_dir().to_string_lossy().contains("oXySol"));
        }
    }
}
