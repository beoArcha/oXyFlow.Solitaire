//! The options of the settings screen: their order, the group each belongs to and the texts that name
//! them. What a row shows and how one step changes [`Settings`] is in `values`. Front and back styles
//! are the folders found under `assets/cards`; the sound styles are the list the application passes in.

use crate::app::{CardFontState, FontFacts};
use crate::card_style::CardStyles;
use crate::i18n::t;
use crate::settings::{
    Aid, AuraColour, AuraSize, AuraStyle, CardBack, ColorMode, FullScreen, GlyphMode, TerminalKind,
};

/// One row of the settings screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Language,
    Colors,
    FullScreen,
    Theme,
    ColorVision,
    CardBack,
    CardFace,
    FourColorDeck,
    TableColour,
    TableFeel,
    FrontStyle,
    BackStyle,
    Glyphs,
    ConsoleFont,
    FontFace,
    FontSize,
    CardSize,
    AuraColour,
    AuraStyle,
    AuraSize,
    Priority,
    SoundStyle,
    Cues,
    Music,
    Volume,
    Aura,
    TargetMarks,
    GuidePanel,
    MovesPanel,
    AutoFinish,
    QuickMove,
}

/// Every row, top to bottom.
pub const FIELDS: [Field; 31] = [
    Field::Language,
    Field::Colors,
    Field::FullScreen,
    Field::Theme,
    Field::ColorVision,
    Field::CardBack,
    Field::CardFace,
    Field::FourColorDeck,
    Field::TableColour,
    Field::TableFeel,
    Field::FrontStyle,
    Field::BackStyle,
    Field::Glyphs,
    Field::ConsoleFont,
    Field::FontFace,
    Field::FontSize,
    Field::CardSize,
    Field::AuraColour,
    Field::AuraStyle,
    Field::AuraSize,
    Field::SoundStyle,
    Field::Cues,
    Field::Music,
    Field::Volume,
    Field::Priority,
    Field::Aura,
    Field::TargetMarks,
    Field::GuidePanel,
    Field::MovesPanel,
    Field::AutoFinish,
    Field::QuickMove,
];

/// A heading of the settings screen; the rows of a group follow its heading.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    Display,
    Interface,
    Sound,
    Performance,
    Hints,
}

impl Group {
    /// The groups, top to bottom.
    pub const ALL: [Group; 5] = [
        Group::Display,
        Group::Interface,
        Group::Sound,
        Group::Performance,
        Group::Hints,
    ];

    /// Localization key of the heading.
    pub const fn label_key(self) -> &'static str {
        match self {
            Group::Display => "settings.group.display",
            Group::Interface => "settings.group.interface",
            Group::Sound => "settings.group.sound",
            Group::Performance => "settings.group.performance",
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

pub(super) const AURA_COLOURS: [AuraColour; 4] = AuraColour::ALL;
pub(super) const AURA_STYLES: [AuraStyle; 3] = AuraStyle::ALL;
pub(super) const AURA_SIZES: [AuraSize; 2] = AuraSize::ALL;

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

/// The sentence for the console font row: what to do in Windows Terminal, in another terminal and
/// when the font is missing or refused.
fn console_font_key(facts: FontFacts) -> &'static str {
    match (facts.card_font, facts.terminal) {
        (_, TerminalKind::WindowsTerminal) => "settings.desc.console_font_wt",
        (CardFontState::Unavailable, _) => "settings.desc.console_font_other",
        (CardFontState::NotRegistered, _) => "settings.desc.console_font_missing",
        (CardFontState::Refused, _) => "settings.desc.console_font_refused",
        _ => "settings.desc.console_font",
    }
}

/// The sentence for the font face and font size rows: only the classic console lets the game change
/// the font; elsewhere the rows say that the terminal draws its font and why the game cannot.
fn font_row_key(field: Field, facts: FontFacts) -> &'static str {
    match (field, facts.terminal, facts.card_font, facts.face) {
        (_, TerminalKind::WindowsTerminal, _, _) => "settings.desc.font_terminal_wt",
        (_, TerminalKind::Linux | TerminalKind::MacOs, _, _) => "settings.desc.font_terminal_other",
        (Field::FontFace, _, CardFontState::Active, _) => "settings.desc.font_face_card",
        (Field::FontFace, _, _, CardFontState::NotRegistered) => "settings.desc.font_face_missing",
        (Field::FontFace, _, _, CardFontState::Refused) => "settings.desc.font_face_refused",
        (Field::FontFace, ..) => "settings.desc.font_face",
        _ => "settings.desc.font_size",
    }
}

