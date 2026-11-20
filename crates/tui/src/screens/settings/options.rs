//! The options of the settings screen: their order, the group each belongs to and the texts that name
//! them. What a row shows and how one step changes [`Settings`] is in `values`. Front and back styles
//! are fixed to the ids that exist today; the sound styles are the list the application passes in.

use crate::settings::{Aid, ColorMode, FullScreen, GlyphMode};

/// One row of the settings screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Language,
    Theme,
    FrontStyle,
    BackStyle,
    Glyphs,
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
pub const FIELDS: [Field; 17] = [
    Field::Language,
    Field::Theme,
    Field::FrontStyle,
    Field::BackStyle,
    Field::Glyphs,
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

/// Front style ids offered today.
pub const FRONT_STYLES: [&str; 1] = ["international"];
/// Back style ids offered today.
pub const BACK_STYLES: [&str; 1] = ["default"];

pub(super) const GLYPH_MODES: [GlyphMode; 2] = [GlyphMode::Unicode, GlyphMode::Ascii];
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
            Field::FrontStyle => "settings.row.front_style",
            Field::BackStyle => "settings.row.back_style",
            Field::Glyphs => "settings.row.glyphs",
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
            | Field::FrontStyle
            | Field::BackStyle
            | Field::Glyphs
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

    /// Localization key of the sentence that explains the row when it is focused.
    pub const fn description_key(self) -> &'static str {
        if let Some(aid) = self.aid() {
            return aid.description_key();
        }
        match self {
            Field::Language => "settings.desc.language",
            Field::Theme => "settings.desc.theme",
            Field::FrontStyle => "settings.desc.front_style",
            Field::BackStyle => "settings.desc.back_style",
            Field::Glyphs => "settings.desc.glyphs",
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
