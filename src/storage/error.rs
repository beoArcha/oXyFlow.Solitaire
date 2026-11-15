//! The internal error of the SQLite backend. It keeps every `source`; the public `Storage` trait
//! gets a [`StorageError`] through `From`, because the TUI cannot name `rusqlite`.

use oxysol_tui::StorageError;
use rusqlite::ErrorCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub(super) enum Fault {
    #[error("file system operation failed")]
    Io(#[from] std::io::Error),
    #[error("SQLite operation failed")]
    Sqlite(#[from] rusqlite::Error),
    #[error("stored JSON is invalid")]
    Json(#[from] serde_json::Error),
    #[error("stored value is invalid: {reason}")]
    Invalid { reason: String },
    #[error("schema version {found} is newer than the supported {supported}")]
    Incompatible { found: u32, supported: u32 },
}

impl Fault {
    pub(super) fn invalid(reason: impl Into<String>) -> Self {
        Self::Invalid {
            reason: reason.into(),
        }
    }

    pub(super) fn is_corrupt_file(&self) -> bool {
        matches!(
            self,
            Self::Sqlite(rusqlite::Error::SqliteFailure(failure, _))
                if matches!(failure.code, ErrorCode::NotADatabase | ErrorCode::DatabaseCorrupt)
        )
    }
}

impl From<Fault> for StorageError {
    fn from(fault: Fault) -> Self {
        let message = describe(&fault);
        if fault.is_corrupt_file() {
            return Self::Corrupt { reason: message };
        }
        match fault {
            Fault::Io(source) => Self::Io(source),
            Fault::Incompatible { found, supported } => Self::Incompatible { found, supported },
            Fault::Json(_) | Fault::Invalid { .. } => Self::Corrupt { reason: message },
            Fault::Sqlite(_) => Self::Backend { message },
        }
    }
}

fn describe(fault: &Fault) -> String {
    let mut text = fault.to_string();
    let mut source = std::error::Error::source(fault);
    while let Some(cause) = source {
        text.push_str(": ");
        text.push_str(&cause.to_string());
        source = cause.source();
    }
    text
}
