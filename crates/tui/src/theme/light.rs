//! The Light palette: dark text on a light surface, green felt table.

use ratatui::style::Color;

use super::palette::{Palette, swatch};

/// The Light theme: dark text on a light surface. Text pairs must reach a ratio of at least 4.5.
pub fn palette() -> Palette {
    Palette {
        background: swatch(244, 246, 248, Color::White),
        surface: swatch(255, 255, 255, Color::White),
        text: swatch(24, 28, 36, Color::Black),
        text_muted: swatch(96, 104, 116, Color::DarkGray),
        accent: swatch(18, 76, 160, Color::Blue),
        border: swatch(192, 198, 208, Color::Gray),
        border_focus: swatch(28, 92, 196, Color::Blue),
        focus_fg: swatch(255, 255, 255, Color::White),
        focus_bg: swatch(28, 92, 196, Color::Blue),
        error: swatch(180, 24, 24, Color::Red),
        success: swatch(20, 116, 48, Color::Green),
        table: swatch(32, 114, 70, Color::Green),
        table_text: swatch(245, 252, 248, Color::White),
        slot: swatch(130, 190, 155, Color::LightGreen),
        card_face: swatch(255, 255, 255, Color::White),
        card_back: swatch(30, 64, 148, Color::Blue),
        card_back_mark: swatch(230, 240, 255, Color::White),
        card_red: swatch(176, 24, 24, Color::Red),
        card_black: swatch(20, 20, 24, Color::Black),
        target: swatch(255, 214, 102, Color::Yellow),
        selection: swatch(80, 200, 255, Color::LightCyan),
        hint: swatch(255, 165, 50, Color::LightYellow),
        aura_focus: swatch(255, 236, 140, Color::Yellow),
        aura_carry: swatch(172, 240, 255, Color::LightCyan),
    }
}
