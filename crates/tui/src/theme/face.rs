//! The colors of a face-up card for each card face tint. A tint sets the face and both suit colors
//! together, so each suit keeps its contrast on the face it is drawn on. `Theme` keeps the palette.

use oxysol_core::Suit;
use ratatui::style::Color;

use super::palette::{Palette, Swatch, swatch};
use crate::settings::{CardFaceTint, ColorVision};

/// The face and the two suit colors of a card face.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FaceColors {
    pub face: Swatch,
    pub red: Swatch,
    pub black: Swatch,
}

/// The face and all four suit colors of a card face.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SuitColors {
    pub face: Swatch,
    pub spades: Swatch,
    pub hearts: Swatch,
    pub diamonds: Swatch,
    pub clubs: Swatch,
}

impl SuitColors {
    /// The ink swatch for `suit`.
    pub const fn for_suit(self, suit: Suit) -> Swatch {
        match suit {
            Suit::Spades => self.spades,
            Suit::Hearts => self.hearts,
            Suit::Diamonds => self.diamonds,
            Suit::Clubs => self.clubs,
        }
    }
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

/// The four-color suit swatches for `tint` in a theme with `palette` and `vision`.
pub fn suit_colors(tint: CardFaceTint, palette: &Palette, vision: ColorVision) -> SuitColors {
    let base = face_colors(tint, palette);
    if tint == CardFaceTint::Dark {
        return dark_four_color(base.face, vision);
    }
    light_four_color(base.face, base.black, vision)
}

fn light_four_color(face: Swatch, spades: Swatch, vision: ColorVision) -> SuitColors {
    match vision {
        ColorVision::Normal => SuitColors {
            face,
            spades,
            hearts: swatch(140, 8, 8, Color::Red),
            diamonds: swatch(0, 68, 146, Color::Blue),
            clubs: swatch(0, 84, 38, Color::Green),
        },
        ColorVision::Protanopia | ColorVision::Deuteranopia => SuitColors {
            face,
            spades: swatch(0, 0, 0, Color::Black),
            hearts: swatch(140, 34, 0, Color::Red),
            diamonds: swatch(0, 68, 146, Color::Blue),
            clubs: swatch(0, 86, 64, Color::Green),
        },
        ColorVision::Tritanopia => SuitColors {
            face,
            spades: swatch(0, 0, 0, Color::Black),
            hearts: swatch(144, 12, 28, Color::Red),
            diamonds: swatch(0, 42, 118, Color::Blue),
            clubs: swatch(24, 86, 42, Color::Green),
        },
        ColorVision::Monochrome => SuitColors {
            face,
            spades: swatch(0, 0, 0, Color::Black),
            hearts: swatch(24, 24, 24, Color::DarkGray),
            diamonds: swatch(48, 48, 48, Color::DarkGray),
            clubs: swatch(74, 74, 74, Color::Gray),
        },
    }
}

fn dark_four_color(face: Swatch, vision: ColorVision) -> SuitColors {
    match vision {
        ColorVision::Normal => SuitColors {
            face,
            spades: swatch(236, 238, 246, Color::White),
            hearts: swatch(255, 140, 140, Color::LightRed),
            diamonds: swatch(125, 200, 255, Color::LightBlue),
            clubs: swatch(130, 235, 160, Color::LightGreen),
        },
        ColorVision::Protanopia | ColorVision::Deuteranopia => SuitColors {
            face,
            spades: swatch(240, 242, 248, Color::White),
            hearts: swatch(255, 165, 95, Color::LightRed),
            diamonds: swatch(120, 205, 255, Color::LightBlue),
            clubs: swatch(245, 232, 105, Color::LightYellow),
        },
        ColorVision::Tritanopia => SuitColors {
            face,
            spades: swatch(240, 242, 248, Color::White),
            hearts: swatch(255, 140, 148, Color::LightRed),
            diamonds: swatch(120, 230, 225, Color::LightCyan),
            clubs: swatch(165, 240, 160, Color::LightGreen),
        },
        ColorVision::Monochrome => SuitColors {
            face,
            spades: swatch(248, 248, 248, Color::White),
            hearts: swatch(220, 220, 220, Color::White),
            diamonds: swatch(195, 195, 195, Color::Gray),
            clubs: swatch(170, 170, 170, Color::Gray),
        },
    }
}
