//! The `version` command: prints the version in `metadata.toml` or the release tag derived from it
//! (T-21). `parse_version` and `release_tag` are pure; only `run` and `commit_count` touch files
//! and git.

use std::fs;
use std::path::Path;
use std::process::Command;

use crate::check::workspace_root;
use crate::error::XtaskError;

const METADATA_FILE: &str = "metadata.toml";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Output {
    Version,
    Tag { stable: bool },
}

pub fn run(output: Output) -> Result<(), XtaskError> {
    let root = workspace_root()?;
    let version = read_version(&root)?;
    match output {
        Output::Version => println!("{version}"),
        Output::Tag { stable: true } => println!("{}", release_tag(&version, 0, true)),
        Output::Tag { stable: false } => {
            let count = commit_count(&root)?;
            println!("{}", release_tag(&version, count, false));
        }
    }
    Ok(())
}

pub fn parse_version(path: &Path, text: &str) -> Result<String, XtaskError> {
    let table: toml::Table = text.parse().map_err(|source| XtaskError::Metadata {
        path: path.to_path_buf(),
        source,
    })?;
    table
        .get("project")
        .and_then(toml::Value::as_table)
        .and_then(|project| project.get("version"))
        .and_then(toml::Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| XtaskError::MissingVersion {
            path: path.to_path_buf(),
        })
}

pub fn release_tag(version: &str, commit_count: u64, stable: bool) -> String {
    if stable {
        format!("v{version}")
    } else {
        format!("v{version}-dev.{commit_count}")
    }
}

fn read_version(root: &Path) -> Result<String, XtaskError> {
    let path = root.join(METADATA_FILE);
    let text = fs::read_to_string(&path).map_err(|source| XtaskError::Io {
        path: path.clone(),
        source,
    })?;
    parse_version(&path, &text)
}

fn commit_count(root: &Path) -> Result<u64, XtaskError> {
    let output = Command::new("git")
        .args(["rev-list", "--count", "HEAD"])
        .current_dir(root)
        .output()
        .map_err(|source| XtaskError::Spawn {
            program: "git".to_owned(),
            source,
        })?;
    if !output.status.success() {
        return Err(XtaskError::GitRevList {
            code: output.status.code(),
        });
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    text.parse().map_err(|source| XtaskError::CommitCount {
        output: text,
        source,
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{parse_version, release_tag};
    use crate::error::XtaskError;

    const PATH: &str = "metadata.toml";

    #[test]
    fn dev_tag_carries_the_commit_count() {
        assert_eq!(release_tag("0.1.0", 12, false), "v0.1.0-dev.12");
    }

    #[test]
    fn stable_tag_has_no_suffix() {
        assert_eq!(release_tag("0.1.0", 12, true), "v0.1.0");
    }

    #[test]
    fn reads_the_project_version() {
        let text = "[project]\nname = \"oXySol\"\nversion = \"0.2.3\"\n";
        assert_eq!(
            parse_version(Path::new(PATH), text).ok().as_deref(),
            Some("0.2.3")
        );
    }

    #[test]
    fn version_outside_project_is_missing() {
        let text = "version = \"0.2.3\"\n[project]\nname = \"oXySol\"\n";
        assert!(matches!(
            parse_version(Path::new(PATH), text),
            Err(XtaskError::MissingVersion { .. })
        ));
    }

    #[test]
    fn missing_project_table_is_an_error() {
        assert!(matches!(
            parse_version(Path::new(PATH), "[other]\nversion = \"1.0.0\"\n"),
            Err(XtaskError::MissingVersion { .. })
        ));
    }

    #[test]
    fn non_string_version_is_an_error() {
        assert!(matches!(
            parse_version(Path::new(PATH), "[project]\nversion = 1\n"),
            Err(XtaskError::MissingVersion { .. })
        ));
    }

    #[test]
    fn invalid_toml_is_a_metadata_error() {
        assert!(matches!(
            parse_version(Path::new(PATH), "[project\nversion = \"1.0.0\"\n"),
            Err(XtaskError::Metadata { .. })
        ));
    }
}
