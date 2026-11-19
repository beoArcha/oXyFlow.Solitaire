//! `cargo xtask screenshots`: renders scripted scenes of the real TUI to SVG for the player site.

mod driver;
mod grid;
mod labels;
mod motion;
mod shapes;
mod stills;
mod svg;
#[cfg(test)]
mod tests;

use std::fs;
use std::path::{Path, PathBuf};

use oxysol_core::MoveError;
use oxysol_tui::i18n::Locale;
use thiserror::Error;

use grid::Grid;
use svg::Frame;

const IMAGE_DIR: &str = "docs/site/src/img";

#[derive(Debug, Error)]
pub enum ScreenshotError {
    #[error("cannot write `{path}`")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("deal {seed} of {variant} has no winning line to play out")]
    NoWinningLine { variant: &'static str, seed: u64 },
    #[error("the winning line of the scene is not legal")]
    IllegalLine(#[source] MoveError),
    #[error("the scene `{name}` produced no frames")]
    NoFrames { name: &'static str },
}

pub enum Body {
    Still(Grid),
    Animation(Vec<Frame>),
}

pub struct Image {
    pub name: &'static str,
    pub body: Body,
}

impl Image {
    fn svg(&self, locale: Locale) -> Result<String, ScreenshotError> {
        let label = labels::label(self.name, locale);
        match &self.body {
            Body::Still(grid) => Ok(svg::still(grid, label)),
            Body::Animation(frames) => {
                svg::animation(frames, label).ok_or(ScreenshotError::NoFrames { name: self.name })
            }
        }
    }
}

/// Folder name of a language under the image directory: the site's language code.
fn language_folder(locale: Locale) -> &'static str {
    match locale {
        Locale::EnUs => "en",
        Locale::PlPl => "pl",
    }
}

pub fn run(root: &Path) -> Result<(), ScreenshotError> {
    for locale in Locale::ALL {
        let dir = root.join(IMAGE_DIR).join(language_folder(locale));
        fs::create_dir_all(&dir).map_err(|source| ScreenshotError::Write {
            path: dir.clone(),
            source,
        })?;
        let mut images = stills::all(locale);
        images.extend(motion::all(locale)?);
        for image in images {
            let path = dir.join(format!("{}.svg", image.name));
            fs::write(&path, image.svg(locale)?)
                .map_err(|source| ScreenshotError::Write { path, source })?;
        }
        println!("wrote {}", dir.display());
    }
    Ok(())
}
