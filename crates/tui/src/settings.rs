//! Player settings. Stored through `Storage` as one value; unknown or missing fields fall back to
//! their defaults so an older save keeps loading after the struct grows.

use serde::{Deserialize, Serialize};

use crate::audio::AudioSettings;
use crate::caps::{ColorDepth, TerminalCaps};
use crate::i18n::Locale;
use crate::theme::ThemeId;

/// How cards and markers are drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GlyphMode {
    Font,
    Unicode,
    Ascii,
}

/// Color mode: detected from the terminal or forced by the player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorMode {
    Auto,
    Ansi16,
    Ansi256,
    TrueColor,
}

/// Everything the player can change in the settings screen. Styles are folder ids.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub locale: Locale,
    pub theme: ThemeId,
    pub front_style: String,
    pub back_style: String,
    pub glyph_mode: GlyphMode,
    pub color_mode: ColorMode,
    pub sound_style: String,
    pub cues: bool,
    pub music: bool,
    /// Volume in percent, `0..=100`.
    pub volume: u8,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            locale: Locale::EnUs,
            theme: ThemeId::Dark,
            front_style: "international".to_owned(),
            back_style: "default".to_owned(),
            glyph_mode: GlyphMode::Unicode,
            color_mode: ColorMode::Auto,
            sound_style: "default".to_owned(),
            cues: true,
            music: false,
            volume: 70,
        }
    }
}

impl Settings {
    /// Defaults adapted to a terminal: ASCII glyphs where Unicode is not available.
    pub fn for_caps(caps: &TerminalCaps) -> Self {
        let glyph_mode = if caps.unicode {
            GlyphMode::Unicode
        } else {
            GlyphMode::Ascii
        };
        Self {
            glyph_mode,
            ..Self::default()
        }
    }

    /// The color depth to render with: the forced mode, or the detected one.
    pub fn color_depth(&self, caps: &TerminalCaps) -> ColorDepth {
        match self.color_mode {
            ColorMode::Auto => caps.color,
            ColorMode::Ansi16 => ColorDepth::Ansi16,
            ColorMode::Ansi256 => ColorDepth::Ansi256,
            ColorMode::TrueColor => ColorDepth::TrueColor,
        }
    }

    /// The part of the settings the audio output needs.
    pub fn audio(&self) -> AudioSettings {
        AudioSettings {
            style: self.sound_style.clone(),
            volume: self.volume.min(100),
            cues: self.cues,
            music: self.music,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TRUE_COLOR: TerminalCaps = TerminalCaps {
        color: ColorDepth::TrueColor,
        unicode: true,
        remote: false,
    };

    #[test]
    fn toml_round_trip_is_lossless() {
        let settings = Settings {
            locale: Locale::PlPl,
            theme: ThemeId::HighContrast,
            ..Settings::default()
        };
        let text = toml::to_string(&settings);
        assert!(text.is_ok());
        let back = text.map(|t| toml::from_str::<Settings>(&t));
        assert!(matches!(back, Ok(Ok(ref s)) if *s == settings));
    }

    #[test]
    fn missing_fields_take_defaults() {
        let parsed = toml::from_str::<Settings>("volume = 10");
        assert!(matches!(parsed, Ok(ref s) if s.volume == 10 && s.cues));
    }

    #[test]
    fn caps_without_unicode_give_ascii_glyphs() {
        let caps = TerminalCaps {
            unicode: false,
            ..TRUE_COLOR
        };
        assert_eq!(Settings::for_caps(&caps).glyph_mode, GlyphMode::Ascii);
        assert_eq!(
            Settings::for_caps(&TRUE_COLOR).glyph_mode,
            GlyphMode::Unicode
        );
    }

    #[test]
    fn forced_color_mode_overrides_detection() {
        let mut settings = Settings::default();
        assert_eq!(settings.color_depth(&TRUE_COLOR), ColorDepth::TrueColor);
        settings.color_mode = ColorMode::Ansi16;
        assert_eq!(settings.color_depth(&TRUE_COLOR), ColorDepth::Ansi16);
    }

    #[test]
    fn audio_volume_is_clamped() {
        let settings = Settings {
            volume: 250,
            ..Settings::default()
        };
        assert_eq!(settings.audio().volume, 100);
    }
}
