//! Player settings. Stored through `Storage` as one value; unknown or missing fields fall back to
//! their defaults so an older save keeps loading after the struct grows.

use serde::{Deserialize, Serialize};

mod aids;
mod card_back;
mod card_size;
mod dock;
mod full_screen;
pub use aids::{Aid, Aids};
pub use card_back::CardBack;
pub use card_size::{CardSizeSetting, Zoom, Zoomed, step as zoom_step};
pub use dock::DockSide;
pub use full_screen::{Decision, FullScreen, TerminalKind, decide};

use crate::audio::AudioSettings;
use crate::caps::{ColorDepth, TerminalCaps};
use crate::card_style::{DEFAULT_BACK, DEFAULT_FRONT};
use crate::i18n::Locale;
use crate::theme::ThemeId;

/// The sound style used when none is saved or the saved one no longer exists.
pub const DEFAULT_SOUND_STYLE: &str = "default";

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

/// Which halos the board draws: all three (focus, carried run, drop targets), or only the focus.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuraMode {
    #[default]
    Full,
    FocusOnly,
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
    /// Whether the player confirmed that the oXySol card font shows right (the graphics check).
    /// `Font` glyph mode draws the card font only with this set; without it the Unicode glyphs
    /// are drawn. An older saved value without the field loads as `false`.
    pub font_confirmed: bool,
    pub color_mode: ColorMode,
    pub sound_style: String,
    pub cues: bool,
    pub music: bool,
    /// Volume in percent, `0..=100`.
    pub volume: u8,
    /// Whether the board shows the guide panel beside it (when the terminal is wide enough).
    pub guide_panel: bool,
    /// Whether the board shows the possible moves panel beside it (a window below 130 columns). An
    /// older saved value without the field loads as `true`.
    pub moves_panel: bool,
    /// Where the help window sits: centred, or docked to the right or left side (P3-28). An older
    /// saved value without the field loads as centred.
    pub help_dock: DockSide,
    /// The undo mode offered in the game selection and used for the next game: `true` allows undo
    /// and redo. An older saved value without the field loads as `true`.
    pub undo_allowed: bool,
    /// Whether a new game with a fresh seed searches for a deal the solver proves winnable.
    /// An older saved value without the field loads as `false`.
    pub winnable_only: bool,
    /// Whether a forced win is played out automatically move by move. An older saved value without
    /// the field loads as `true`.
    pub auto_finish: bool,
    /// Which halos the board draws. An older saved value without the field loads as `Full`.
    pub aura: AuraMode,
    /// Whether the board marks the places a carried run can go (`+` and the target halo). An older
    /// saved value without the field loads as `true`.
    pub target_marks: bool,
    /// Whether the game fills the whole screen. An older saved value without the field loads as
    /// `Auto`.
    pub full_screen: FullScreen,
    /// Which pattern and palette face-down cards use.
    pub card_back: CardBack,
    /// The card size on the board. An older saved value without the field loads as `Auto`.
    pub card_size: CardSizeSetting,
    pub favourites: Vec<oxysol_core::Variant>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            locale: Locale::EnUs,
            theme: ThemeId::Dark,
            front_style: DEFAULT_FRONT.to_owned(),
            back_style: DEFAULT_BACK.to_owned(),
            glyph_mode: GlyphMode::Unicode,
            font_confirmed: false,
            color_mode: ColorMode::Auto,
            sound_style: DEFAULT_SOUND_STYLE.to_owned(),
            cues: true,
            music: false,
            volume: 70,
            guide_panel: true,
            moves_panel: true,
            help_dock: DockSide::Centre,
            undo_allowed: true,
            winnable_only: false,
            auto_finish: true,
            aura: AuraMode::Full,
            target_marks: true,
            full_screen: FullScreen::Auto,
            card_back: CardBack::Theme,
            card_size: CardSizeSetting::Auto,
            favourites: Vec::new(),
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

    /// The glyph mode to draw with: `Font` only when the player confirmed the oXySol font, else the
    /// Unicode glyphs, so the Private Use Area code points never show as squares.
    pub fn drawn_glyph_mode(&self) -> GlyphMode {
        match self.glyph_mode {
            GlyphMode::Font if !self.font_confirmed => GlyphMode::Unicode,
            mode => mode,
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
    fn the_guide_panel_is_on_by_default_and_old_values_without_it_keep_it_on() {
        assert!(Settings::default().guide_panel);
        let parsed = toml::from_str::<Settings>("volume = 10");
        assert!(matches!(parsed, Ok(ref s) if s.guide_panel));
        let off = toml::from_str::<Settings>("guide_panel = false");
        assert!(matches!(off, Ok(ref s) if !s.guide_panel));
    }

    #[test]
    fn the_help_window_is_centred_by_default_and_old_values_load_centred() {
        assert_eq!(Settings::default().help_dock, DockSide::Centre);
        let old = toml::from_str::<Settings>("guide_panel = false");
        assert!(matches!(old, Ok(ref s) if s.help_dock == DockSide::Centre));
        let left = toml::from_str::<Settings>("help_dock = \"left\"");
        assert!(matches!(left, Ok(ref s) if s.help_dock == DockSide::Left));
    }

    #[test]
    fn the_moves_panel_is_on_by_default_and_old_values_without_it_keep_it_on() {
        assert!(Settings::default().moves_panel);
        let parsed = toml::from_str::<Settings>("volume = 10");
        assert!(matches!(parsed, Ok(ref s) if s.moves_panel));
        let off = toml::from_str::<Settings>("moves_panel = false");
        assert!(matches!(off, Ok(ref s) if !s.moves_panel));
    }

    #[test]
    fn the_moves_panel_round_trips() {
        let settings = Settings {
            moves_panel: false,
            ..Settings::default()
        };
        let text = toml::to_string(&settings).unwrap_or_default();
        let back = toml::from_str::<Settings>(&text);
        assert!(matches!(back, Ok(ref s) if *s == settings && !s.moves_panel));
    }

    #[test]
    fn the_guide_panel_round_trips() {
        let settings = Settings {
            guide_panel: false,
            ..Settings::default()
        };
        let text = toml::to_string(&settings);
        assert!(text.is_ok());
        let back = text.map(|t| toml::from_str::<Settings>(&t));
        assert!(matches!(back, Ok(Ok(ref s)) if *s == settings && !s.guide_panel));
    }

    #[test]
    fn undo_is_allowed_by_default_and_old_values_without_it_keep_it_allowed() {
        assert!(Settings::default().undo_allowed);
        let parsed = toml::from_str::<Settings>("volume = 10");
        assert!(matches!(parsed, Ok(ref s) if s.undo_allowed));
        let off = toml::from_str::<Settings>("undo_allowed = false");
        assert!(matches!(off, Ok(ref s) if !s.undo_allowed));
    }

    #[test]
    fn winnable_only_is_off_by_default_and_round_trips() {
        assert!(!Settings::default().winnable_only);
        let parsed = toml::from_str::<Settings>("volume = 10");
        assert!(matches!(parsed, Ok(ref s) if !s.winnable_only));
        let settings = Settings {
            winnable_only: true,
            ..Settings::default()
        };
        let text = toml::to_string(&settings).unwrap_or_default();
        let back = toml::from_str::<Settings>(&text);
        assert!(matches!(back, Ok(ref s) if s.winnable_only));
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

    #[test]
    fn auto_finish_is_on_by_default_and_old_values_without_it_keep_it_on() {
        assert!(Settings::default().auto_finish);
        let parsed = toml::from_str::<Settings>("volume = 10");
        assert!(matches!(parsed, Ok(ref s) if s.auto_finish));
        let off = toml::from_str::<Settings>("auto_finish = false");
        assert!(matches!(off, Ok(ref s) if !s.auto_finish));
    }

    #[test]
    fn the_aura_is_full_by_default_and_old_values_without_it_keep_it_full() {
        assert_eq!(Settings::default().aura, AuraMode::Full);
        let parsed = toml::from_str::<Settings>("volume = 10");
        assert!(matches!(parsed, Ok(ref s) if s.aura == AuraMode::Full));
    }

    #[test]
    fn the_aura_focus_only_round_trips() {
        let settings = Settings {
            aura: AuraMode::FocusOnly,
            ..Settings::default()
        };
        let text = toml::to_string(&settings).unwrap_or_default();
        assert!(text.contains("aura = \"focus_only\""));
        let back = toml::from_str::<Settings>(&text);
        assert!(matches!(back, Ok(ref s) if *s == settings));
    }

    #[test]
    fn target_marks_are_on_by_default_and_old_values_without_them_keep_them_on() {
        assert!(Settings::default().target_marks);
        let parsed = toml::from_str::<Settings>("volume = 10");
        assert!(matches!(parsed, Ok(ref s) if s.target_marks));
        let off = toml::from_str::<Settings>("target_marks = false");
        assert!(matches!(off, Ok(ref s) if !s.target_marks));
    }

    #[test]
    fn auto_finish_round_trips() {
        let settings = Settings {
            auto_finish: false,
            ..Settings::default()
        };
        let text = toml::to_string(&settings);
        assert!(text.is_ok());
        let back = text.map(|t| toml::from_str::<Settings>(&t));
        assert!(matches!(back, Ok(Ok(ref s)) if *s == settings && !s.auto_finish));
    }

    #[test]
    fn card_back_is_theme_by_default_and_old_values_without_it_keep_theme() {
        assert_eq!(Settings::default().card_back, CardBack::Theme);
        let parsed = toml::from_str::<Settings>("volume = 10");
        assert!(matches!(parsed, Ok(ref s) if s.card_back == CardBack::Theme));
    }

    #[test]
    fn card_back_round_trips() {
        let settings = Settings {
            card_back: CardBack::Monokai,
            ..Settings::default()
        };
        let text = toml::to_string(&settings).unwrap_or_default();
        assert!(text.contains("card_back = \"monokai\""));
        let back = toml::from_str::<Settings>(&text);
        assert!(matches!(back, Ok(ref s) if *s == settings));
    }
}
