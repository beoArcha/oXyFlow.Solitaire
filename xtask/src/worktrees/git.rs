//! Git queries of the worktree check. Every call runs `git` in the given directory.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use super::WorktreesError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub path: PathBuf,
    pub branch: Option<String>,
    pub missing: bool,
    bare: bool,
}

/// The worktrees in `git worktree list` order; the main checkout comes first. Bare repositories are left out.
pub fn worktrees(root: &Path) -> Result<Vec<Record>, WorktreesError> {
    Ok(parse_porcelain(&stdout(
        root,
        &["worktree", "list", "--porcelain"],
    )?))
}

pub fn parse_porcelain(text: &str) -> Vec<Record> {
    let mut records = Vec::new();
    let mut current: Option<Record> = None;
    for line in text.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            records.extend(current.take().filter(|record| !record.bare));
            current = Some(Record {
                path: PathBuf::from(path),
                branch: None,
                missing: false,
                bare: false,
            });
        } else if let Some(record) = current.as_mut() {
            apply_attribute(record, line);
        }
    }
    records.extend(current.filter(|record| !record.bare));
    records
}

fn apply_attribute(record: &mut Record, line: &str) {
    if let Some(reference) = line.strip_prefix("branch ") {
        let name = reference.strip_prefix("refs/heads/").unwrap_or(reference);
        record.branch = Some(name.to_owned());
    } else if line == "bare" {
        record.bare = true;
    } else if line.starts_with("prunable") {
        record.missing = true;
    }
}

/// The upstream state of a local branch. `configured` means the branch was published, so a branch that
/// only tracks `origin/main` (created from it) does not count. `gone` means the upstream was deleted.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Upstream {
    pub configured: bool,
    pub gone: bool,
}

const BASE_REF: &str = "refs/remotes/origin/main";

/// The upstream state of every local branch, by branch name.
pub fn upstreams(root: &Path) -> Result<HashMap<String, Upstream>, WorktreesError> {
    let format = "--format=%(refname)%09%(upstream)%09%(upstream:track)";
    let text = stdout(root, &["for-each-ref", format, "refs/heads"])?;
    Ok(text.lines().filter_map(parse_upstream).collect())
}

pub fn parse_upstream(line: &str) -> Option<(String, Upstream)> {
    let mut fields = line.split('\t');
    let name = fields.next()?.strip_prefix("refs/heads/")?;
    let configured = fields
        .next()
        .is_some_and(|upstream| !upstream.is_empty() && upstream != BASE_REF);
    let gone = fields.next() == Some("[gone]");
    Some((name.to_owned(), Upstream { configured, gone }))
}

/// Whether the branch tip is reachable from `origin/main`. Exit code 1 means "no", any other failure is an error.
pub fn is_ancestor(root: &Path, branch: &str) -> Result<bool, WorktreesError> {
    let reference = format!("refs/heads/{branch}");
    let args = [
        "merge-base",
        "--is-ancestor",
        reference.as_str(),
        "origin/main",
    ];
    let output = run(root, &args)?;
    match output.status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => Err(failed(&args, &output)),
    }
}

pub fn has_origin_main(root: &Path) -> Result<bool, WorktreesError> {
    let output = run(root, &["rev-parse", "--verify", "--quiet", "origin/main"])?;
    Ok(output.status.success())
}

pub fn has_changes(dir: &Path) -> Result<bool, WorktreesError> {
    Ok(!stdout(dir, &["status", "--porcelain"])?.trim().is_empty())
}

pub fn stdout(dir: &Path, args: &[&str]) -> Result<String, WorktreesError> {
    let output = run(dir, args)?;
    if !output.status.success() {
        return Err(failed(args, &output));
    }
    String::from_utf8(output.stdout).map_err(|_| WorktreesError::NotUtf8 {
        command: args.join(" "),
    })
}

fn run(dir: &Path, args: &[&str]) -> Result<Output, WorktreesError> {
    Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .map_err(WorktreesError::Spawn)
}

fn failed(args: &[&str], output: &Output) -> WorktreesError {
    WorktreesError::Git {
        command: args.join(" "),
        code: output.status.code(),
        stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    }
}
