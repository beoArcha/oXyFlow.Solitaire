//! The font face and font size rows of the Interface group. In the classic Windows console they
//! change the console font; in every other terminal they show the terminal's own zoom shortcut
//! instead of a value, because only the player can change the font there.

use super::options::Field;
use crate::app::{CardFontState, FontFacts};
use crate::i18n::{t, tf};
use crate::settings::{ConsoleFace, Settings};

/// Localization key of the name of a face choice.
const fn face_key(face: ConsoleFace) -> &'static str {
    match face {
        ConsoleFace::Default => "settings.value.console_default",
        ConsoleFace::Consolas => "settings.face.consolas",
        ConsoleFace::LucidaConsole => "settings.face.lucida_console",
        ConsoleFace::CascadiaMono => "settings.face.cascadia_mono",
    }
}

/// The value of a font row as the settings alone give it.
pub(super) fn value_text(field: Field, settings: &Settings) -> String {
    match field {
        Field::FontFace => t(face_key(settings.console_face)).into_owned(),
        _ => size_text(settings.console_font_size),
    }
}

fn size_text(size: u8) -> String {
    if size == 0 {
        return t("settings.value.console_default").into_owned();
    }
    tf("settings.value.pixels", &[("size", &size.to_string())])
}

/// The value of a font row: the terminal's shortcut where the game cannot change the font, the face
/// the oXySol font holds while it is in use, else the choice and why it did not take.
pub(super) fn value(field: Field, settings: &Settings, facts: FontFacts) -> String {
    if let Some(keys) = facts.terminal.font_keys_key() {
        return t(keys).into_owned();
    }
    let option = value_text(field, settings);
    if field != Field::FontFace {
        return option;
    }
    if facts.card_font == CardFontState::Active {
        return t("settings.value.oxysol_font_in_use").into_owned();
    }
    let reason = match facts.face {
        CardFontState::NotRegistered => "settings.value.not_installed",
        CardFontState::Refused => "settings.value.refused",
        _ => return option,
    };
    format!("{option} ({})", t(reason))
}

/// Whether a step on the row changes nothing: the game cannot change the font of this terminal, or
/// the oXySol font holds the face.
pub(super) fn is_inert(field: Field, facts: FontFacts) -> bool {
    let is_font_row = matches!(field, Field::FontFace | Field::FontSize);
    let unavailable = facts.card_font == CardFontState::Unavailable;
    let held = field == Field::FontFace && facts.card_font == CardFontState::Active;
    is_font_row && (unavailable || held)
}
