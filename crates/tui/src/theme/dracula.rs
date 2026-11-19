//! The Dracula palette: dark gothic background with pastel neon highlights.

use ratatui::style::Color;

use super::palette::{Palette, swatch};

/// The Dracula theme. Every text pair reaches a contrast ratio of at least 4.5.
pub fn palette() -> Palette {
    Palette {
        background: swatch(40, 42, 54, Color::Black),
        surface: swatch(68, 71, 90, Color::Black),
        text: swatch(248, 248, 242, Color::White),
        text_muted: swatch(190, 195, 225, Color::Gray),
        accent: swatch(139, 233, 253, Color::LightCyan),
        border: swatch(155, 165, 205, Color::Gray),
        border_focus: swatch(241, 250, 140, Color::LightYellow),
        focus_fg: swatch(40, 42, 54, Color::Black),
        focus_bg: swatch(241, 250, 140, Color::LightYellow),
        error: swatch(255, 155, 155, Color::LightRed),
        success: swatch(80, 250, 123, Color::LightGreen),
        table: swatch(40, 130, 94, Color::Green),
        table_text: swatch(255, 255, 255, Color::White),
        slot: swatch(140, 245, 180, Color::LightGreen),
        card_face: swatch(248, 248, 242, Color::White),
        card_back: swatch(30, 32, 42, Color::Black),
        card_back_mark: swatch(139, 233, 253, Color::LightCyan),
        card_red: swatch(185, 25, 30, Color::Red),
        card_black: swatch(40, 42, 54, Color::Black),
        target: swatch(80, 250, 123, Color::LightGreen),
        selection: swatch(139, 233, 253, Color::LightCyan),
        hint: swatch(241, 250, 140, Color::LightYellow),
        aura_focus: swatch(255, 255, 185, Color::LightYellow),
        aura_carry: swatch(230, 255, 255, Color::LightCyan),
        field_edge: false,
    }
}
