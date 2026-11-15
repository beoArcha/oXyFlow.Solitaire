//! `sync-metadata`: keeps `metadata.toml` and the Cargo manifests in agreement.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use toml_edit::{DocumentMut, Item, TableLike, Value};

#[derive(Debug, thiserror::Error)]
pub enum SyncMetadataError {
    #[error("cannot locate the workspace root")]
    WorkspaceRoot,
    #[error("cannot read `{path}`")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("cannot write `{path}`")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("cannot parse Cargo.toml: {0}")]
    ParseCargo(#[source] toml_edit::TomlError),
    #[error("cannot parse metadata.toml: {0}")]
    ParseMetadata(#[source] toml_edit::TomlError),
    #[error("missing `[project]` table in metadata.toml")]
    MissingProjectTable,
    #[error("missing `[project].{0}` key in metadata.toml")]
    MissingProjectKey(&'static str),
    #[error("invalid `[workspace]` table in Cargo.toml")]
    InvalidWorkspace,
    #[error("invalid `[workspace.package]` table in Cargo.toml")]
    InvalidWorkspacePackage,
    #[error("workspace metadata in Cargo.toml is out of sync with metadata.toml")]
    OutOfSync,
}

fn workspace_root() -> Result<PathBuf, SyncMetadataError> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .ok_or(SyncMetadataError::WorkspaceRoot)
}

fn project_string<'a>(
    project: &'a dyn TableLike,
    key: &'static str,
) -> Result<&'a str, SyncMetadataError> {
    project
        .get(key)
        .and_then(|item| item.as_str())
        .ok_or(SyncMetadataError::MissingProjectKey(key))
}

fn update_table_key(table: &mut dyn TableLike, key: &str, new_value: &str) {
    if let Some(val) = table.get_mut(key).and_then(Item::as_value_mut) {
        if val.as_str() == Some(new_value) {
            return;
        }
        let decor = val.decor().clone();
        *val = Value::from(new_value);
        *val.decor_mut() = decor;
        return;
    }
    table.insert(key, Item::Value(Value::from(new_value)));
}

/// Syncs workspace package metadata in `cargo_toml` using fields from `metadata_toml`.
pub fn sync(cargo_toml: &str, metadata_toml: &str) -> Result<String, SyncMetadataError> {
    let metadata_doc: DocumentMut = metadata_toml
        .parse()
        .map_err(SyncMetadataError::ParseMetadata)?;
    let project = metadata_doc
        .get("project")
        .and_then(|item| item.as_table_like())
        .ok_or(SyncMetadataError::MissingProjectTable)?;

    let version = project_string(project, "version")?;
    let description = project_string(project, "description")?;
    let license = project_string(project, "license")?;
    let repository = project_string(project, "repository_url")?;

    let mut cargo_doc: DocumentMut = cargo_toml.parse().map_err(SyncMetadataError::ParseCargo)?;

    let ws = cargo_doc
        .entry("workspace")
        .or_insert(Item::Table(toml_edit::Table::new()))
        .as_table_like_mut()
        .ok_or(SyncMetadataError::InvalidWorkspace)?;

    let pkg = ws
        .entry("package")
        .or_insert(Item::Table(toml_edit::Table::new()))
        .as_table_like_mut()
        .ok_or(SyncMetadataError::InvalidWorkspacePackage)?;

    update_table_key(pkg, "version", version);
    update_table_key(pkg, "description", description);
    update_table_key(pkg, "license", license);
    update_table_key(pkg, "repository", repository);

    let output = cargo_doc.to_string();
    if output == cargo_toml {
        Ok(cargo_toml.to_owned())
    } else {
        Ok(output)
    }
}

