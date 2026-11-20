//! `cargo xtask fonts`: builds the oXySol Mono card font (D-06, T-16).
//!
//! Rust checks the registry `assets/cards/codepoints.toml` and the names of the glyph files, finds Python
//! 3.11 or newer and runs `xtask/fonts/build_font.py`, which validates the outlines and writes the font and
//! the preview sheet. Python is used by this command only.

mod registry;
#[cfg(test)]
mod tests;

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use registry::{Glyphs, read_registry, valid_name};

const PYTHON_CANDIDATES: [&str; 2] = ["python3", "python"];
const PYTHON_CHECK: &str =
    "import sys; import fontTools, pathops; sys.exit(0 if sys.version_info >= (3, 11) else 1)";
const MOSAIC: &str = "mosaic";
const MOSAIC_CORNERS: [&str; 4] = ["tl", "tr", "bl", "br"];

#[derive(Debug, thiserror::Error)]
pub enum FontsError {
    #[error(
        "no Python 3.11 or newer with fontTools and skia-pathops found; tried {tried}; \
         install them with `python -m pip install -r xtask/fonts/requirements.txt`"
    )]
    NoPython { tried: String },
    #[error("`{path}`: {reason}")]
    Registry { path: PathBuf, reason: String },
    #[error("`{path}`: {reason}")]
    Glyph { path: PathBuf, reason: String },
    #[error("cannot read `{path}`")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error(
        "cannot start `{program}`; install Python 3.11 or newer and `xtask/fonts/requirements.txt`"
    )]
    Spawn {
        program: String,
        #[source]
        source: io::Error,
    },
    #[error(
        "the font build stopped (exit code {code:?}); the message above names the file and the rule"
    )]
    Build { code: Option<i32> },
}

/// Every file the command reads or writes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paths {
    pub cards: PathBuf,
    pub registry: PathBuf,
    pub base: PathBuf,
    pub out: PathBuf,
    pub preview: PathBuf,
    pub script: PathBuf,
}

impl Paths {
    pub fn workspace(root: &Path) -> Self {
        let cards = root.join("assets").join("cards");
        let fonts = root.join("assets").join("fonts");
        Self {
            registry: cards.join("codepoints.toml"),
            cards,
            base: fonts.join("base").join("CascadiaMono.ttf"),
            out: fonts.join("oXySolMono-Regular.ttf"),
            preview: root.join("target").join("fonts").join("preview.html"),
            script: root.join("xtask").join("fonts").join("build_font.py"),
        }
    }
}

/// One glyph to build: its registry key, code point and the SVG it is drawn in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub key: String,
    pub code: u32,
    pub svg: PathBuf,
}

/// The glyphs with art, and the registry keys that have none yet (skipped with a note).
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Sources {
    pub files: Vec<Source>,
    pub skipped: Vec<String>,
}

pub fn run(root: &Path) -> Result<(), FontsError> {
    build(&Paths::workspace(root))
}

pub fn build(paths: &Paths) -> Result<(), FontsError> {
    let python = find_python(probe_python)?;
    let glyphs = read_registry(&paths.registry)?;
    let sources = glyph_sources(&paths.cards, &glyphs)?;
    for key in &sources.skipped {
        eprintln!("note: no SVG for `{key}`; skipped, the art is not drawn yet");
    }
    run_builder(&python, paths, &sources.files)
}

fn find_python(probe: impl Fn(&str) -> bool) -> Result<String, FontsError> {
    PYTHON_CANDIDATES
        .iter()
        .find(|candidate| probe(candidate))
        .map(|candidate| (*candidate).to_owned())
        .ok_or_else(|| FontsError::NoPython {
            tried: PYTHON_CANDIDATES
                .map(|candidate| format!("`{candidate}`"))
                .join(", "),
        })
}

fn probe_python(program: &str) -> bool {
    Command::new(program)
        .args(["-c", PYTHON_CHECK])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn run_builder(python: &str, paths: &Paths, sources: &[Source]) -> Result<(), FontsError> {
    let mut command = Command::new(python);
    command
        .arg(&paths.script)
        .arg("--base")
        .arg(&paths.base)
        .arg("--out")
        .arg(&paths.out)
        .arg("--preview")
        .arg(&paths.preview);
    for source in sources {
        command
            .arg("--glyph")
            .arg(&source.key)
            .arg(format!("{:04X}", source.code))
            .arg(&source.svg);
    }
    let status = command.status().map_err(|source| FontsError::Spawn {
        program: python.to_owned(),
        source,
    })?;
    if status.success() {
        Ok(())
    } else {
        Err(FontsError::Build {
            code: status.code(),
        })
    }
}

struct SvgFile {
    path: PathBuf,
    prefix: String,
    name: String,
}

pub fn glyph_sources(cards: &Path, glyphs: &Glyphs) -> Result<Sources, FontsError> {
    let mut sources = Sources::default();
    let mut covered = BTreeSet::new();
    for svg in svg_files(cards)? {
        for key in keys_of(&svg) {
            let code = glyphs.get(&key).copied().ok_or_else(|| FontsError::Glyph {
                path: svg.path.clone(),
                reason: format!(
                    "no entry `{key}` in codepoints.toml; add it with a free code point"
                ),
            })?;
            covered.insert(key.clone());
            sources.files.push(Source {
                key,
                code,
                svg: svg.path.clone(),
            });
        }
    }
    sources.skipped = glyphs
        .keys()
        .filter(|key| !covered.contains(*key))
        .cloned()
        .collect();
    Ok(sources)
}

fn svg_files(cards: &Path) -> Result<Vec<SvgFile>, FontsError> {
    let mut found = Vec::new();
    for group in ["fronts", "backs"] {
        for style in sorted_children(&cards.join(group))?
            .into_iter()
            .filter(|path| path.is_dir())
        {
            let glyph_dir = style.join("glyphs");
            for path in sorted_children(&glyph_dir)? {
                if path.extension().is_some_and(|extension| extension == "svg") {
                    found.push(svg_file(group, &style, path)?);
                }
            }
        }
    }
    Ok(found)
}

fn svg_file(group: &str, style: &Path, path: PathBuf) -> Result<SvgFile, FontsError> {
    let name = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default();
    if !valid_name(&name) {
        return Err(FontsError::Glyph {
            reason: "the file name must be lowercase ASCII: [a-z][a-z0-9-]*".to_owned(),
            path,
        });
    }
    Ok(SvgFile {
        path,
        prefix: format!("{group}/{}", name_of(style)),
        name,
    })
}

fn keys_of(svg: &SvgFile) -> Vec<String> {
    if svg.name == MOSAIC {
        MOSAIC_CORNERS
            .iter()
            .map(|corner| format!("{}/mosaic-{corner}", svg.prefix))
            .collect()
    } else {
        vec![format!("{}/{}", svg.prefix, svg.name)]
    }
}

fn sorted_children(dir: &Path) -> Result<Vec<PathBuf>, FontsError> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let entries = fs::read_dir(dir).map_err(|source| FontsError::Io {
        path: dir.to_owned(),
        source,
    })?;
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| FontsError::Io {
            path: dir.to_owned(),
            source,
        })?;
        paths.push(entry.path());
    }
    paths.sort();
    Ok(paths)
}

fn name_of(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}