impl Field {
    /// The aid a row switches, for the rows of the "Hints and help" group.
    pub const fn aid(self) -> Option<Aid> {
        match self {
            Field::Aura => Some(Aid::Aura),
            Field::TargetMarks => Some(Aid::TargetMarks),
            Field::GuidePanel => Some(Aid::GuidePanel),
            Field::MovesPanel => Some(Aid::MovesPanel),
            Field::AutoFinish => Some(Aid::AutoFinish),
            Field::QuickMove => Some(Aid::QuickMove),
            _ => None,
        }
    }

    /// Whether the row's changes show at once and are kept only after confirmation: the Interface
    /// rows, and Process priority, which is saved with them (its saved value is part of the
    /// interface baseline in `Settings::with_interface_of`).
    pub const fn is_interface(self) -> bool {
        matches!(self, Field::Priority) || matches!(self.group(), Group::Interface)
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
            Field::ColorVision => "settings.row.color_vision",
            Field::CardBack => "settings.row.card_back",
            Field::CardFace => "settings.row.card_face",
            Field::FourColorDeck => "settings.row.four_color_deck",
            Field::TableColour => "settings.row.table_colour",
            Field::TableFeel => "settings.row.table_feel",
            Field::FrontStyle => "settings.row.front_style",
            Field::BackStyle => "settings.row.back_style",
            Field::Glyphs => "settings.row.glyphs",
            Field::ConsoleFont => "settings.row.console_font",
            Field::FontFace => "settings.row.font_face",
            Field::FontSize => "settings.row.font_size",
            Field::Colors => "settings.row.colors",
            Field::CardSize => "settings.row.card_size",
            Field::AuraColour => "settings.row.aura_colour",
            Field::AuraStyle => "settings.row.aura_style",
            Field::AuraSize => "settings.row.aura_size",
            Field::Priority => "settings.row.priority",
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
            Field::Language | Field::Colors | Field::FullScreen => Group::Display,
            Field::Theme
            | Field::ColorVision
            | Field::CardBack
            | Field::CardFace
            | Field::FourColorDeck
            | Field::TableColour
            | Field::TableFeel
            | Field::FrontStyle
            | Field::BackStyle
            | Field::Glyphs
            | Field::ConsoleFont
            | Field::FontFace
            | Field::FontSize
            | Field::CardSize
            | Field::AuraColour
            | Field::AuraStyle
            | Field::AuraSize => Group::Interface,
            Field::Priority => Group::Performance,
            Field::SoundStyle | Field::Cues | Field::Music | Field::Volume => Group::Sound,
            Field::Aura
            | Field::TargetMarks
            | Field::GuidePanel
            | Field::MovesPanel
            | Field::AutoFinish
            | Field::QuickMove => Group::Hints,
        }
    }

    /// The sentence that explains the row when it is focused. The console font row says what to do
    /// in Windows Terminal, in another terminal and when the font is missing or refused.
    pub fn describe(self, facts: FontFacts) -> String {
        match self {
            Field::ConsoleFont => t(console_font_key(facts)).into_owned(),
            Field::FontFace | Field::FontSize => t(font_row_key(self, facts)).into_owned(),
            _ => t(self.description_key()).into_owned(),
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
            Field::ColorVision => "settings.desc.color_vision",
            Field::CardBack => "settings.desc.card_back",
            Field::CardFace => "settings.desc.card_face",
            Field::FourColorDeck => "settings.desc.four_color_deck",
            Field::TableColour => "settings.desc.table_colour",
            Field::TableFeel => "settings.desc.table_feel",
            Field::FrontStyle => "settings.desc.front_style",
            Field::BackStyle => "settings.desc.back_style",
            Field::Glyphs => "settings.desc.glyphs",
            Field::ConsoleFont => "settings.desc.console_font",
            Field::FontFace => "settings.desc.font_face",
            Field::FontSize => "settings.desc.font_size",
            Field::Colors => "settings.desc.colors",
            Field::CardSize => "settings.desc.card_size",
            Field::AuraColour => "settings.desc.aura_colour",
            Field::AuraStyle => "settings.desc.aura_style",
            Field::AuraSize => "settings.desc.aura_size",
            Field::Priority => "settings.desc.priority",
            Field::FullScreen => "settings.desc.full_screen",
            Field::SoundStyle => "settings.desc.sound_style",
            Field::Cues => "settings.desc.cues",
            Field::Music => "settings.desc.music",
            _ => "settings.desc.volume",
        }
    }
}
