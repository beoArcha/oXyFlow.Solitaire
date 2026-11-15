//! Errors of the application layer and their exit codes.

use std::io;
use std::process::ExitCode;

use oxysol_tui::{StorageError, TuiError};
use thiserror::Error;

use crate::cli::CliError;
use crate::paths::PathsError;

/// Every failure that ends the application before or during play.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("invalid command-line arguments")]
    Arguments(#[from] CliError),
    #[error("no folder for the application files")]
    Paths(#[from] PathsError),
    #[error("the database cannot be opened")]
    Storage(#[from] StorageError),
    #[error("the terminal interface stopped")]
    Terminal(#[from] TuiError),
    #[error("standard output cannot be written")]
    Output(#[from] io::Error),
}

impl AppError {
    /// `2` for a usage error, `1` for every runtime failure.
    pub fn exit_code(&self) -> ExitCode {
        match self {
            Self::Arguments(_) => ExitCode::from(2),
            Self::Paths(_) | Self::Storage(_) | Self::Terminal(_) | Self::Output(_) => {
                ExitCode::FAILURE
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_errors_exit_with_two_and_runtime_failures_with_one() {
        let usage = AppError::from(CliError::UnknownArgument("--x".to_owned()));
        assert_eq!(usage.exit_code(), ExitCode::from(2));
        let runtime = AppError::from(PathsError);
        assert_eq!(runtime.exit_code(), ExitCode::FAILURE);
    }
}
