//! The card face options: Theme, White, Ivory, Light grey and Dark. The tint is set apart from the
//! theme; the suit colors of the theme give way to the suit colors of the tint.

use serde::{Deserialize, Serialize};

/// The color of the face of a face-up card. `Theme` keeps the face and suit colors of the theme;
/// the other values bring their own face with suit colors that keep their contrast on it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardFaceTint {
    #[default]
    Theme,
    White,
    Ivory,
    LightGrey,
    Dark,
}

impl CardFaceTint {
    /// Every tint, in the order of the settings screen.
    pub const ALL: [CardFaceTint; 5] = [
        CardFaceTint::Theme,
        CardFaceTint::White,
        CardFaceTint::Ivory,
        CardFaceTint::LightGrey,
        CardFaceTint::Dark,
    ];

    /// Localization key of the tint name.
    pub const fn label_key(self) -> &'static str {
        match self {
            CardFaceTint::Theme => "settings.card_face.theme",
            CardFaceTint::White => "settings.card_face.white",
            CardFaceTint::Ivory => "settings.card_face.ivory",
            CardFaceTint::LightGrey => "settings.card_face.light_grey",
            CardFaceTint::Dark => "settings.card_face.dark",
        }
    }
}
