//! The High Contrast palette: maximum separation on a pure black surface.

use ratatui::style::Color;

use super::palette::{Palette, swatch};

/// The High Contrast theme: maximum separation, no mid-tones for text. Text pairs must reach a
/// ratio of at least 7.0.
pub fn palette() -> Palette {
    Palette {
        background: swatch(0, 0, 0, Color::Black),
        surface: swatch(16, 16, 16, Color::Black),
        text: swatch(255, 255, 255, Color::White),
        text_muted: swatch(255, 255, 255, Color::White),
        accent: swatch(0, 255, 255, Color::Cyan),
        border: swatch(255, 255, 255, Color::White),
        border_focus: swatch(255, 255, 0, Color::Yellow),
        focus_fg: swatch(0, 0, 0, Color::Black),
        focus_bg: swatch(255, 255, 0, Color::Yellow),
        error: swatch(255, 128, 128, Color::LightRed),
        success: swatch(0, 255, 0, Color::LightGreen),
        table: swatch(0, 0, 0, Color::Black),
        table_text: swatch(255, 255, 255, Color::White),
        slot: swatch(255, 255, 255, Color::White),
        card_face: swatch(255, 255, 255, Color::White),
        card_back: swatch(0, 160, 255, Color::LightBlue),
        card_back_mark: swatch(0, 0, 0, Color::Black),
        card_red: swatch(160, 0, 0, Color::Red),
        card_black: swatch(0, 0, 0, Color::Black),
        target: swatch(0, 255, 0, Color::LightGreen),
        selection: swatch(0, 255, 255, Color::LightCyan),
        hint: swatch(255, 255, 0, Color::LightYellow),
        aura_focus: swatch(255, 255, 0, Color::Yellow),
        aura_carry: swatch(0, 255, 255, Color::LightCyan),
    }
}
