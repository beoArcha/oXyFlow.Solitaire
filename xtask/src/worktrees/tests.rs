//! Tests of the worktree check. The repository tests build a temporary repository with a bare `origin`.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::inspect::{Entry, Listing, State, inspect};
use super::pulls::PullRequests;
use super::{WorktreesError, remove_merged};

struct NoPulls;

impl PullRequests for NoPulls {
    fn finished(&self, _root: &Path, _branch: &str) -> bool {
        false
    }
}

struct Finished(&'static str);

impl PullRequests for Finished {
    fn finished(&self, _root: &Path, branch: &str) -> bool {
        branch == self.0
    }
}

struct Fixture {
    base: PathBuf,
}

impl Fixture {
    fn new(name: &str, with_origin: bool) -> Self {
        let base =
            std::env::temp_dir().join(format!("oxysol-worktrees-{}-{name}", std::process::id()));
        if base.exists() {
            fs::remove_dir_all(&base).expect("clean the old temporary folder");
        }
        fs::create_dir_all(&base).expect("create the temporary folder");
        let fixture = Self { base };
        if with_origin {
            let origin = fixture.origin();
            git(
                &fixture.base,
                &["init", "-q", "--bare", "-b", "main", text(&origin)],
            );
        }
        git(&fixture.base, &["init", "-q", "-b", "main", "repo"]);
        git(&fixture.repo(), &["config", "user.name", "Test"]);
        git(
            &fixture.repo(),
            &["config", "user.email", "test@example.invalid"],
        );
        commit(&fixture.repo(), "README.md");
        if with_origin {
            git(
                &fixture.repo(),
                &["remote", "add", "origin", text(&fixture.origin())],
            );
            git(&fixture.repo(), &["push", "-q", "-u", "origin", "main"]);
        }
        fixture
    }

    fn repo(&self) -> PathBuf {
        self.base.join("repo")
    }

    fn origin(&self) -> PathBuf {
        self.base.join("origin.git")
    }

    /// A worktree beside the repository on a new branch from `main`, with one commit pushed upstream.
    fn feature(&self, branch: &str, folder: &str) -> PathBuf {
        let path = self.base.join(folder);
        git(
            &self.repo(),
            &["worktree", "add", "-q", "-b", branch, text(&path), "main"],
        );
        commit(&path, &format!("{folder}.txt"));
        git(&path, &["push", "-q", "-u", "origin", branch]);
        path
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.base) {
            eprintln!("warning: cannot remove `{}`: {error}", self.base.display());
        }
    }
}

fn text(path: &Path) -> &str {
    path.to_str().expect("the temporary path is UTF-8")
}

fn git(dir: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("git starts");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn commit(dir: &Path, file: &str) {
    fs::write(dir.join(file), format!("{file}\n")).expect("write a file");
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", file]);
}

/// `wt-ancestor` is merged with a merge commit, `wt-squash` with a squash merge and a deleted upstream,
/// `wt-open` is still open and `wt-fresh` was never pushed, so it is an ancestor of `main` yet open.
fn set_up(name: &str) -> Fixture {
    let fixture = Fixture::new(name, true);
    fixture.feature("feat/ancestor", "wt-ancestor");
    fixture.feature("feat/squash", "wt-squash");
    fixture.feature("feat/open", "wt-open");
    git(
        &fixture.repo(),
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "feat/fresh",
            text(&fixture.base.join("wt-fresh")),
            "main",
        ],
    );
    let repo = fixture.repo();
    git(
        &repo,
        &["merge", "-q", "--no-ff", "-m", "merge", "feat/ancestor"],
    );
    git(&repo, &["merge", "-q", "--squash", "feat/squash"]);
    git(&repo, &["commit", "-q", "-m", "squash feat/squash"]);
    git(&repo, &["push", "-q", "origin", "main"]);
    git(&repo, &["push", "-q", "origin", "--delete", "feat/squash"]);
    git(&repo, &["fetch", "-q", "--prune", "origin"]);
    fixture
}

fn find<'a>(listing: &'a Listing, folder: &str) -> &'a Entry {
    listing
        .entries
        .iter()
        .find(|entry| entry.path.file_name() == Some(OsStr::new(folder)))
        .expect("the worktree is listed")
}

fn is_listed(listing: &Listing, folder: &str) -> bool {
    listing
        .entries
        .iter()
        .any(|entry| entry.path.file_name() == Some(OsStr::new(folder)))
}

