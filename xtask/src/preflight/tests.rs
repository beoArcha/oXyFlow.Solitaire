//! Tests of the preflight. The repository tests build a temporary repository per test.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::card::{self, Card};
use super::git;
use super::glob::matches;
use super::{PreflightError, parse_options};

const TASK_LIST: &str = "# Phase 4\n\
\n\
### O-64 — `cargo xtask preflight` and a local CI mode\n\
\n\
- **Owner:** Haiku\n\
- **Files:** `xtask/src/**`, `docs/tech/workflow.md`, `AGENTS.md`\n\
- **Shared files:** `docs/tech/decisions.md`\n\
- **Scope:** C2, C3 (P2).\n\
\n\
### O-65 — Release provenance\n\
\n\
- **Files:** `.github/workflows/release.yml`\n";

struct Repo {
    dir: PathBuf,
}

impl Repo {
    fn new(name: &str) -> Self {
        let dir =
            std::env::temp_dir().join(format!("oxysol-preflight-{}-{name}", std::process::id()));
        if dir.exists() {
            fs::remove_dir_all(&dir).expect("clean the old temporary folder");
        }
        fs::create_dir_all(&dir).expect("create the temporary folder");
        let repo = Self { dir };
        git(&repo.dir, &["init", "-q", "-b", "main"]);
        git(&repo.dir, &["config", "user.name", "Test"]);
        git(&repo.dir, &["config", "user.email", "test@example.invalid"]);
        repo
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        if let Err(source) = fs::remove_dir_all(&self.dir) {
            eprintln!("cannot remove {}: {source}", self.dir.display());
        }
    }
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

fn words(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_owned()).collect()
}

#[test]
fn counts_commits_behind_and_ahead_of_the_base() {
    let repo = Repo::new("divergence");
    commit(&repo.dir, "README.md");
    git(&repo.dir, &["checkout", "-q", "-b", "feature"]);
    commit(&repo.dir, "first.txt");
    commit(&repo.dir, "second.txt");
    git(&repo.dir, &["checkout", "-q", "main"]);
    commit(&repo.dir, "upstream.txt");
    git(&repo.dir, &["checkout", "-q", "feature"]);
    assert_eq!(git::divergence(&repo.dir, "main").ok(), Some((1, 2)));
}

#[test]
fn a_branch_without_new_commits_is_neither_behind_nor_ahead() {
    let repo = Repo::new("level");
    commit(&repo.dir, "README.md");
    assert_eq!(git::divergence(&repo.dir, "main").ok(), Some((0, 0)));
}

#[test]
fn a_missing_base_is_an_error() {
    let repo = Repo::new("missing-base");
    commit(&repo.dir, "README.md");
    assert!(matches!(
        git::divergence(&repo.dir, "origin/absent"),
        Err(PreflightError::Git { .. })
    ));
}

#[test]
fn changed_files_include_commits_edits_and_untracked_files() {
    let repo = Repo::new("changed");
    commit(&repo.dir, "README.md");
    git(&repo.dir, &["checkout", "-q", "-b", "feature"]);
    commit(&repo.dir, "committed.txt");
    fs::write(repo.dir.join("README.md"), "edited\n").expect("edit a tracked file");
    fs::write(repo.dir.join("untracked.txt"), "new\n").expect("write an untracked file");
    assert_eq!(
        git::changed_files(&repo.dir, "main").ok(),
        Some(words(&["README.md", "committed.txt", "untracked.txt"]))
    );
}

#[test]
fn an_unchanged_branch_has_no_changed_files() {
    let repo = Repo::new("unchanged");
    commit(&repo.dir, "README.md");
    assert_eq!(git::changed_files(&repo.dir, "main").ok(), Some(Vec::new()));
}

#[test]
fn parse_counts_reads_behind_then_ahead() {
    assert_eq!(git::parse_counts("3\t5\n"), Some((3, 5)));
    assert_eq!(git::parse_counts("3\n"), None);
    assert_eq!(git::parse_counts("x y"), None);
}

