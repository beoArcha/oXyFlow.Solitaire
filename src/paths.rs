//! Where the application keeps its files.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use thiserror::Error;

/// Environment variable that points every file at one directory (tests, portable use).
pub const HOME_VARIABLE: &str = "OXYSOL_HOME";

/// File whose presence next to the executable makes the copy portable.
pub const PORTABLE_MARKER: &str = "oxysol.portable";

/// Folder next to the executable that holds the files of a portable copy.
pub const PORTABLE_DATA_DIR: &str = "data";

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

    /// Resolves the locations: `OXYSOL_HOME` when set and not empty, else `data` beside the
    /// executable when the portable marker sits there, else the platform data directory of
    /// `ProjectDirs::from("", "oXyFlow", "oXySol")`.
    pub fn detect() -> Result<Self, PathsError> {
        Self::resolve(std::env::var_os(HOME_VARIABLE), executable_dir().as_deref())
    }

    /// The same as [`AppPaths::detect`] with the override and the executable's directory passed
    /// explicitly.
    pub fn resolve(home: Option<OsString>, exe_dir: Option<&Path>) -> Result<Self, PathsError> {
        match home.filter(|value| !value.is_empty()) {
            Some(home) => Ok(Self::in_dir(PathBuf::from(home))),
            None => match exe_dir.filter(|dir| is_portable_dir(dir)) {
                Some(dir) => Ok(Self::in_dir(dir.join(PORTABLE_DATA_DIR))),
                None => ProjectDirs::from("", "oXyFlow", "oXySol")
                    .map(|dirs| Self::in_dir(dirs.data_dir()))
                    .ok_or(PathsError),
            },
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

fn executable_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()?
        .parent()
        .map(Path::to_path_buf)
}

fn is_portable_dir(dir: &Path) -> bool {
    dir.join(PORTABLE_MARKER).is_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_home_puts_every_file_in_one_directory() {
        let paths = AppPaths::resolve(Some(OsString::from("portable")), None);
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
        let paths = AppPaths::resolve(Some(OsString::new()), None);
        assert!(!matches!(paths, Ok(ref p) if p.data_dir().as_os_str().is_empty()));
    }

    #[test]
    fn platform_directory_is_used_without_an_override() {
        let paths = AppPaths::resolve(None, None);
        if let Ok(paths) = paths {
            let dir = paths.data_dir().to_string_lossy().to_lowercase();
            assert!(dir.contains("oxysol"), "{dir}");
        }
    }

    #[test]
    fn marker_beside_the_executable_selects_the_data_folder_next_to_it() {
        let exe_dir = tempfile::tempdir().expect("temp dir");
        std::fs::write(exe_dir.path().join(PORTABLE_MARKER), b"").expect("marker");
        let paths = AppPaths::resolve(None, Some(exe_dir.path()));
        assert!(matches!(
            paths,
            Ok(ref p) if p.data_dir() == exe_dir.path().join(PORTABLE_DATA_DIR)
        ));
    }

    #[test]
    fn without_marker_the_platform_directory_is_used() {
        let exe_dir = tempfile::tempdir().expect("temp dir");
        let paths = AppPaths::resolve(None, Some(exe_dir.path()));
        if let Ok(paths) = paths {
            assert_ne!(paths.data_dir(), exe_dir.path().join(PORTABLE_DATA_DIR));
        }
    }

    #[test]
    fn explicit_home_wins_over_the_marker() {
        let exe_dir = tempfile::tempdir().expect("temp dir");
        std::fs::write(exe_dir.path().join(PORTABLE_MARKER), b"").expect("marker");
        let paths = AppPaths::resolve(Some(OsString::from("override")), Some(exe_dir.path()));
        assert!(matches!(paths, Ok(ref p) if p.data_dir() == Path::new("override")));
    }

    #[test]
    fn a_directory_named_like_the_marker_is_not_a_marker() {
        let exe_dir = tempfile::tempdir().expect("temp dir");
        std::fs::create_dir(exe_dir.path().join(PORTABLE_MARKER)).expect("directory");
        let paths = AppPaths::resolve(None, Some(exe_dir.path()));
        if let Ok(paths) = paths {
            assert_ne!(paths.data_dir(), exe_dir.path().join(PORTABLE_DATA_DIR));
        }
    }
}