#[test]
fn states_follow_ancestry_deleted_upstream_and_open_branches() {
    let fixture = set_up("states");
    let listing = inspect(&fixture.repo(), &NoPulls).expect("inspect");
    assert_eq!(find(&listing, "repo").state, State::Main);
    assert_eq!(find(&listing, "wt-ancestor").state, State::Merged);
    assert_eq!(find(&listing, "wt-squash").state, State::Merged);
    assert_eq!(find(&listing, "wt-open").state, State::Open);
    assert_eq!(find(&listing, "wt-fresh").state, State::Open);
    assert!(listing.orphans.is_empty());
}

#[test]
fn closed_or_merged_pull_request_marks_a_branch_merged() {
    let fixture = set_up("pulls");
    let listing = inspect(&fixture.repo(), &Finished("feat/open")).expect("inspect");
    assert_eq!(find(&listing, "wt-open").state, State::Merged);
    assert!(find(&listing, "wt-open").is_removable());
}

#[test]
fn merged_worktree_with_changes_is_kept() {
    let fixture = set_up("dirty");
    fs::write(fixture.base.join("wt-ancestor").join("draft.txt"), "wip\n").expect("write");
    let listing = inspect(&fixture.repo(), &NoPulls).expect("inspect");
    let entry = find(&listing, "wt-ancestor");
    assert!(entry.dirty);
    assert!(!entry.is_removable());
    let failed = remove_merged(&fixture.repo(), &listing, false).expect("remove");
    assert_eq!(failed, 0);
    assert!(fixture.base.join("wt-ancestor").exists());
    assert!(!fixture.base.join("wt-squash").exists());
}

#[test]
fn remove_deletes_merged_worktrees_and_prunes_missing_folders() {
    let fixture = set_up("remove");
    fixture.feature("feat/vanished", "wt-vanished");
    fs::remove_dir_all(fixture.base.join("wt-vanished")).expect("delete the folder");
    let listing = inspect(&fixture.repo(), &NoPulls).expect("inspect");
    assert!(find(&listing, "wt-vanished").missing);

    let failed = remove_merged(&fixture.repo(), &listing, false).expect("remove");

    assert_eq!(failed, 0);
    assert!(!fixture.base.join("wt-ancestor").exists());
    assert!(!fixture.base.join("wt-squash").exists());
    assert!(fixture.base.join("wt-open").exists());
    let after = inspect(&fixture.repo(), &NoPulls).expect("inspect");
    assert!(!is_listed(&after, "wt-ancestor"));
    assert!(!is_listed(&after, "wt-vanished"));
    assert!(is_listed(&after, "wt-open"));
}

#[test]
fn unregistered_worktree_folders_are_orphans() {
    let fixture = Fixture::new("orphans", true);
    let repo = fixture.repo();
    let main = porcelain_main_path(&repo);
    let ghost = repo.join(".claude").join("worktrees").join("ghost");
    fs::create_dir_all(&ghost).expect("create the folder");
    let admin = format!("{main}/.git/worktrees/ghost");
    fs::write(ghost.join(".git"), format!("gitdir: {admin}\n")).expect("write the marker");
    let stray = fixture.base.join("stray");
    fs::create_dir_all(&stray).expect("create the folder");
    fs::write(stray.join(".git"), "gitdir: /nowhere/.git/worktrees/x\n").expect("write the marker");
    let plain = fixture.base.join("plain");
    fs::create_dir_all(plain.join(".git")).expect("create a repository folder");

    let listing = inspect(&repo, &NoPulls).expect("inspect");

    assert_eq!(listing.orphans.len(), 1);
    assert_eq!(listing.orphans[0].file_name(), Some(OsStr::new("ghost")));
}

fn porcelain_main_path(repo: &Path) -> String {
    let output = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(repo)
        .output()
        .expect("git starts");
    let text = String::from_utf8(output.stdout).expect("UTF-8 output");
    let first = text.lines().next().expect("the main worktree is listed");
    first
        .strip_prefix("worktree ")
        .expect("the first line names the worktree")
        .replace('\\', "/")
}

#[test]
fn missing_origin_main_is_an_error() {
    let fixture = Fixture::new("no-origin", false);
    let error = inspect(&fixture.repo(), &NoPulls).expect_err("origin/main is needed");
    assert!(matches!(error, WorktreesError::NoOriginMain));
}