#[test]
fn card_files_and_shared_files_are_read_from_the_card_bullets() {
    let card = card::parse(TASK_LIST, "O-64").expect("the card exists");
    assert_eq!(
        card.patterns,
        words(&[
            "xtask/src/**",
            "docs/tech/workflow.md",
            "AGENTS.md",
            "docs/tech/decisions.md",
        ])
    );
}

#[test]
fn a_card_names_its_files_and_not_the_others() {
    let card = card::parse(TASK_LIST, "O-64").expect("the card exists");
    assert!(card.names("xtask/src/preflight/mod.rs"));
    assert!(card.names("docs/tech/workflow.md"));
    assert!(!card.names(".github/workflows/release.yml"));
    assert!(!card.names("xtask/Cargo.toml"));
}

#[test]
fn a_card_id_must_match_the_whole_heading_id() {
    assert!(card::parse(TASK_LIST, "O-6").is_none());
    assert!(card::parse(TASK_LIST, "O-65").is_some());
}

#[test]
fn an_unknown_card_is_an_error() {
    let repo = Repo::new("unknown-card");
    fs::create_dir_all(repo.dir.join("docs").join("task_list")).expect("create the task list");
    fs::write(repo.dir.join("docs/task_list/phase-4.md"), TASK_LIST).expect("write the task list");
    assert!(matches!(
        card::find(&repo.dir, "O-99"),
        Err(PreflightError::UnknownCard { id }) if id == "O-99"
    ));
}

#[test]
fn a_known_card_is_found_in_the_task_list_folder() {
    let repo = Repo::new("known-card");
    fs::create_dir_all(repo.dir.join("docs").join("task_list")).expect("create the task list");
    fs::write(repo.dir.join("docs/task_list/phase-4.md"), TASK_LIST).expect("write the task list");
    let card = card::find(&repo.dir, "O-65").expect("the card exists");
    assert_eq!(card.patterns, words(&[".github/workflows/release.yml"]));
}

#[test]
fn a_card_without_a_files_line_names_nothing() {
    let text = "### O-70 — Plain\n\n- **Scope:** nothing\n";
    let card: Card = card::parse(text, "O-70").expect("the card exists");
    assert!(card.patterns.is_empty());
    assert!(!card.names("README.md"));
}

#[test]
fn double_star_matches_any_depth_and_zero_segments() {
    assert!(matches("xtask/src/**", "xtask/src/main.rs"));
    assert!(matches("xtask/src/**", "xtask/src/lint/mod.rs"));
    assert!(!matches("xtask/src/**", "xtask/Cargo.toml"));
    assert!(!matches("xtask/src/**", "xtask/srcx/main.rs"));
    assert!(matches("**/*.md", "AGENTS.md"));
    assert!(matches("**/*.md", "docs/tech/workflow.md"));
}

#[test]
fn star_and_question_mark_stay_inside_one_segment() {
    assert!(matches("docs/*.md", "docs/a.md"));
    assert!(!matches("docs/*.md", "docs/tech/a.md"));
    assert!(matches("xtask/src/check*", "xtask/src/check_result.rs"));
    assert!(matches("docs/?.md", "docs/a.md"));
    assert!(!matches("docs/?.md", "docs/ab.md"));
}

#[test]
fn an_exact_path_matches_only_itself() {
    assert!(matches("AGENTS.md", "AGENTS.md"));
    assert!(!matches("AGENTS.md", "docs/AGENTS.md"));
}

#[test]
fn options_read_card_base_and_ci() {
    let flags = words(&["--card", "O-64", "--base", "origin/dev", "--ci"]);
    let options = parse_options(&flags).expect("the flags fit");
    assert_eq!(options.card.as_deref(), Some("O-64"));
    assert_eq!(options.base, "origin/dev");
    assert!(options.ci);
}

#[test]
fn options_default_to_origin_main_without_card_or_ci() {
    let options = parse_options(&[]).expect("no flags fit");
    assert_eq!(options.card, None);
    assert_eq!(options.base, "origin/main");
    assert!(!options.ci);
}

#[test]
fn options_reject_a_flag_without_its_value_or_an_unknown_flag() {
    assert!(parse_options(&words(&["--card"])).is_none());
    assert!(parse_options(&words(&["--base"])).is_none());
    assert!(parse_options(&words(&["--quick"])).is_none());
}
