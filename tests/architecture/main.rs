//! Architecture boundary tests.
//!
//! Validates dependency boundaries and engine purity across workspace crates.

use std::path::{Path, PathBuf};

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn core_dependencies_contain_only_serde_and_thiserror() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cargo_path = manifest_dir.join("crates/core/Cargo.toml");
    let content = std::fs::read_to_string(&cargo_path);
    assert!(content.is_ok(), "failed to read {}", cargo_path.display());
    let Ok(content) = content else {
        return;
    };

    let parsed = toml::from_str::<toml::Table>(&content);
    assert!(parsed.is_ok(), "failed to parse {}", cargo_path.display());
    let Ok(parsed) = parsed else {
        return;
    };

    let deps = parsed.get("dependencies").and_then(toml::Value::as_table);
    assert!(
        deps.is_some(),
        "missing [dependencies] in {}",
        cargo_path.display()
    );
    let Some(deps) = deps else {
        return;
    };

    let allowed = ["serde", "thiserror"];
    for key in deps.keys() {
        assert!(
            allowed.contains(&key.as_str()),
            "crates/core/Cargo.toml [dependencies] contains forbidden dependency: {key}"
        );
    }
}

#[test]
fn tui_dependencies_contain_no_disallowed_crates() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cargo_path = manifest_dir.join("crates/tui/Cargo.toml");
    let content = std::fs::read_to_string(&cargo_path);
    assert!(content.is_ok(), "failed to read {}", cargo_path.display());
    let Ok(content) = content else {
        return;
    };

    let parsed = toml::from_str::<toml::Table>(&content);
    assert!(parsed.is_ok(), "failed to parse {}", cargo_path.display());
    let Ok(parsed) = parsed else {
        return;
    };

    let deps = parsed.get("dependencies").and_then(toml::Value::as_table);
    assert!(
        deps.is_some(),
        "missing [dependencies] in {}",
        cargo_path.display()
    );
    let Some(deps) = deps else {
        return;
    };

    let forbidden = [
        "oxysol",
        "serde_json",
        "directories",
        "cpal",
        "midly",
        "rusqlite",
    ];
    for dep in forbidden {
        assert!(
            !deps.contains_key(dep),
            "crates/tui/Cargo.toml [dependencies] contains forbidden dependency: {dep}"
        );
    }
}

#[test]
fn core_source_contains_no_io_time_process_or_printing() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let core_src = manifest_dir.join("crates/core/src");
    let mut files = Vec::new();
    collect_rs_files(&core_src, &mut files);
    assert!(
        !files.is_empty(),
        "expected to find .rs files in {}",
        core_src.display()
    );

    let forbidden_patterns = [
        "std::fs",
        "std::io",
        "std::env",
        "std::time",
        "std::net",
        "std::process",
        "std::thread",
        "println!",
        "eprintln!",
    ];

    for file in &files {
        let content = std::fs::read_to_string(file);
        assert!(content.is_ok(), "failed to read {}", file.display());
        let Ok(content) = content else {
            return;
        };

        for pattern in forbidden_patterns {
            assert!(
                !content.contains(pattern),
                "file {} contains forbidden pattern `{pattern}`",
                file.display()
            );
        }
    }
}

#[test]
fn core_lib_does_not_contain_pub_mod_games() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let lib_rs = manifest_dir.join("crates/core/src/lib.rs");
    let content = std::fs::read_to_string(&lib_rs);
    assert!(content.is_ok(), "failed to read {}", lib_rs.display());
    let Ok(content) = content else {
        return;
    };

    assert!(
        !content.contains("pub mod games"),
        "crates/core/src/lib.rs must not contain `pub mod games`"
    );
}
