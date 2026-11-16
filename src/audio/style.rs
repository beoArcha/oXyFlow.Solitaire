//! The `style.toml` schema of a sound style and the timbres it defines.

use std::collections::{BTreeMap, HashMap};

use oxysol_tui::SoundCue;
use serde::Deserialize;
use thiserror::Error;

const MAX_PROGRAM: u8 = 127;
const MAX_ENVELOPE_SECONDS: f32 = 10.0;

const CUE_KEYS: [(&str, SoundCue); 10] = [
    ("card_pick", SoundCue::CardPick),
    ("card_place", SoundCue::CardPlace),
    ("card_flip", SoundCue::CardFlip),
    ("stock_deal", SoundCue::StockDeal),
    ("illegal", SoundCue::Illegal),
    ("undo", SoundCue::Undo),
    ("hint", SoundCue::Hint),
    ("win", SoundCue::Win),
    ("menu_move", SoundCue::MenuMove),
    ("menu_select", SoundCue::MenuSelect),
];

/// The `style.toml` key of a cue, as used in the `[cues]` table.
pub fn cue_key(cue: SoundCue) -> &'static str {
    match cue {
        SoundCue::CardPick => "card_pick",
        SoundCue::CardPlace => "card_place",
        SoundCue::CardFlip => "card_flip",
        SoundCue::StockDeal => "stock_deal",
        SoundCue::Illegal => "illegal",
        SoundCue::Undo => "undo",
        SoundCue::Hint => "hint",
        SoundCue::Win => "win",
        SoundCue::MenuMove => "menu_move",
        SoundCue::MenuSelect => "menu_select",
    }
}

/// Failure to read or validate a `style.toml`.
#[derive(Debug, Error)]
pub enum StyleError {
    #[error("style.toml is not valid TOML for the style schema")]
    Parse(#[from] toml::de::Error),
    #[error("unknown sound cue `{0}` in [cues]")]
    UnknownCue(String),
    #[error("`{0}` in [programs] is not a MIDI program number (0..=127)")]
    InvalidProgram(String),
    #[error("program {program}: `{field}` is out of range")]
    InvalidTimbre { program: u8, field: &'static str },
    #[error("{section} names an empty file")]
    EmptyFileName { section: String },
}

/// The shape of the oscillator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Waveform {
    #[default]
    Sine,
    Square,
    Saw,
    Triangle,
}

/// The timbre of one MIDI program. Times are in seconds, `sustain` and `gain` are `0..=1`.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Timbre {
    pub waveform: Waveform,
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
    pub gain: f32,
}

impl Default for Timbre {
    fn default() -> Self {
        Self {
            waveform: Waveform::Sine,
            attack: 0.005,
            decay: 0.05,
            sustain: 0.7,
            release: 0.05,
            gain: 0.5,
        }
    }
}

impl Timbre {
    fn validate(&self, program: u8) -> Result<(), StyleError> {
        let checks = [
            ("attack", in_seconds(self.attack)),
            ("decay", in_seconds(self.decay)),
            ("release", in_seconds(self.release)),
            ("sustain", in_unit(self.sustain)),
            ("gain", in_unit(self.gain)),
        ];
        match checks.iter().find(|(_, ok)| !ok) {
            Some((field, _)) => Err(StyleError::InvalidTimbre { program, field }),
            None => Ok(()),
        }
    }
}

fn in_seconds(value: f32) -> bool {
    (0.0..=MAX_ENVELOPE_SECONDS).contains(&value)
}

fn in_unit(value: f32) -> bool {
    (0.0..=1.0).contains(&value)
}

/// The timbres of a style by MIDI program; undefined programs use the default timbre.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Programs {
    timbres: BTreeMap<u8, Timbre>,
}

impl Programs {
    pub fn new(timbres: BTreeMap<u8, Timbre>) -> Self {
        Self { timbres }
    }

    pub fn get(&self, program: u8) -> Timbre {
        self.timbres.get(&program).copied().unwrap_or_default()
    }
}

/// A parsed `style.toml`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Style {
    pub name: Option<String>,
    pub cues: HashMap<SoundCue, String>,
    pub music: Option<String>,
    pub programs: Programs,
}

#[derive(Deserialize)]
struct RawStyle {
    name: Option<String>,
    #[serde(default)]
    cues: BTreeMap<String, String>,
    music: Option<RawMusic>,
    #[serde(default)]
    programs: BTreeMap<String, Timbre>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawMusic {
    file: String,
}

impl Style {
    /// Parses and validates the text of a `style.toml`.
    pub fn parse(text: &str) -> Result<Self, StyleError> {
        let raw: RawStyle = toml::from_str(text)?;
        let cues = convert_cues(raw.cues)?;
        let music = raw
            .music
            .map(|music| non_empty("[music]", music.file))
            .transpose()?;
        let programs = convert_programs(raw.programs)?;
        Ok(Self {
            name: raw.name,
            cues,
            music,
            programs,
        })
    }
}

fn non_empty(section: &str, file: String) -> Result<String, StyleError> {
    if file.trim().is_empty() {
        return Err(StyleError::EmptyFileName {
            section: section.to_owned(),
        });
    }
    Ok(file)
}

fn convert_cues(raw: BTreeMap<String, String>) -> Result<HashMap<SoundCue, String>, StyleError> {
    let mut cues = HashMap::new();
    for (key, file) in raw {
        let cue = CUE_KEYS
            .iter()
            .find(|(name, _)| *name == key)
            .map(|(_, cue)| *cue)
            .ok_or_else(|| StyleError::UnknownCue(key.clone()))?;
        cues.insert(cue, non_empty(&format!("[cues] {key}"), file)?);
    }
    Ok(cues)
}

fn convert_programs(raw: BTreeMap<String, Timbre>) -> Result<Programs, StyleError> {
    let mut timbres = BTreeMap::new();
    for (key, timbre) in raw {
        let program = key
            .parse::<u8>()
            .ok()
            .filter(|program| *program <= MAX_PROGRAM)
            .ok_or_else(|| StyleError::InvalidProgram(key.clone()))?;
        timbre.validate(program)?;
        timbres.insert(program, timbre);
    }
    Ok(Programs::new(timbres))
}

#[cfg(test)]
mod tests;
