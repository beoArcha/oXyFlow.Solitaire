//! Errors of the `xtask` runner.

use std::path::PathBuf;

use crate::icon::IconError;
use crate::midi::MidiError;
use crate::sync_metadata::SyncMetadataError;

#[derive(Debug, thiserror::Error)]
pub enum XtaskError {
    #[error("{0}")]
    Usage(String),
    #[error("cannot locate the workspace root")]
    WorkspaceRoot,
    #[error("cannot read `{path}`")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("cannot start `{program}`")]
    Spawn {
        program: String,
        #[source]
        source: std::io::Error,
    },
    #[error("step `{step}` failed (exit code {code:?})")]
    StepFailed {
        step: &'static str,
        code: Option<i32>,
    },
    #[error("`git diff --name-only {base}...HEAD` failed (exit code {code:?}); is `{base}` a ref?")]
    GitDiff { base: String, code: Option<i32> },
    #[error("`{path}` is not valid TOML")]
    Exceptions {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("lint found {count} error(s)")]
    LintFailed { count: usize },
    #[error("`{path}` is not valid TOML")]
    Metadata {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("`{path}` has no string `version` in `[project]`")]
    MissingVersion { path: PathBuf },
    #[error("`git rev-list --count HEAD` failed (exit code {code:?})")]
    GitRevList { code: Option<i32> },
    #[error("`git rev-list --count HEAD` printed `{output}`, which is not a number")]
    CommitCount {
        output: String,
        #[source]
        source: std::num::ParseIntError,
    },
    #[error(transparent)]
    SyncMetadata(#[from] SyncMetadataError),
    #[error(transparent)]
    Midi(#[from] MidiError),
    #[error(transparent)]
    Icon(#[from] IconError),
}
