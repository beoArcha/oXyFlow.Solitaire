//! SVG rasterization with `resvg`.

use std::fs;
use std::path::Path;

use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{Options, Tree};

use super::IconError;

const MAX_SMALL_SIZE: u32 = 24;
const MAX_MEDIUM_SIZE: u32 = 40;

/// Which hand-tuned source is used for a pixel size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Small,
    Medium,
    Full,
}

impl Source {
    pub fn for_size(size: u32) -> Self {
        if size <= MAX_SMALL_SIZE {
            Self::Small
        } else if size <= MAX_MEDIUM_SIZE {
            Self::Medium
        } else {
            Self::Full
        }
    }

    fn file_name(self) -> &'static str {
        match self {
            Self::Small => "oxysol-16.svg",
            Self::Medium => "oxysol-32.svg",
            Self::Full => "oxysol.svg",
        }
    }
}

pub struct Sources {
    small: Tree,
    medium: Tree,
    full: Tree,
}

impl Sources {
    pub fn load(dir: &Path) -> Result<Self, IconError> {
        Ok(Self {
            small: load_tree(&dir.join(Source::Small.file_name()))?,
            medium: load_tree(&dir.join(Source::Medium.file_name()))?,
            full: load_tree(&dir.join(Source::Full.file_name()))?,
        })
    }

    pub fn render(&self, size: u32) -> Result<Pixmap, IconError> {
        let tree = match Source::for_size(size) {
            Source::Small => &self.small,
            Source::Medium => &self.medium,
            Source::Full => &self.full,
        };
        render_tree(tree, size)
    }
}

fn load_tree(path: &Path) -> Result<Tree, IconError> {
    let data = fs::read(path).map_err(|source| IconError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    Tree::from_data(&data, &Options::default()).map_err(|source| IconError::Svg {
        path: path.to_path_buf(),
        source,
    })
}

pub fn render_tree(tree: &Tree, size: u32) -> Result<Pixmap, IconError> {
    let mut pixmap = Pixmap::new(size, size).ok_or(IconError::Canvas { size })?;
    let scale = Transform::from_scale(
        size as f32 / tree.size().width(),
        size as f32 / tree.size().height(),
    );
    resvg::render(tree, scale, &mut pixmap.as_mut());
    Ok(pixmap)
}

pub fn encode(size: u32, pixmap: &Pixmap) -> Result<Vec<u8>, IconError> {
    pixmap.encode_png().map_err(|source| IconError::Encode {
        size,
        source: std::io::Error::other(source),
    })
}
