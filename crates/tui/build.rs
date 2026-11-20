//! Build script. Embeds the card styles (D-06, P5-02): the `style.toml` of every folder under
//! `assets/cards/fronts/` and `assets/cards/backs/`, and `assets/cards/codepoints.toml`, as a table in
//! `OUT_DIR`. A new style folder needs no code change. It reruns only when something under
//! `assets/cards` changes.

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const CARDS_DIR: &str = "../../assets/cards";
const MANIFEST: &str = "style.toml";
const TABLE: &str = "card_styles.rs";

fn main() -> ExitCode {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={CARDS_DIR}");
    match embed() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            println!("cargo:warning=cannot embed the card styles: {error}");
            ExitCode::FAILURE
        }
    }
}

fn embed() -> io::Result<()> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").map_err(io::Error::other)?);
    let cards = manifest_dir.join(CARDS_DIR);
    let mut table = String::new();
    for (name, folder) in [("EMBEDDED_FRONTS", "fronts"), ("EMBEDDED_BACKS", "backs")] {
        table.push_str(&style_table(name, &cards.join(folder))?);
    }
    let registry = cards.join("codepoints.toml");
    println!("cargo:rerun-if-changed={}", registry.display());
    table.push_str(&format!(
        "pub static CODEPOINTS: &str = include_str!({:?});\n",
        registry.to_string_lossy()
    ));
    let out_dir = PathBuf::from(env::var("OUT_DIR").map_err(io::Error::other)?);
    fs::write(out_dir.join(TABLE), table)
}

fn style_table(name: &str, folder: &Path) -> io::Result<String> {
    let mut table = format!("pub static {name}: &[(&str, &str)] = &[\n");
    for dir in style_dirs(folder)? {
        let manifest = dir.join(MANIFEST);
        let Some(id) = dir.file_name().map(|id| id.to_string_lossy().into_owned()) else {
            continue;
        };
        if !manifest.is_file() {
            println!(
                "cargo:warning={} has no {MANIFEST} and is not a card style",
                dir.display()
            );
            continue;
        }
        println!("cargo:rerun-if-changed={}", manifest.display());
        table.push_str(&format!(
            "    ({id:?}, include_str!({:?})),\n",
            manifest.to_string_lossy()
        ));
    }
    table.push_str("];\n");
    Ok(table)
}

fn style_dirs(folder: &Path) -> io::Result<Vec<PathBuf>> {
    if !folder.is_dir() {
        return Ok(Vec::new());
    }
    let mut dirs = Vec::new();
    for entry in fs::read_dir(folder)? {
        let path = entry?.path();
        if path.is_dir() {
            dirs.push(path);
        }
    }
    dirs.sort();
    Ok(dirs)
}
