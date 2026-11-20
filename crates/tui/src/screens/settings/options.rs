//! The options of the settings screen: their order, the group each belongs to and the texts that name
//! them. What a row shows and how one step changes [`Settings`] is in `values`. Front and back styles
//! are the folders found under `assets/cards`; the sound styles are the list the application passes in.

use crate::app::{CardFontState, FontFacts};
use crate::card_style::CardStyles;
use crate::i18n::t;
use crate::settings::{Aid, CardBack, ColorMode, FullScreen, GlyphMode, TerminalKind};

/// One row of the settings screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Language,
    Theme,
    CardBack,
    FrontStyle,
    BackStyle,
    Glyphs,
    ConsoleFont,
    Colors,
    CardSize,
    FullScreen,
    SoundStyle,
    Cues,
    Music,
    Volume,
    Aura,
    TargetMarks,
    GuidePanel,
    MovesPanel,
    AutoFinish,
}

/// Every row, top to bottom.
pub const FIELDS: [Field; 19] = [
    Field::Language,
    Field::Theme,
    Field::CardBack,
    Field::FrontStyle,
    Field::BackStyle,
    Field::Glyphs,
    Field::ConsoleFont,
    Field::Colors,
    Field::CardSize,
    Field::FullScreen,
    Field::SoundStyle,
    Field::Cues,
    Field::Music,
    Field::Volume,
    Field::Aura,
    Field::TargetMarks,
    Field::GuidePanel,
    Field::MovesPanel,
    Field::AutoFinish,
];

/// A heading of the settings screen; the rows of a group follow its heading.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    Display,
    Sound,
    Hints,
}

impl Group {
    /// The groups, top to bottom.
    pub const ALL: [Group; 3] = [Group::Display, Group::Sound, Group::Hints];

    /// Localization key of the heading.
    pub const fn label_key(self) -> &'static str {
        match self {
            Group::Display => "settings.group.display",
            Group::Sound => "settings.group.sound",
            Group::Hints => "settings.group.hints",
        }
    }
}

/// Which way a change moves through the options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Next,
    Previous,
}

/// The ids of the front styles found, the default first.
pub fn front_styles() -> Vec<&'static str> {
    CardStyles::embedded()
        .fronts()
        .iter()
        .map(|style| style.id.as_str())
        .collect()
}

/// The ids of the back styles found, the default first.
pub fn back_styles() -> Vec<&'static str> {
    CardStyles::embedded()
        .backs()
        .iter()
        .map(|style| style.id.as_str())
        .collect()
}

pub(super) const CARD_BACKS: [CardBack; 3] =
    [CardBack::Theme, CardBack::Monokai, CardBack::Dracula];

pub(super) const GLYPH_MODES: [GlyphMode; 3] =
    [GlyphMode::Unicode, GlyphMode::Ascii, GlyphMode::Font];
pub(super) const COLOR_MODES: [ColorMode; 4] = [
    ColorMode::Auto,
    ColorMode::Ansi16,
    ColorMode::Ansi256,
    ColorMode::TrueColor,
];
pub(super) const FULL_SCREEN_MODES: [FullScreen; 3] =
    [FullScreen::Auto, FullScreen::On, FullScreen::Off];

impl Field {
    /// The aid a row switches, for the rows of the "Hints and help" group.
    pub const fn aid(self) -> Option<Aid> {
        match self {
            Field::Aura => Some(Aid::Aura),
            Field::TargetMarks => Some(Aid::TargetMarks),
            Field::GuidePanel => Some(Aid::GuidePanel),
            Field::MovesPanel => Some(Aid::MovesPanel),
            Field::AutoFinish => Some(Aid::AutoFinish),
            _ => None,
        }
    }

    /// Whether the row needs the sound output.
    pub const fn is_audio(self) -> bool {
        matches!(
            self,
            Field::SoundStyle | Field::Cues | Field::Music | Field::Volume
        )
    }

    /// Localization key of the row label.
    pub const fn label_key(self) -> &'static str {
        if let Some(aid) = self.aid() {
            return aid.label_key();
        }
        match self {
            Field::Language => "settings.row.language",
            Field::Theme => "settings.row.theme",
            Field::CardBack => "settings.row.card_back",
            Field::FrontStyle => "settings.row.front_style",
            Field::BackStyle => "settings.row.back_style",
            Field::Glyphs => "settings.row.glyphs",
            Field::ConsoleFont => "settings.row.console_font",
            Field::Colors => "settings.row.colors",
            Field::CardSize => "settings.row.card_size",
            Field::FullScreen => "settings.row.full_screen",
            Field::SoundStyle => "settings.row.sound_style",
            Field::Cues => "settings.row.cues",
            Field::Music => "settings.row.music",
            _ => "settings.row.volume",
        }
    }

    /// The group whose heading the row follows.
    pub const fn group(self) -> Group {
        match self {
            Field::Language
            | Field::Theme
            | Field::CardBack
            | Field::FrontStyle
            | Field::BackStyle
            | Field::Glyphs
            | Field::ConsoleFont
            | Field::Colors
            | Field::CardSize
            | Field::FullScreen => Group::Display,
            Field::SoundStyle | Field::Cues | Field::Music | Field::Volume => Group::Sound,
            Field::Aura
            | Field::TargetMarks
            | Field::GuidePanel
            | Field::MovesPanel
            | Field::AutoFinish => Group::Hints,
        }
    }

    /// The sentence that explains the row when it is focused. The console font row says what to do
    /// in Windows Terminal, in another terminal and when the font is missing or refused.
    pub fn describe(self, facts: FontFacts) -> String {
        if self != Field::ConsoleFont {
            return t(self.description_key()).into_owned();
        }
        let key = match (facts.card_font, facts.terminal) {
            (_, TerminalKind::WindowsTerminal) => "settings.desc.console_font_wt",
            (CardFontState::Unavailable, _) => "settings.desc.console_font_other",
            (CardFontState::NotRegistered, _) => "settings.desc.console_font_missing",
            (CardFontState::Refused, _) => "settings.desc.console_font_refused",
            _ => "settings.desc.console_font",
        };
        t(key).into_owned()
    }

    /// Localization key of the sentence that explains the row when it is focused.
    pub const fn description_key(self) -> &'static str {
        if let Some(aid) = self.aid() {
            return aid.description_key();
        }
        match self {
            Field::Language => "settings.desc.language",
            Field::Theme => "settings.desc.theme",
            Field::CardBack => "settings.desc.card_back",
            Field::FrontStyle => "settings.desc.front_style",
            Field::BackStyle => "settings.desc.back_style",
            Field::Glyphs => "settings.desc.glyphs",
            Field::ConsoleFont => "settings.desc.console_font",
            Field::Colors => "settings.desc.colors",
            Field::CardSize => "settings.desc.card_size",
            Field::FullScreen => "settings.desc.full_screen",
            Field::SoundStyle => "settings.desc.sound_style",
            Field::Cues => "settings.desc.cues",
            Field::Music => "settings.desc.music",
            _ => "settings.desc.volume",
        }
    }
}
