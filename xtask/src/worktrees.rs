//! `worktrees [--remove [--force]]`: lists the git worktrees with the state of their branch (`main`,
//! `merged` or `open`) and the worktree folders that git no longer knows. A branch is merged when its
//! upstream is gone, when its pushed tip is an ancestor of `origin/main` (squash merges are not ancestors),
//! or when its pull requests are closed or merged. `--remove` deletes the merged worktrees without
//! uncommitted changes and prunes the records of missing folders. The default is a dry run.

mod git;
mod inspect;
mod pulls;
#[cfg(test)]
mod pure_tests;
#[cfg(test)]
mod tests;

use std::io;
use std::path::{Path, PathBuf};

use inspect::{Entry, Listing, inspect};
use pulls::GhCli;

#[derive(Debug, thiserror::Error)]
pub enum WorktreesError {
    #[error("cannot start `git`")]
    Spawn(#[source] io::Error),
    #[error("`git {command}` failed (exit code {code:?}): {stderr}")]
    Git {
        command: String,
        code: Option<i32>,
        stderr: String,
    },
    #[error("`git {command}` printed text that is not UTF-8")]
    NotUtf8 { command: String },
    #[error("`origin/main` does not exist; run `git fetch origin`")]
    NoOriginMain,
    #[error("cannot read `{path}`")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("{count} worktree(s) could not be removed")]
    RemoveFailed { count: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Options {
    pub remove: bool,
    pub force: bool,
}

/// Reads the flags after `worktrees`; `None` when they do not fit. `--force` needs `--remove`.
pub fn parse_options(flags: &[String]) -> Option<Options> {
    let mut options = Options {
        remove: false,
        force: false,
    };
    for flag in flags {
        match flag.as_str() {
            "--remove" => options.remove = true,
            "--force" => options.force = true,
            _ => return None,
        }
    }
    (!options.force || options.remove).then_some(options)
}

pub fn run(root: &Path, options: Options) -> Result<(), WorktreesError> {
    let listing = inspect(root, &GhCli)?;
    print_listing(&listing, !options.remove);
    if !options.remove {
        println!("dry run: nothing was removed; pass --remove to delete the merged worktrees");
        return Ok(());
    }
    match remove_merged(root, &listing, options.force)? {
        0 => Ok(()),
        count => Err(WorktreesError::RemoveFailed { count }),
    }
}

fn print_listing(listing: &Listing, dry_run: bool) {
    for entry in &listing.entries {
        println!("{}", describe(entry, dry_run));
    }
    if listing.orphans.is_empty() {
        println!("no orphan worktree folders");
        return;
    }
    println!("orphan worktree folders (git no longer knows them; not removed by this command):");
    for orphan in &listing.orphans {
        println!("  {}", orphan.display());
    }
}

fn describe(entry: &Entry, dry_run: bool) -> String {
    let branch = entry.branch.as_deref().unwrap_or("(detached)");
    let note = if entry.dirty {
        " [kept: uncommitted changes]"
    } else if entry.missing {
        " [folder missing]"
    } else if dry_run && entry.is_removable() {
        " [would remove]"
    } else {
        ""
    };
    format!(
        "{:<7} {:<40} {}{note}",
        entry.state.label(),
        branch,
        entry.path.display()
    )
}

/// Removes every merged worktree that is clean, then prunes; returns how many removals failed.
fn remove_merged(root: &Path, listing: &Listing, force: bool) -> Result<usize, WorktreesError> {
    let mut failed = 0;
    for entry in listing.entries.iter().filter(|entry| entry.is_removable()) {
        let path = entry.path.to_string_lossy().into_owned();
        let mut args = vec!["worktree", "remove"];
        if force {
            args.push("--force");
        }
        args.push(path.as_str());
        match git::stdout(root, &args) {
            Ok(_) => println!("removed {path}"),
            Err(error) => {
                eprintln!("error: {path}: {error}");
                failed += 1;
            }
        }
    }
    git::stdout(root, &["worktree", "prune"])?;
    Ok(failed)
}
