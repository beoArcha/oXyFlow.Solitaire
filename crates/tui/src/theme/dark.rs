//! The Dark palette: light text on a dark blue-gray surface, green felt table.

use ratatui::style::Color;

use super::palette::{Palette, swatch};

/// The Dark theme. Every text pair reaches a contrast ratio of at least 7.0.
pub fn palette() -> Palette {
    Palette {
        background: swatch(24, 28, 38, Color::Black),
        surface: swatch(36, 42, 58, Color::Black),
        text: swatch(232, 236, 244, Color::White),
        text_muted: swatch(184, 194, 214, Color::Gray),
        accent: swatch(140, 190, 255, Color::LightCyan),
        border: swatch(110, 122, 150, Color::DarkGray),
        border_focus: swatch(255, 214, 102, Color::Yellow),
        focus_fg: swatch(20, 24, 32, Color::Black),
        focus_bg: swatch(255, 214, 102, Color::Yellow),
        error: swatch(255, 160, 150, Color::LightRed),
        success: swatch(150, 230, 160, Color::LightGreen),
        table: swatch(24, 92, 58, Color::Green),
        table_text: swatch(236, 246, 240, Color::White),
        slot: swatch(120, 180, 150, Color::LightGreen),
        card_face: swatch(250, 250, 244, Color::White),
        card_back: swatch(28, 56, 132, Color::Blue),
        card_back_mark: swatch(226, 236, 255, Color::White),
        card_red: swatch(166, 20, 20, Color::Red),
        card_black: swatch(16, 16, 20, Color::Black),
        target: swatch(255, 214, 102, Color::Yellow),
        selection: swatch(102, 204, 255, Color::LightCyan),
        hint: swatch(255, 170, 60, Color::LightYellow),
        aura_focus: swatch(255, 232, 130, Color::Yellow),
        aura_carry: swatch(150, 228, 255, Color::LightCyan),
    }
}
