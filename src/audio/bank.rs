//! The sounds of one sound style: the cue and music sequences of an embedded style folder, parsed at the
//! sample rate of the output. Pure: it reads only the bytes it is given.

use std::collections::HashMap;
use std::str::Utf8Error;
use std::sync::Arc;

use oxysol_tui::{SoundCue, StyleInfo};
use thiserror::Error;

use super::sequence::{Sequence, SequenceError};
use super::style::{Programs, Style, StyleError};

const MANIFEST: &str = "style.toml";

/// One file of an embedded sound style folder.
pub struct EmbeddedFile {
    pub name: &'static str,
    pub bytes: &'static [u8],
}

/// One sound style folder, embedded at build time by `build.rs`.
pub struct EmbeddedStyle {
    pub id: &'static str,
    pub files: &'static [EmbeddedFile],
}

impl EmbeddedStyle {
    fn file(&self, name: &str) -> Option<&'static [u8]> {
        self.files
            .iter()
            .find(|file| file.name == name)
            .map(|file| file.bytes)
    }
}

include!(concat!(env!("OUT_DIR"), "/audio_styles.rs"));

/// Failure to load a sound style. Each variant names the style and, where it applies, the file.
#[derive(Debug, Error)]
pub enum BankError {
    #[error("there is no sound style `{style}`")]
    UnknownStyle { style: String },
    #[error("`{file}` of sound style `{style}` is missing")]
    MissingFile { style: String, file: String },
    #[error("the style.toml of sound style `{style}` is not UTF-8")]
    NotUtf8 {
        style: String,
        #[source]
        source: Utf8Error,
    },
    #[error("the style.toml of sound style `{style}` is invalid")]
    Manifest {
        style: String,
        #[source]
        source: StyleError,
    },
    #[error("`{file}` of sound style `{style}` is not a Standard MIDI File")]
    Midi {
        style: String,
        file: String,
        #[source]
        source: SequenceError,
    },
}

/// The sounds of one style: a sequence per cue, the optional music loop and the timbres they use.
#[derive(Debug, Default)]
pub struct Bank {
    cues: HashMap<SoundCue, Arc<Sequence>>,
    music: Option<Arc<Sequence>>,
    programs: Programs,
}

impl Bank {
    /// Loads the embedded style `style` at `sample_rate`.
    pub fn load(style: &str, sample_rate: u32) -> Result<Self, BankError> {
        let embedded = find_style(style).ok_or_else(|| BankError::UnknownStyle {
            style: style.to_owned(),
        })?;
        Self::build(style, |name| embedded.file(name), sample_rate)
    }

    /// Builds a bank from the files `lookup` returns for a file name; `style.toml` is required.
    pub fn build<'a>(
        style: &str,
        lookup: impl Fn(&str) -> Option<&'a [u8]>,
        sample_rate: u32,
    ) -> Result<Self, BankError> {
        let manifest = lookup(MANIFEST).ok_or_else(|| missing(style, MANIFEST))?;
        let text = std::str::from_utf8(manifest).map_err(|source| BankError::NotUtf8 {
            style: style.to_owned(),
            source,
        })?;
        let parsed = Style::parse(text).map_err(|source| BankError::Manifest {
            style: style.to_owned(),
            source,
        })?;
        let mut cues = HashMap::new();
        for (cue, file) in &parsed.cues {
            let sequence = load_sequence(style, &lookup, file, sample_rate)?;
            cues.insert(*cue, sequence);
        }
        let music = parsed
            .music
            .as_deref()
            .map(|file| load_sequence(style, &lookup, file, sample_rate))
            .transpose()?;
        Ok(Self {
            cues,
            music,
            programs: parsed.programs,
        })
    }

    /// The sequence that plays for `cue`, if the style has one.
    pub fn cue(&self, cue: SoundCue) -> Option<Arc<Sequence>> {
        self.cues.get(&cue).cloned()
    }

    /// The music loop, if the style has one.
    pub fn music(&self) -> Option<Arc<Sequence>> {
        self.music.clone()
    }

    pub fn programs(&self) -> &Programs {
        &self.programs
    }
}

/// Every embedded sound style, in folder order, named by the `name` of its `style.toml`. A style whose
/// name is missing or unreadable is listed by its id; selecting it reports the failure.
pub fn style_list() -> Vec<StyleInfo> {
    EMBEDDED_STYLES
        .iter()
        .map(|style| style_info(style.id, style.file(MANIFEST)))
        .collect()
}

fn style_info(id: &str, manifest: Option<&[u8]>) -> StyleInfo {
    let name = manifest
        .and_then(|bytes| std::str::from_utf8(bytes).ok())
        .and_then(|text| Style::parse(text).ok())
        .and_then(|parsed| parsed.name)
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| id.to_owned());
    StyleInfo {
        id: id.to_owned(),
        name,
    }
}

fn find_style(id: &str) -> Option<&'static EmbeddedStyle> {
    EMBEDDED_STYLES.iter().find(|style| style.id == id)
}

fn load_sequence<'a>(
    style: &str,
    lookup: &dyn Fn(&str) -> Option<&'a [u8]>,
    file: &str,
    sample_rate: u32,
) -> Result<Arc<Sequence>, BankError> {
    let bytes = lookup(file).ok_or_else(|| missing(style, file))?;
    let sequence = Sequence::from_midi(bytes, sample_rate).map_err(|source| BankError::Midi {
        style: style.to_owned(),
        file: file.to_owned(),
        source,
    })?;
    Ok(Arc::new(sequence))
}

fn missing(style: &str, file: &str) -> BankError {
    BankError::MissingFile {
        style: style.to_owned(),
        file: file.to_owned(),
    }
}

#[cfg(test)]
mod tests;
