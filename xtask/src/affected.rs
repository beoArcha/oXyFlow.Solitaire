//! The `affected` command: maps changed paths to the workspace packages whose tests they can
//! affect (T-22). `select` is pure; only `changed_paths` and `run` touch git and stdout.

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

use crate::error::XtaskError;

pub const CORE: &str = "oxysol-core";
pub const TUI: &str = "oxysol-tui";
pub const ROOT: &str = "oxysol";
pub const XTASK: &str = "xtask";

const ALL_FILE_NAMES: [&str; 4] = [
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "clippy.toml",
];
const ALL_PREFIXES: [&str; 4] = [".cargo/", ".github/", "locales/", "assets/"];
const DEPENDENTS: [(&str, &[&str]); 4] = [
    (CORE, &[TUI, ROOT]),
    (TUI, &[ROOT]),
    (ROOT, &[]),
    (XTASK, &[]),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Affected {
    All,
    Packages(BTreeSet<&'static str>),
}

pub fn select<'a>(paths: impl IntoIterator<Item = &'a str>) -> Affected {
    let mut direct = BTreeSet::new();
    for path in paths {
        if is_all_trigger(path) {
            return Affected::All;
        }
        direct.extend(owners(path).iter().copied());
    }
    Affected::Packages(close_over_dependents(&direct))
}

fn is_all_trigger(path: &str) -> bool {
    let file_name = path.rsplit('/').next().unwrap_or(path);
    ALL_FILE_NAMES.contains(&file_name)
        || ALL_PREFIXES.iter().any(|prefix| path.starts_with(prefix))
}

fn owners(path: &str) -> &'static [&'static str] {
    if path.starts_with("docs/") || path.ends_with(".md") {
        &[]
    } else if path == "metadata.toml" {
        &[XTASK, ROOT]
    } else if path.starts_with("crates/core/") {
        &[CORE]
    } else if path.starts_with("crates/tui/") {
        &[TUI]
    } else if path.starts_with("src/") || path.starts_with("tests/") {
        &[ROOT]
    } else if path.starts_with("xtask/") {
        &[XTASK]
    } else {
        &[]
    }
}

fn close_over_dependents(direct: &BTreeSet<&'static str>) -> BTreeSet<&'static str> {
    let mut closed = direct.clone();
    for (package, dependents) in DEPENDENTS {
        if direct.contains(package) {
            closed.extend(dependents.iter().copied());
        }
    }
    closed
}

pub fn changed_paths(root: &Path, base: &str) -> Result<Vec<String>, XtaskError> {
    let range = format!("{base}...HEAD");
    let output = Command::new("git")
        .args(["diff", "--name-only", range.as_str()])
        .current_dir(root)
        .output()
        .map_err(|source| XtaskError::Spawn {
            program: "git".to_owned(),
            source,
        })?;
    if !output.status.success() {
        return Err(XtaskError::GitDiff {
            base: base.to_owned(),
            code: output.status.code(),
        });
    }
    let text = String::from_utf8_lossy(&output.stdout);
    Ok(text
        .lines()
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect())
}

pub fn run(root: &Path, base: &str) -> Result<(), XtaskError> {
    let paths = changed_paths(root, base)?;
    match select(paths.iter().map(String::as_str)) {
        Affected::All => println!("*"),
        Affected::Packages(packages) => {
            for package in packages {
                println!("{package}");
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn packages(names: &[&'static str]) -> Affected {
        Affected::Packages(names.iter().copied().collect())
    }

    #[test]
    fn docs_and_markdown_select_nothing() {
        let paths = [
            "docs/tech/workflow.md",
            "docs/task_list/phase-0.md",
            "README.md",
            "crates/core/README.md",
        ];
        assert_eq!(select(paths), packages(&[]));
    }

    #[test]
    fn unknown_paths_select_nothing() {
        assert_eq!(
            select(["LICENSE", "AGENTS.md", ".gitignore"]),
            packages(&[])
        );
    }

    #[test]
    fn empty_input_selects_nothing() {
        assert_eq!(select([]), packages(&[]));
    }

    #[test]
    fn metadata_selects_xtask_and_root() {
        assert_eq!(select(["metadata.toml"]), packages(&[XTASK, ROOT]));
    }

    #[test]
    fn core_source_selects_core_tui_and_root() {
        assert_eq!(
            select(["crates/core/src/x.rs"]),
            packages(&[CORE, TUI, ROOT])
        );
    }

    #[test]
    fn tui_source_selects_tui_and_root() {
        assert_eq!(select(["crates/tui/src/app.rs"]), packages(&[TUI, ROOT]));
    }

    #[test]
    fn root_sources_select_root_only() {
        assert_eq!(
            select(["src/lib.rs", "tests/e2e/main.rs"]),
            packages(&[ROOT])
        );
    }

    #[test]
    fn xtask_source_selects_xtask_only() {
        assert_eq!(select(["xtask/src/affected.rs"]), packages(&[XTASK]));
    }

    #[test]
    fn lockfile_selects_every_package() {
        assert_eq!(select(["Cargo.lock"]), Affected::All);
    }

    #[test]
    fn manifests_and_toolchain_select_every_package() {
        for path in [
            "Cargo.toml",
            "crates/core/Cargo.toml",
            "rust-toolchain.toml",
            "clippy.toml",
            ".cargo/config.toml",
        ] {
            assert_eq!(select([path]), Affected::All, "{path}");
        }
    }

    #[test]
    fn workflows_locales_and_assets_select_every_package() {
        for path in [
            ".github/workflows/pr.yml",
            "locales/en-US/common.toml",
            "assets/cards/fronts/international/glyphs/a.svg",
        ] {
            assert_eq!(select([path]), Affected::All, "{path}");
        }
    }

    #[test]
    fn mixed_input_merges_packages() {
        let paths = ["xtask/src/main.rs", "crates/tui/src/lib.rs", "docs/x.md"];
        assert_eq!(select(paths), packages(&[XTASK, TUI, ROOT]));
    }

    #[test]
    fn all_wins_over_other_paths() {
        assert_eq!(select(["docs/x.md", "Cargo.lock"]), Affected::All);
    }
}
