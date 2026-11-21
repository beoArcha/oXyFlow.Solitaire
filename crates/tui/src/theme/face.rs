//! The colors of a face-up card for each card face tint. A tint sets the face and both suit colors
//! together, so each suit keeps its contrast on the face it is drawn on. `Theme` keeps the palette.

use ratatui::style::Color;

use super::palette::{Palette, Swatch, swatch};
use crate::settings::CardFaceTint;

/// The face and the two suit colors of a card face.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FaceColors {
    pub face: Swatch,
    pub red: Swatch,
    pub black: Swatch,
}

/// The colors a face-up card takes for `tint` in a theme with `palette`.
pub fn face_colors(tint: CardFaceTint, palette: &Palette) -> FaceColors {
    match tint {
        CardFaceTint::Theme => FaceColors {
            face: palette.card_face,
            red: palette.card_red,
            black: palette.card_black,
        },
        CardFaceTint::White => light_face(swatch(255, 255, 255, Color::White)),
        CardFaceTint::Ivory => light_face(swatch(255, 250, 230, Color::White)),
        CardFaceTint::LightGrey => light_face(swatch(230, 230, 232, Color::Gray)),
        CardFaceTint::Dark => FaceColors {
            face: swatch(28, 30, 38, Color::Black),
            red: swatch(255, 140, 140, Color::LightRed),
            black: swatch(236, 238, 246, Color::White),
        },
    }
}

/// The suit colors of a light face: a deep red and a near-black.
fn light_face(face: Swatch) -> FaceColors {
    FaceColors {
        face,
        red: swatch(140, 8, 8, Color::Red),
        black: swatch(16, 16, 20, Color::Black),
    }
}
