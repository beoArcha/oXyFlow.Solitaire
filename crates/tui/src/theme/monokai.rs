//! The Monokai palette: warm dark background with vivid accents.

use ratatui::style::Color;

use super::palette::{Palette, swatch};

/// The Monokai theme. Every text pair reaches a contrast ratio of at least 4.5.
pub fn palette() -> Palette {
    Palette {
        background: swatch(39, 40, 34, Color::Black),
        surface: swatch(46, 47, 39, Color::Black),
        text: swatch(248, 248, 242, Color::White),
        text_muted: swatch(185, 182, 164, Color::Gray),
        accent: swatch(102, 217, 239, Color::LightCyan),
        border: swatch(130, 126, 105, Color::Gray),
        border_focus: swatch(230, 219, 116, Color::LightYellow),
        focus_fg: swatch(39, 40, 34, Color::Black),
        focus_bg: swatch(230, 219, 116, Color::LightYellow),
        error: swatch(255, 110, 150, Color::LightRed),
        success: swatch(166, 226, 46, Color::LightGreen),
        table: swatch(46, 128, 70, Color::Green),
        table_text: swatch(255, 255, 255, Color::White),
        slot: swatch(166, 226, 46, Color::LightGreen),
        card_face: swatch(248, 248, 242, Color::White),
        card_back: swatch(30, 31, 26, Color::Black),
        card_back_mark: swatch(230, 219, 116, Color::LightYellow),
        card_red: swatch(180, 24, 48, Color::Red),
        card_black: swatch(39, 40, 34, Color::Black),
        target: swatch(230, 219, 116, Color::LightYellow),
        selection: swatch(118, 224, 244, Color::LightCyan),
        hint: swatch(255, 198, 70, Color::LightYellow),
        aura_focus: swatch(255, 255, 180, Color::LightYellow),
        aura_carry: swatch(210, 255, 255, Color::LightCyan),
    }
}
