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

fn read_manifest(relative: &str) -> (String, toml::Table) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    let content = std::fs::read_to_string(&path);
    assert!(content.is_ok(), "failed to read {}", path.display());
    let content = content.unwrap_or_default();
    let parsed = toml::from_str::<toml::Table>(&content);
    assert!(parsed.is_ok(), "failed to parse {}", path.display());
    (content, parsed.unwrap_or_default())
}

fn declares(table: Option<&toml::Value>, dependency: &str) -> bool {
    table
        .and_then(toml::Value::as_table)
        .is_some_and(|deps| deps.contains_key(dependency))
}

#[test]
fn rusqlite_is_declared_only_by_the_root_package() {
    let (_, root) = read_manifest("Cargo.toml");
    assert!(
        declares(root.get("dependencies"), "rusqlite"),
        "the root package must depend on rusqlite"
    );
    assert!(
        declares(
            root.get("workspace").and_then(|w| w.get("dependencies")),
            "rusqlite"
        ),
        "rusqlite must be a workspace dependency"
    );

    for member in [
        "crates/core/Cargo.toml",
        "crates/tui/Cargo.toml",
        "xtask/Cargo.toml",
    ] {
        let (content, _) = read_manifest(member);
        assert!(
            !content.contains("rusqlite"),
            "{member} must not mention rusqlite"
        );
    }
}

fn read_source(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    let content = std::fs::read_to_string(&path);
    assert!(content.is_ok(), "failed to read {}", path.display());
    content.unwrap_or_default()
}

#[test]
fn main_installs_the_panic_hook_before_the_terminal_starts() {
    let main = read_source("src/main.rs");
    let hook = main.find("panic_hook::install(");
    let run = main.find("oxysol_tui::run(");
    assert!(hook.is_some(), "src/main.rs must install the panic hook");
    assert!(run.is_some(), "src/main.rs must call oxysol_tui::run");
    assert!(
        hook < run,
        "the panic hook must be installed before oxysol_tui::run"
    );
}

#[test]
fn the_binary_uses_the_console_subsystem_and_no_console_api() {
    let mut files = Vec::new();
    collect_rs_files(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut files,
    );
    assert!(!files.is_empty());
    for file in files {
        let text = std::fs::read_to_string(&file).unwrap_or_default();
        for forbidden in ["windows_subsystem", "AttachConsole", "AllocConsole"] {
            assert!(
                !text.contains(forbidden),
                "{} must not mention {forbidden} (decision T-10)",
                file.display()
            );
        }
    }
}
