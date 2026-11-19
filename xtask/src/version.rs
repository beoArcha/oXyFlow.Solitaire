//! The `version` command: prints the effective version or its release tag (T-21, O-36). The metadata
//! version gives major and minor; the patch is derived from git by `version_scheme`. `parse_version` is
//! pure; only `run` touches files and git.

use std::fs;
use std::path::Path;

use crate::check::workspace_root;
use crate::error::XtaskError;
use crate::version_scheme::{effective_version, release_tag};

const METADATA_FILE: &str = "metadata.toml";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Output {
    Metadata,
    Effective,
    Tag,
}

pub fn run(output: Output) -> Result<(), XtaskError> {
    let root = workspace_root()?;
    let base = read_version(&root)?;
    match output {
        Output::Metadata => println!("{base}"),
        Output::Effective => println!("{}", effective_version(&root, &base)?),
        Output::Tag => println!("{}", release_tag(&effective_version(&root, &base)?)),
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

fn read_version(root: &Path) -> Result<String, XtaskError> {
    let path = root.join(METADATA_FILE);
    let text = fs::read_to_string(&path).map_err(|source| XtaskError::Io {
        path: path.clone(),
        source,
    })?;
    parse_version(&path, &text)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::parse_version;
    use crate::error::XtaskError;

    const PATH: &str = "metadata.toml";

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
