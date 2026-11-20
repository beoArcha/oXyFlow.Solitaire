//! Build script. Embeds every sound style folder under `assets/audio/` (T-07) into a generated table in
//! `OUT_DIR`, and the application icon and version information into the Windows executable with
//! `winresource`, only when the target OS is Windows. It also computes the effective version (T-21, O-36:
//! the metadata version with the patch derived from git, or `<version>+src` without git history), exports
//! it as `OXYSOL_VERSION` and gives it to the Windows `FileVersion` and `ProductVersion`; the legal
//! copyright is built from `CARGO_PKG_AUTHORS` and `CARGO_PKG_LICENSE`.

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[path = "xtask/src/version_scheme.rs"]
#[allow(
    dead_code,
    reason = "the file is shared with xtask; the build script uses a part"
)]
mod version_scheme;

const ICON_PATH: &str = "assets/icon/generated/oxysol.ico";
const AUDIO_DIR: &str = "assets/audio";
const AUDIO_TABLE: &str = "audio_styles.rs";
const MANIFEST: &str = "style.toml";

fn main() -> ExitCode {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={ICON_PATH}");
    if let Err(error) = embed_audio_styles() {
        println!("cargo:warning=cannot embed the sound styles: {error}");
        return ExitCode::FAILURE;
    }
    let version = export_version();
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return ExitCode::SUCCESS;
    }
    match embed_windows_resources(&version) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            println!("cargo:warning=cannot embed the Windows icon: {message}");
            ExitCode::FAILURE
        }
    }
}

fn export_version() -> String {
    let base = env::var("CARGO_PKG_VERSION").unwrap_or_default();
    let version = match env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => {
            let root = PathBuf::from(dir);
            for file in version_scheme::watched_files(&root) {
                println!("cargo:rerun-if-changed={}", file.display());
            }
            version_scheme::build_version(&root, &base)
        }
        Err(_) => version_scheme::source_fallback(&base),
    };
    println!("cargo:rustc-env=OXYSOL_VERSION={version}");
    version
}

fn embed_audio_styles() -> io::Result<()> {
    println!("cargo:rerun-if-changed={AUDIO_DIR}");
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").map_err(io::Error::other)?);
    let mut table = String::from("pub static EMBEDDED_STYLES: &[EmbeddedStyle] = &[\n");
    for dir in style_dirs(&manifest_dir.join(AUDIO_DIR))? {
        if let Some(entry) = style_entry(&dir)? {
            table.push_str(&entry);
        }
    }
    table.push_str("];\n");
    let out_dir = PathBuf::from(env::var("OUT_DIR").map_err(io::Error::other)?);
    fs::write(out_dir.join(AUDIO_TABLE), table)
}

fn style_dirs(audio_dir: &Path) -> io::Result<Vec<PathBuf>> {
    if !audio_dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut dirs = Vec::new();
    for entry in fs::read_dir(audio_dir)? {
        let path = entry?.path();
        if path.is_dir() {
            dirs.push(path);
        }
    }
    dirs.sort();
    Ok(dirs)
}

fn style_entry(dir: &Path) -> io::Result<Option<String>> {
    let Some(folder) = dir.file_name() else {
        return Ok(None);
    };
    let id = folder.to_string_lossy().into_owned();
    let files = sound_files(dir)?;
    if !files.iter().any(|(name, _)| name == MANIFEST) {
        println!("cargo:warning=assets/audio/{id} has no {MANIFEST} and is not a sound style");
        return Ok(None);
    }
    let mut entry = format!("    EmbeddedStyle {{\n        id: {id:?},\n        files: &[\n");
    for (name, path) in files {
        let location = path.to_string_lossy().into_owned();
        entry.push_str(&format!(
            "            EmbeddedFile {{ name: {name:?}, bytes: include_bytes!({location:?}) }},\n"
        ));
    }
    entry.push_str("        ],\n    },\n");
    Ok(Some(entry))
}

fn sound_files(dir: &Path) -> io::Result<Vec<(String, PathBuf)>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = entry.path();
        if is_sound_file(&name) && path.is_file() {
            files.push((name, path));
        }
    }
    files.sort();
    Ok(files)
}

fn is_sound_file(name: &str) -> bool {
    name == MANIFEST
        || Path::new(name)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("mid"))
}

#[cfg(windows)]
fn embed_windows_resources(version: &str) -> Result<(), std::io::Error> {
    const PRODUCT_NAME: &str = "oXySol";
    const FILE_DESCRIPTION: &str = "oXySol - terminal solitaire";
    const COMPANY_NAME: &str = "oXyFlow";
    let mut resource = winresource::WindowsResource::new();
    resource
        .set_icon(ICON_PATH)
        .set("ProductName", PRODUCT_NAME)
        .set("FileDescription", FILE_DESCRIPTION)
        .set("CompanyName", COMPANY_NAME)
        .set("LegalCopyright", &legal_copyright())
        .set("OriginalFilename", "oxysol.exe")
        .set("InternalName", "oxysol")
        .set("FileVersion", version)
        .set("ProductVersion", version);
    if let Some(numeric) = version_scheme::windows_file_version(version) {
        resource
            .set_version_info(winresource::VersionInfo::FILEVERSION, numeric)
            .set_version_info(winresource::VersionInfo::PRODUCTVERSION, numeric);
    }
    resource.compile()
}

#[cfg(windows)]
fn legal_copyright() -> String {
    const COPYRIGHT_YEAR: &str = "2026";
    let authors = env::var("CARGO_PKG_AUTHORS")
        .unwrap_or_default()
        .replace(':', ", ");
    let license = env::var("CARGO_PKG_LICENSE").unwrap_or_default();
    format!("Copyright (c) {COPYRIGHT_YEAR} {authors}. {license} License.")
}

#[cfg(not(windows))]
fn embed_windows_resources(_version: &str) -> Result<(), std::io::Error> {
    Ok(())
}
