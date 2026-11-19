use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};

use super::{
    GitError, Semver, assemble, build_version, effective_version, merges_since_version_change,
    parse_semver, release_tag, source_fallback, watched_files, windows_file_version,
};

#[test]
fn the_merge_count_is_added_to_the_patch() {
    assert_eq!(assemble("0.4.0", 0), "0.4.0");
    assert_eq!(assemble("0.4.0", 12), "0.4.12");
    assert_eq!(assemble("0.4.5", 3), "0.4.8");
}

#[test]
fn a_version_that_is_not_three_numbers_falls_back() {
    assert_eq!(assemble("0.4", 3), "0.4+src");
    assert_eq!(assemble("0.4.x", 3), "0.4.x+src");
    assert_eq!(assemble("0.4.0.1", 3), "0.4.0.1+src");
}

#[test]
fn the_fallback_keeps_the_metadata_version_and_marks_the_source() {
    assert_eq!(source_fallback("0.4.0"), "0.4.0+src");
}

#[test]
fn the_tag_is_the_version_with_a_v() {
    assert_eq!(release_tag("0.4.12"), "v0.4.12");
}

#[test]
fn a_suffix_is_ignored_when_reading_the_numbers() {
    let expected = Semver {
        major: 0,
        minor: 4,
        patch: 0,
    };
    assert_eq!(parse_semver("0.4.0+src"), Some(expected));
    assert_eq!(parse_semver("0.4.0-dev.2"), Some(expected));
    assert_eq!(parse_semver("0.4"), None);
}

#[test]
fn the_windows_file_version_packs_sixteen_bits_per_part() {
    assert_eq!(windows_file_version("0.4.12"), Some(0x0000_0004_000C_0000));
    assert_eq!(
        windows_file_version("1.2.3+src"),
        Some(0x0001_0002_0003_0000)
    );
    assert_eq!(
        windows_file_version("0.4.99999"),
        Some(0x0000_0004_FFFF_0000)
    );
    assert_eq!(windows_file_version("nope"), None);
}

static COUNTER: AtomicU32 = AtomicU32::new(0);

struct TempRepo {
    path: PathBuf,
}

impl TempRepo {
    fn new() -> Option<Self> {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!("oxysol-version-{}-{id}", std::process::id()));
        fs::create_dir_all(&path).ok()?;
        let repo = Self { path };
        repo.git(&["init", "-q", "-b", "main"])?;
        Some(repo)
    }

    fn git(&self, args: &[&str]) -> Option<()> {
        let status = Command::new("git")
            .args([
                "-c",
                "user.name=test",
                "-c",
                "user.email=test@example.invalid",
            ])
            .args(["-c", "commit.gpgsign=false"])
            .args(args)
            .current_dir(&self.path)
            .status()
            .ok()?;
        status.success().then_some(())
    }

    fn commit(&self, file: &str, text: &str) -> Option<()> {
        fs::write(self.path.join(file), text).ok()?;
        self.git(&["add", "-A"])?;
        self.git(&["commit", "-q", "-m", file])
    }

    fn set_version(&self, version: &str) -> Option<()> {
        let text = format!("[project]\nname = \"oXySol\"\nversion = \"{version}\"\n");
        self.commit("metadata.toml", &text)
    }
}

impl Drop for TempRepo {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.path).ok();
    }
}

fn root(repo: &TempRepo) -> &Path {
    &repo.path
}

#[test]
fn every_commit_after_the_version_line_is_one_merge() {
    let Some(repo) = TempRepo::new() else {
        return;
    };
    assert!(repo.set_version("0.4.0").is_some());
    assert_eq!(merges_since_version_change(root(&repo)).ok(), Some(0));
    assert!(repo.commit("a.txt", "a").is_some());
    assert!(repo.commit("b.txt", "b").is_some());
    assert_eq!(merges_since_version_change(root(&repo)).ok(), Some(2));
    assert_eq!(
        effective_version(root(&repo), "0.4.0").ok().as_deref(),
        Some("0.4.2")
    );
}

#[test]
fn changing_the_version_line_restarts_the_count() {
    let Some(repo) = TempRepo::new() else {
        return;
    };
    assert!(repo.set_version("0.4.0").is_some());
    assert!(repo.commit("a.txt", "a").is_some());
    assert!(repo.commit("b.txt", "b").is_some());
    assert!(repo.set_version("0.5.0").is_some());
    assert!(repo.commit("c.txt", "c").is_some());
    assert_eq!(
        effective_version(root(&repo), "0.5.0").ok().as_deref(),
        Some("0.5.1")
    );
}

#[test]
fn other_changes_to_the_metadata_do_not_restart_the_count() {
    let Some(repo) = TempRepo::new() else {
        return;
    };
    assert!(repo.set_version("0.4.0").is_some());
    assert!(repo.commit("a.txt", "a").is_some());
    let text = "[project]\nname = \"oXySol\"\nversion = \"0.4.0\"\ndescription = \"x\"\n";
    assert!(repo.commit("metadata.toml", text).is_some());
    assert_eq!(merges_since_version_change(root(&repo)).ok(), Some(2));
}

#[test]
fn a_directory_without_history_falls_back_to_the_source_marker() {
    let Some(repo) = TempRepo::new() else {
        return;
    };
    assert!(matches!(
        merges_since_version_change(root(&repo)),
        Err(GitError::Failed { .. } | GitError::NoBase)
    ));
    assert_eq!(build_version(root(&repo), "0.4.0"), "0.4.0+src");
}

#[test]
fn a_repository_that_never_set_the_version_has_no_base() {
    let Some(repo) = TempRepo::new() else {
        return;
    };
    assert!(repo.commit("a.txt", "a").is_some());
    assert!(matches!(
        merges_since_version_change(root(&repo)),
        Err(GitError::NoBase)
    ));
}

#[test]
fn the_watched_files_include_head_and_the_metadata() {
    let Some(repo) = TempRepo::new() else {
        return;
    };
    assert!(repo.set_version("0.4.0").is_some());
    let files = watched_files(root(&repo));
    assert!(files.iter().any(|file| file.ends_with("HEAD")), "{files:?}");
    assert!(
        files.iter().any(|file| file.ends_with("metadata.toml")),
        "{files:?}"
    );
}

#[test]
fn a_shallow_clone_falls_back_to_the_source_marker() {
    let Some(origin) = TempRepo::new() else {
        return;
    };
    assert!(origin.set_version("0.4.0").is_some());
    assert!(origin.commit("a.txt", "a").is_some());
    assert!(origin.commit("b.txt", "b").is_some());
    let Some(clone) = TempRepo::new() else {
        return;
    };
    let url = format!("file://{}", root(&origin).display()).replace('\\', "/");
    let cloned = Command::new("git")
        .args(["clone", "-q", "--depth", "1", url.as_str()])
        .arg(root(&clone).join("shallow"))
        .status();
    if !cloned.is_ok_and(|status| status.success()) {
        return;
    }
    let shallow = root(&clone).join("shallow");
    assert!(matches!(
        merges_since_version_change(&shallow),
        Err(GitError::Shallow)
    ));
    assert_eq!(build_version(&shallow, "0.4.0"), "0.4.0+src");
}
