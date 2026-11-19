//! The version scheme (T-21, O-36). Standard library only, because the two build scripts include this
//! file with `#[path]` as well as `xtask` (`cargo xtask version`), so the binary and the release tag are
//! computed by the same code.
//!
//! `metadata.toml` holds `major.minor.patch` and the owner raises the minor when a plan phase closes.
//! The effective patch is derived from git: the number of first-parent commits on `HEAD` since the
//! commit that last changed the `version` line of `metadata.toml`, added to the patch of that line.
//! Squash merges give one commit per merge to `main`, so every merge raises the patch by one.

use std::error::Error;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Appended to the metadata version when git cannot give the patch (a source archive, a shallow clone).
pub const SOURCE_MARKER: &str = "src";

const METADATA_FILE: &str = "metadata.toml";
const VERSION_LINE_PATTERN: &str = "^version *=";
const WINDOWS_PART_MAX: u64 = 0xFFFF;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Semver {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

/// Reads `major.minor.patch`, ignoring a pre-release or build suffix.
pub fn parse_semver(text: &str) -> Option<Semver> {
    let core = text.split(['+', '-']).next()?;
    let mut parts = core.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some(Semver {
        major,
        minor,
        patch,
    })
}

/// The effective version: the metadata version with `merges` added to its patch.
pub fn assemble(base: &str, merges: u64) -> String {
    match parse_semver(base) {
        Some(version) => format!(
            "{}.{}.{}",
            version.major,
            version.minor,
            version.patch.saturating_add(merges)
        ),
        None => source_fallback(base),
    }
}

/// The version of a build without usable git history.
pub fn source_fallback(base: &str) -> String {
    format!("{base}+{SOURCE_MARKER}")
}

/// The release tag of an effective version.
pub fn release_tag(version: &str) -> String {
    format!("v{version}")
}

/// The numeric Windows file version (`major.minor.patch.0`, 16 bits each) of an effective version.
pub fn windows_file_version(version: &str) -> Option<u64> {
    let parsed = parse_semver(version)?;
    let part = |value: u64| value.min(WINDOWS_PART_MAX);
    Some((part(parsed.major) << 48) | (part(parsed.minor) << 32) | (part(parsed.patch) << 16))
}

#[derive(Debug)]
pub enum GitError {
    Spawn(io::Error),
    Failed { command: String, code: Option<i32> },
    Shallow,
    NoBase,
    NotANumber { command: String, output: String },
}

impl fmt::Display for GitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spawn(_) => write!(formatter, "cannot start `git`"),
            Self::Failed { command, code } => {
                write!(formatter, "`git {command}` failed (exit code {code:?})")
            }
            Self::Shallow => write!(
                formatter,
                "the repository is a shallow clone, so the number of merges cannot be counted; fetch the full history"
            ),
            Self::NoBase => write!(
                formatter,
                "no commit in the history changed the version line of `{METADATA_FILE}`"
            ),
            Self::NotANumber { command, output } => {
                write!(
                    formatter,
                    "`git {command}` printed `{output}`, which is not a number"
                )
            }
        }
    }
}

impl Error for GitError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Spawn(source) => Some(source),
            _ => None,
        }
    }
}

fn git(root: &Path, args: &[&str]) -> Result<String, GitError> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(GitError::Spawn)?;
    if !output.status.success() {
        return Err(GitError::Failed {
            command: args.join(" "),
            code: output.status.code(),
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

/// The number of merges to `main` since the version line of `metadata.toml` last changed.
pub fn merges_since_version_change(root: &Path) -> Result<u64, GitError> {
    if git(root, &["rev-parse", "--is-shallow-repository"])? == "true" {
        return Err(GitError::Shallow);
    }
    let base = git(
        root,
        &[
            "log",
            "-1",
            "--first-parent",
            "--format=%H",
            "-G",
            VERSION_LINE_PATTERN,
            "--",
            METADATA_FILE,
        ],
    )?;
    if base.is_empty() {
        return Err(GitError::NoBase);
    }
    let range = format!("{base}..HEAD");
    let args = ["rev-list", "--count", "--first-parent", range.as_str()];
    let output = git(root, &args)?;
    output.parse().map_err(|_| GitError::NotANumber {
        command: args.join(" "),
        output,
    })
}

/// The effective version for a metadata version, or an error when git cannot count the merges.
pub fn effective_version(root: &Path, base: &str) -> Result<String, GitError> {
    merges_since_version_change(root).map(|merges| assemble(base, merges))
}

/// The effective version for a build: it never fails, and falls back to the source marker.
pub fn build_version(root: &Path, base: &str) -> String {
    effective_version(root, base).unwrap_or_else(|_| source_fallback(base))
}

/// The files whose change can change the effective version: `HEAD`, the branch it names, `packed-refs`
/// and `metadata.toml`. Only existing files are returned, because a missing one would rerun every build.
pub fn watched_files(root: &Path) -> Vec<PathBuf> {
    let mut names = vec!["HEAD".to_owned(), "packed-refs".to_owned()];
    if let Ok(branch) = git(root, &["symbolic-ref", "-q", "HEAD"]) {
        names.push(branch);
    }
    let mut files = vec![root.join(METADATA_FILE)];
    for name in names {
        if let Ok(path) = git(root, &["rev-parse", "--git-path", name.as_str()]) {
            files.push(root.join(path));
        }
    }
    files.retain(|path| path.exists());
    files
}

#[cfg(test)]
#[path = "version_scheme/tests.rs"]
mod tests;