pub fn run(check: bool) -> Result<(), SyncMetadataError> {
    let root = workspace_root()?;
    let cargo_path = root.join("Cargo.toml");
    let metadata_path = root.join("metadata.toml");

    let cargo_text = fs::read_to_string(&cargo_path).map_err(|source| SyncMetadataError::Io {
        path: cargo_path.clone(),
        source,
    })?;
    let metadata_text =
        fs::read_to_string(&metadata_path).map_err(|source| SyncMetadataError::Io {
            path: metadata_path.clone(),
            source,
        })?;

    let synced_text = sync(&cargo_text, &metadata_text)?;

    if check {
        if synced_text != cargo_text {
            return Err(SyncMetadataError::OutOfSync);
        }
    } else if synced_text != cargo_text {
        fs::write(&cargo_path, synced_text).map_err(|source| SyncMetadataError::Write {
            path: cargo_path,
            source,
        })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const METADATA_SAMPLE: &str = r#"[project]
organization = "oXyFlow"
repository = "oXyFlow.Solitare"
product = "oXySol"
binary = "oxysol"
version = "0.2.0"
description = "Modern Solitaire"
license = "MIT OR Apache-2.0"
repository_url = "https://github.com/beoArcha/oXyFlow.Solitare"
"#;

    const CARGO_SAMPLE: &str = r#"# Top comment
[workspace]
members = [".", "xtask"]

# Package table comment
[workspace.package]
version = "0.1.0" # version inline
edition = "2024"
license = "MIT"
# repository comment
repository = "https://github.com/old/repo"
"#;

    #[test]
    fn test_values_are_copied() {
        let synced = match sync(CARGO_SAMPLE, METADATA_SAMPLE) {
            Ok(content) => content,
            Err(err) => panic!("sync failed: {err:?}"),
        };
        assert!(synced.contains("version = \"0.2.0\" # version inline"));
        assert!(synced.contains("description = \"Modern Solitaire\""));
        assert!(synced.contains("license = \"MIT OR Apache-2.0\""));
        assert!(synced.contains("repository = \"https://github.com/beoArcha/oXyFlow.Solitare\""));
    }

    #[test]
    fn test_comments_and_ordering_survive() {
        let synced = match sync(CARGO_SAMPLE, METADATA_SAMPLE) {
            Ok(content) => content,
            Err(err) => panic!("sync failed: {err:?}"),
        };
        assert!(synced.contains("# Top comment"));
        assert!(synced.contains("# Package table comment"));
        assert!(synced.contains("# version inline"));
        assert!(synced.contains("# repository comment"));

        let version_pos = synced.find("version = \"0.2.0\"");
        let edition_pos = synced.find("edition = \"2024\"");
        let license_pos = synced.find("license = \"MIT OR Apache-2.0\"");
        let repo_pos = synced.find("repository = \"https://github.com/beoArcha/oXyFlow.Solitare\"");

        assert!(version_pos < edition_pos);
        assert!(edition_pos < license_pos);
        assert!(license_pos < repo_pos);
    }

    #[test]
    fn test_already_synced_input_is_returned_unchanged() {
        let first_pass = match sync(CARGO_SAMPLE, METADATA_SAMPLE) {
            Ok(content) => content,
            Err(err) => panic!("sync failed: {err:?}"),
        };
        let second_pass = match sync(&first_pass, METADATA_SAMPLE) {
            Ok(content) => content,
            Err(err) => panic!("resync failed: {err:?}"),
        };
        assert_eq!(first_pass, second_pass);
    }

    #[test]
    fn test_missing_project_table_is_error() {
        let metadata_without_project = r#"
[other]
version = "0.1.0"
"#;
        let err = match sync(CARGO_SAMPLE, metadata_without_project) {
            Ok(_) => panic!("expected MissingProjectTable error"),
            Err(err) => err,
        };
        assert!(matches!(err, SyncMetadataError::MissingProjectTable));
    }

    #[test]
    fn test_missing_project_key_is_error() {
        let missing_version = r#"[project]
description = "Solitaire"
license = "MIT"
repository_url = "https://example.com"
"#;
        let err = match sync(CARGO_SAMPLE, missing_version) {
            Ok(_) => panic!("expected error"),
            Err(err) => err,
        };
        assert!(matches!(
            err,
            SyncMetadataError::MissingProjectKey("version")
        ));

        let missing_desc = r#"[project]
version = "0.1.0"
license = "MIT"
repository_url = "https://example.com"
"#;
        let err = match sync(CARGO_SAMPLE, missing_desc) {
            Ok(_) => panic!("expected error"),
            Err(err) => err,
        };
        assert!(matches!(
            err,
            SyncMetadataError::MissingProjectKey("description")
        ));

        let missing_lic = r#"[project]
version = "0.1.0"
description = "Solitaire"
repository_url = "https://example.com"
"#;
        let err = match sync(CARGO_SAMPLE, missing_lic) {
            Ok(_) => panic!("expected error"),
            Err(err) => err,
        };
        assert!(matches!(
            err,
            SyncMetadataError::MissingProjectKey("license")
        ));

        let missing_repo = r#"[project]
version = "0.1.0"
description = "Solitaire"
license = "MIT"
"#;
        let err = match sync(CARGO_SAMPLE, missing_repo) {
            Ok(_) => panic!("expected error"),
            Err(err) => err,
        };
        assert!(matches!(
            err,
            SyncMetadataError::MissingProjectKey("repository_url")
        ));
    }
}
