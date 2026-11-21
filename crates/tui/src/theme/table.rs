//! The felt of the table for the table colour setting. A preset must pass the contrast tests of
//! everything drawn on the table; a preset that fails is drawn in the nearest shade that passes, and
//! the player is told so.

use ratatui::style::Color;

use super::ThemeId;
use super::contrast::contrast_ratio;
use super::palette::{GRAPHIC_PAIRS, MARK_PAIRS, Palette, Swatch, TEXT_PAIRS, Token, swatch};
use crate::settings::TableColour;

#[cfg(test)]
mod tests;

/// Graphics drawn on the felt must reach 3:1 (WCAG 1.4.11), as the theme's own felt does.
const GRAPHIC_TARGET: f32 = 3.0;

/// The search takes `SHADE_STEPS` steps of `SHADE_STEP_PERCENT` from a failing preset toward white or
/// black, and stops at the first shade that passes.
const SHADE_STEPS: i32 = 20;
const SHADE_STEP_PERCENT: i32 = 5;
const WHITE: [u8; 3] = [255, 255, 255];
const BLACK: [u8; 3] = [0, 0, 0];

/// The felt the table is drawn in, and whether it is a shade of the colour asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Felt {
    pub swatch: Swatch,
    /// `true` when the colour asked for fails the contrast tests here and `swatch` is the nearest
    /// shade that passes (or the theme's felt when no shade does).
    pub shifted: bool,
}

/// The felt of a preset, or `None` for the theme's own felt.
pub fn preset(colour: TableColour) -> Option<Swatch> {
    match colour {
        TableColour::Theme => None,
        TableColour::Green => Some(swatch(40, 120, 80, Color::Green)),
        TableColour::Blue => Some(swatch(36, 74, 150, Color::Blue)),
        TableColour::Red => Some(swatch(138, 34, 44, Color::Red)),
        TableColour::Grey => Some(swatch(104, 108, 116, Color::Gray)),
        TableColour::Brown => Some(swatch(118, 76, 44, Color::Yellow)),
        TableColour::Black => Some(swatch(24, 24, 28, Color::Black)),
    }
}

/// Whether a felt of `rgb` passes every contrast test of the table in `id`: the text drawn on the
/// felt reaches the theme's contrast target, and the cards, slots and auras reach 3:1.
pub fn passes(id: ThemeId, palette: &Palette, rgb: [u8; 3]) -> bool {
    let probe = Palette {
        table: Swatch {
            rgb,
            ..palette.table
        },
        ..*palette
    };
    let text_ok = TEXT_PAIRS
        .iter()
        .filter(|(_, bg)| *bg == Token::Table)
        .all(|(fg, _)| contrast_ratio(probe.swatch(*fg).rgb, rgb) >= id.contrast_target());
    let graphic_ok = GRAPHIC_PAIRS
        .iter()
        .chain(MARK_PAIRS.iter())
        .filter(|(_, bg)| *bg == Token::Table)
        .all(|(fg, _)| contrast_ratio(probe.swatch(*fg).rgb, rgb) >= GRAPHIC_TARGET);
    text_ok && graphic_ok
}

/// The felt for `colour` in the theme `id`: the preset when it passes, else the nearest shade that
/// passes, searched in steps toward white and toward black. The theme's own felt is never shifted.
pub fn felt(id: ThemeId, palette: &Palette, colour: TableColour) -> Felt {
    let Some(requested) = preset(colour) else {
        return Felt {
            swatch: palette.table,
            shifted: false,
        };
    };
    if passes(id, palette, requested.rgb) {
        return Felt {
            swatch: requested,
            shifted: false,
        };
    }
    let shade = (1..=SHADE_STEPS)
        .flat_map(|step| {
            let percent = step * SHADE_STEP_PERCENT;
            [WHITE, BLACK].map(|toward| mix(requested.rgb, toward, percent))
        })
        .find(|rgb| passes(id, palette, *rgb));
    Felt {
        swatch: shade.map_or(palette.table, |rgb| Swatch { rgb, ..requested }),
        shifted: true,
    }
}

/// The felt for `colour` in the theme `id`, with the theme's own palette.
pub fn felt_for(id: ThemeId, colour: TableColour) -> Felt {
    felt(id, &id.palette(), colour)
}

/// `#RRGGBB`, for the note that names the shade the table is drawn in.
pub fn hex(rgb: [u8; 3]) -> String {
    format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2])
}

/// `from` moved `percent` of the way toward the colour `toward`, channel by channel. Every channel
/// stays between its start and its target, so it is always a valid channel.
pub(super) fn mix(from: [u8; 3], toward: [u8; 3], percent: i32) -> [u8; 3] {
    std::array::from_fn(|index| {
        let start = i32::from(from[index]);
        let end = i32::from(toward[index]);
        let moved = start + (end - start) * percent / 100;
        u8::try_from(moved).unwrap_or(from[index])
    })
}
