//! Classifies the worktrees of the repository and finds the worktree folders git no longer knows.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use super::WorktreesError;
use super::git::{self, Record};
use super::pulls::PullRequests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Main,
    Merged,
    Open,
}

impl State {
    pub fn label(self) -> &'static str {
        match self {
            Self::Main => "main",
            Self::Merged => "merged",
            Self::Open => "open",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub path: PathBuf,
    pub branch: Option<String>,
    pub state: State,
    pub dirty: bool,
    pub missing: bool,
}

impl Entry {
    pub fn is_removable(&self) -> bool {
        self.state == State::Merged && !self.dirty && !self.missing
    }
}

#[derive(Debug, Default)]
pub struct Listing {
    pub entries: Vec<Entry>,
    pub orphans: Vec<PathBuf>,
}

pub fn inspect(root: &Path, pulls: &dyn PullRequests) -> Result<Listing, WorktreesError> {
    if !git::has_origin_main(root)? {
        return Err(WorktreesError::NoOriginMain);
    }
    let records = git::worktrees(root)?;
    let Some((main, linked)) = records.split_first() else {
        return Ok(Listing::default());
    };
    let upstreams = git::upstreams(root)?;
    let mut entries = vec![Entry {
        path: main.path.clone(),
        branch: main.branch.clone(),
        state: State::Main,
        dirty: false,
        missing: main.missing,
    }];
    for record in linked {
        let state = classify(root, record, &upstreams, pulls)?;
        let dirty = state == State::Merged && !record.missing && git::has_changes(&record.path)?;
        entries.push(Entry {
            path: record.path.clone(),
            branch: record.branch.clone(),
            state,
            dirty,
            missing: record.missing,
        });
    }
    let orphans = orphan_folders(&main.path, &records)?;
    Ok(Listing { entries, orphans })
}

/// A branch is merged when its deleted upstream shows a merged pull request, when its pushed tip is an
/// ancestor of `origin/main`, or when its pull requests are finished. A branch that was never pushed is
/// never merged by ancestry: a fresh worktree is its own tip on `main` and must not be removed.
fn classify(
    root: &Path,
    record: &Record,
    upstreams: &HashMap<String, git::Upstream>,
    pulls: &dyn PullRequests,
) -> Result<State, WorktreesError> {
    let Some(branch) = record.branch.as_deref() else {
        return Ok(State::Open);
    };
    let upstream = upstreams.get(branch).copied().unwrap_or_default();
    let merged = upstream.gone
        || (upstream.configured && git::is_ancestor(root, branch)?)
        || pulls.finished(root, branch);
    Ok(if merged { State::Merged } else { State::Open })
}

/// Folders in `.claude/worktrees` and beside the main checkout whose `.git` file points into this
/// repository's `worktrees` directory, but which no registered worktree uses.
fn orphan_folders(main: &Path, records: &[Record]) -> Result<Vec<PathBuf>, WorktreesError> {
    let registered: HashSet<String> = records.iter().map(|record| key(&record.path)).collect();
    let admin = format!("{}/.git/worktrees/", key(main));
    let parents = [
        Some(main.join(".claude").join("worktrees")),
        main.parent().map(Path::to_path_buf),
    ];
    let mut orphans = Vec::new();
    for parent in parents.into_iter().flatten() {
        for folder in subfolders(&parent)? {
            if registered.contains(&key(&folder)) {
                continue;
            }
            if gitdir_of(&folder)?.is_some_and(|dir| dir.starts_with(&admin)) {
                orphans.push(folder);
            }
        }
    }
    Ok(orphans)
}

fn subfolders(parent: &Path) -> Result<Vec<PathBuf>, WorktreesError> {
    if !parent.is_dir() {
        return Ok(Vec::new());
    }
    let entries = fs::read_dir(parent).map_err(|source| WorktreesError::Io {
        path: parent.to_path_buf(),
        source,
    })?;
    let mut folders = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| WorktreesError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
        if entry.path().is_dir() {
            folders.push(entry.path());
        }
    }
    Ok(folders)
}

/// The normalized `gitdir:` target of a worktree marker file, or `None` when the folder has no such file.
fn gitdir_of(folder: &Path) -> Result<Option<String>, WorktreesError> {
    let marker = folder.join(".git");
    if !marker.is_file() {
        return Ok(None);
    }
    let text = fs::read_to_string(&marker).map_err(|source| WorktreesError::Io {
        path: marker.clone(),
        source,
    })?;
    let target = text
        .lines()
        .next()
        .and_then(|line| line.strip_prefix("gitdir:"))
        .map(|dir| normalize(dir.trim()));
    Ok(target)
}

fn key(path: &Path) -> String {
    normalize(&path.to_string_lossy())
}

fn normalize(text: &str) -> String {
    let unified = text.replace('\\', "/");
    let trimmed = unified.trim_end_matches('/');
    if cfg!(windows) {
        trimmed.to_lowercase()
    } else {
        trimmed.to_owned()
    }
}
