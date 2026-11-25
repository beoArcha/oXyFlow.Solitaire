//! The look of the aura (colour, style, size) and the part of [`Settings`] that the Interface
//! section previews: those fields change on screen at once and are kept only after confirmation.

use serde::{Deserialize, Serialize};

use super::Settings;
use crate::theme::Token;

/// The colour of the aura: the theme's own colours, or one theme colour for every halo (the halos
/// still differ by symbol).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuraColour {
    #[default]
    Theme,
    Text,
    Selection,
    Hint,
}

impl AuraColour {
    /// Every colour, in the order of the settings screen.
    pub const ALL: [AuraColour; 4] = [
        AuraColour::Theme,
        AuraColour::Text,
        AuraColour::Selection,
        AuraColour::Hint,
    ];

    /// The token every halo is drawn in; `None` leaves each halo its own token.
    pub const fn token(self) -> Option<Token> {
        match self {
            AuraColour::Theme => None,
            AuraColour::Text => Some(Token::TableText),
            AuraColour::Selection => Some(Token::Selection),
            AuraColour::Hint => Some(Token::Hint),
        }
    }

    /// Localization key of the colour name.
    pub const fn label_key(self) -> &'static str {
        match self {
            AuraColour::Theme => "settings.aura_colour.theme",
            AuraColour::Text => "settings.aura_colour.text",
            AuraColour::Selection => "settings.aura_colour.selection",
            AuraColour::Hint => "settings.aura_colour.hint",
        }
    }
}

/// How the aura is drawn around a card or run: shade symbols, a line frame, or not at all (the `+`
/// and `●` marks on the cards stay).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuraStyle {
    #[default]
    Halo,
    Frame,
    MarksOnly,
}

impl AuraStyle {
    /// Every style, in the order of the settings screen.
    pub const ALL: [AuraStyle; 3] = [AuraStyle::Halo, AuraStyle::Frame, AuraStyle::MarksOnly];

    /// Localization key of the style name.
    pub const fn label_key(self) -> &'static str {
        match self {
            AuraStyle::Halo => "settings.aura_style.halo",
            AuraStyle::Frame => "settings.aura_style.frame",
            AuraStyle::MarksOnly => "settings.aura_style.marks_only",
        }
    }
}

/// How many cells wide the aura is.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuraSize {
    #[default]
    Thin,
    Thick,
}

impl AuraSize {
    /// Both sizes, in the order of the settings screen.
    pub const ALL: [AuraSize; 2] = [AuraSize::Thin, AuraSize::Thick];

    /// The number of rings of free cells the aura covers.
    pub const fn rings(self) -> u16 {
        match self {
            AuraSize::Thin => 1,
            AuraSize::Thick => 2,
        }
    }

    /// Localization key of the size name.
    pub const fn label_key(self) -> &'static str {
        match self {
            AuraSize::Thin => "settings.aura_size.thin",
            AuraSize::Thick => "settings.aura_size.thick",
        }
    }
}

impl Settings {
    /// These settings with every previewed field of the Interface section taken from `saved`. What
    /// is written to storage while a preview is open, so the preview never reaches the file before
    /// the player keeps it.
    pub fn with_interface_of(&self, saved: &Settings) -> Settings {
        Settings {
            theme: saved.theme,
            color_vision: saved.color_vision,
            card_back: saved.card_back,
            card_face: saved.card_face,
            four_color_deck: saved.four_color_deck,
            table_colour: saved.table_colour,
            table_feel: saved.table_feel,
            front_style: saved.front_style.clone(),
            back_style: saved.back_style.clone(),
            glyph_mode: saved.glyph_mode,
            console_font: saved.console_font,
            console_face: saved.console_face,
            console_font_size: saved.console_font_size,
            card_size: saved.card_size,
            aura_colour: saved.aura_colour,
            aura_style: saved.aura_style,
            aura_size: saved.aura_size,
            process_priority: saved.process_priority,
            ..self.clone()
        }
    }
}
