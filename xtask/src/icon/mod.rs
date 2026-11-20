//! Application icon tool: renders the SVG sources to PNG sizes and writes a multi-size ICO.

mod ico;
mod render;
#[cfg(test)]
mod tests;

use std::fs;
use std::path::{Path, PathBuf};

use resvg::tiny_skia::Pixmap;
use thiserror::Error;

pub const SIZES: [u32; 10] = [16, 20, 24, 32, 40, 48, 64, 96, 128, 256];
const DOC_SIZE: u32 = 256;
const SOURCE_DIR: &str = "assets/icon";
const GENERATED_DIR: &str = "assets/icon/generated";
const ICO_NAME: &str = "oxysol.ico";
const DOC_PNG_NAME: &str = "oxysol-256.png";
const MAX_CHANNEL_DIFFERENCE: u8 = 2;

/// Error while rendering, writing or checking the application icon.
#[derive(Debug, Error)]
pub enum IconError {
    #[error("cannot read `{path}`")]
    Read {
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
    #[error("`{path}` is not a valid SVG")]
    Svg {
        path: PathBuf,
        #[source]
        source: resvg::usvg::Error,
    },
    #[error("cannot allocate a {size} px canvas")]
    Canvas { size: u32 },
    #[error("cannot encode the {size} px PNG")]
    Encode {
        size: u32,
        #[source]
        source: std::io::Error,
    },
    #[error("`{path}` holds an invalid PNG or ICO structure")]
    Decode {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("`{path}` is out of date; run `cargo xtask-art icon` and commit the result")]
    Stale { path: PathBuf },
}

/// Renders every size and either writes the generated files or, with `check`, verifies them.
pub fn run(root: &Path, check: bool) -> Result<(), IconError> {
    let sources = render::Sources::load(&root.join(SOURCE_DIR))?;
    let mut rendered = Vec::with_capacity(SIZES.len());
    for size in SIZES {
        rendered.push((size, sources.render(size)?));
    }
    let generated = root.join(GENERATED_DIR);
    if check {
        verify(&generated, &rendered)?;
        println!("icon up to date");
        Ok(())
    } else {
        write_all(&generated, &rendered)
    }
}

fn write_all(generated: &Path, rendered: &[(u32, Pixmap)]) -> Result<(), IconError> {
    fs::create_dir_all(generated).map_err(|source| IconError::Write {
        path: generated.to_path_buf(),
        source,
    })?;
    let mut entries = Vec::with_capacity(rendered.len());
    for (size, pixmap) in rendered {
        entries.push((*size, render::encode(*size, pixmap)?));
    }
    write_file(&generated.join(ICO_NAME), &ico::build(&entries))?;
    for (size, png) in &entries {
        if *size == DOC_SIZE {
            write_file(&generated.join(DOC_PNG_NAME), png)?;
        }
    }
    Ok(())
}

fn write_file(path: &Path, bytes: &[u8]) -> Result<(), IconError> {
    fs::write(path, bytes).map_err(|source| IconError::Write {
        path: path.to_path_buf(),
        source,
    })?;
    println!("wrote {}", path.display());
    Ok(())
}

fn verify(generated: &Path, rendered: &[(u32, Pixmap)]) -> Result<(), IconError> {
    let ico_path = generated.join(ICO_NAME);
    let bytes = read_file(&ico_path)?;
    let committed = ico::parse(&bytes).map_err(|source| IconError::Decode {
        path: ico_path.clone(),
        source,
    })?;
    if committed.len() != rendered.len() {
        return Err(IconError::Stale { path: ico_path });
    }
    for ((size, png), (fresh_size, fresh)) in committed.iter().zip(rendered) {
        if size != fresh_size || !same_picture(&ico_path, png, fresh)? {
            return Err(IconError::Stale { path: ico_path });
        }
    }
    let doc_path = generated.join(DOC_PNG_NAME);
    let doc_bytes = read_file(&doc_path)?;
    for (_, fresh) in rendered.iter().filter(|(size, _)| *size == DOC_SIZE) {
        if !same_picture(&doc_path, &doc_bytes, fresh)? {
            return Err(IconError::Stale { path: doc_path });
        }
    }
    Ok(())
}

fn read_file(path: &Path) -> Result<Vec<u8>, IconError> {
    fs::read(path).map_err(|source| IconError::Read {
        path: path.to_path_buf(),
        source,
    })
}

fn same_picture(path: &Path, png: &[u8], fresh: &Pixmap) -> Result<bool, IconError> {
    let committed = Pixmap::decode_png(png).map_err(|source| IconError::Decode {
        path: path.to_path_buf(),
        source: std::io::Error::other(source),
    })?;
    Ok(pixels_match(&committed, fresh))
}

/// Compares two canvases allowing a tiny per-channel difference, so that rasterizer rounding
/// differences between CPUs and platforms do not fail the check.
fn pixels_match(left: &Pixmap, right: &Pixmap) -> bool {
    left.width() == right.width()
        && left.height() == right.height()
        && left
            .data()
            .iter()
            .zip(right.data())
            .all(|(a, b)| a.abs_diff(*b) <= MAX_CHANNEL_DIFFERENCE)
}
