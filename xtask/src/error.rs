//! Errors of the `xtask` runner.

use std::path::PathBuf;

use crate::check_result::CheckResultError;
use crate::fonts::FontsError;
use crate::icon::IconError;
use crate::locale_diff::LocaleDiffError;
use crate::midi::MidiError;
use crate::screenshots::ScreenshotError;
use crate::sync_metadata::SyncMetadataError;
use crate::version_scheme::GitError;
use crate::worktrees::WorktreesError;

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
    #[error("cannot wait for `{program}`")]
    Wait {
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
    #[error("cannot work out the version")]
    Version(#[from] GitError),
    #[error(transparent)]
    SyncMetadata(#[from] SyncMetadataError),
    #[error(transparent)]
    Midi(#[from] MidiError),
    #[error(transparent)]
    Icon(#[from] IconError),
    #[error(transparent)]
    LocaleDiff(#[from] LocaleDiffError),
    #[error(transparent)]
    Fonts(#[from] FontsError),
    #[error(transparent)]
    Screenshots(#[from] ScreenshotError),
    #[error(transparent)]
    CheckResult(#[from] CheckResultError),
    #[error(transparent)]
    Worktrees(#[from] WorktreesError),
}
