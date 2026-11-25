//! Low-vision palette adjustment: enforces at least 7:1 contrast for every text pair in
//! [`TEXT_PAIRS`], maps [`Token::TextMuted`] to [`Token::Text`] (no dim text), and keeps all
//! graphics and field edges at least 3:1 across every theme and color vision mode.

use ratatui::style::Color;

use super::contrast::contrast_ratio;
use super::palette::{GRAPHIC_PAIRS, MARK_PAIRS, Palette, Swatch, TEXT_PAIRS, Token};
use super::table::mix;
use super::vision::srgb_to_linear;

const TEXT_TARGET: f32 = 7.0;
const GRAPHIC_TARGET: f32 = 3.0;
const TABLE_WHITE_TARGET: f32 = 8.5;
const WHITE: [u8; 3] = [255, 255, 255];
const BLACK: [u8; 3] = [0, 0, 0];

fn luminance(rgb: [u8; 3]) -> f32 {
    0.2126 * srgb_to_linear(rgb[0])
        + 0.7152 * srgb_to_linear(rgb[1])
        + 0.0722 * srgb_to_linear(rgb[2])
}

fn push_toward(from: [u8; 3], toward: [u8; 3], bg: [u8; 3], target: f32) -> [u8; 3] {
    if contrast_ratio(from, bg) >= target {
        return from;
    }
    (1..=100)
        .map(|percent| mix(from, toward, percent))
        .find(|rgb| contrast_ratio(*rgb, bg) >= target)
        .unwrap_or(toward)
}

fn ensure_contrast(fg: Swatch, bg: [u8; 3], target: f32) -> Swatch {
    if contrast_ratio(fg.rgb, bg) >= target {
        return fg;
    }
    let toward = if contrast_ratio(WHITE, bg) >= contrast_ratio(BLACK, bg) {
        WHITE
    } else {
        BLACK
    };
    let rgb = push_toward(fg.rgb, toward, bg, target);
    let ansi16 = if rgb == WHITE || (toward == WHITE && luminance(rgb) > 0.5) {
        Color::White
    } else if rgb == BLACK || (toward == BLACK && luminance(rgb) < 0.05) {
        Color::Black
    } else {
        fg.ansi16
    };
    Swatch { rgb, ansi16 }
}

fn ensure_against_both(fg: Swatch, bg_a: [u8; 3], bg_b: [u8; 3], target: f32) -> Swatch {
    let first = ensure_contrast(fg, bg_a, target);
    let second = ensure_contrast(first, bg_b, target);
    if contrast_ratio(second.rgb, bg_a) >= target && contrast_ratio(second.rgb, bg_b) >= target {
        return second;
    }
    let toward = if contrast_ratio(WHITE, bg_a).min(contrast_ratio(WHITE, bg_b))
        >= contrast_ratio(BLACK, bg_a).min(contrast_ratio(BLACK, bg_b))
    {
        WHITE
    } else {
        BLACK
    };
    let rgb = (1..=100)
        .map(|percent| mix(fg.rgb, toward, percent))
        .find(|rgb| contrast_ratio(*rgb, bg_a) >= target && contrast_ratio(*rgb, bg_b) >= target)
        .unwrap_or(toward);
    Swatch { rgb, ..fg }
}

fn darken_for_headroom(bg: Swatch, target: f32) -> Swatch {
    if luminance(bg.rgb) >= 0.5 {
        return bg;
    }
    let rgb = push_toward(bg.rgb, BLACK, WHITE, target);
    Swatch { rgb, ..bg }
}

fn adjust_bg_for_pair(bg: Swatch, target: f32) -> Swatch {
    if contrast_ratio(WHITE, bg.rgb) >= target || contrast_ratio(BLACK, bg.rgb) >= target {
        return bg;
    }
    let toward = if luminance(bg.rgb) < 0.35 {
        BLACK
    } else {
        WHITE
    };
    let fg = if toward == BLACK { WHITE } else { BLACK };
    let rgb = push_toward(bg.rgb, toward, fg, target);
    Swatch { rgb, ..bg }
}

fn adjust_card_back(card_back: Swatch, table: [u8; 3]) -> Swatch {
    let has_mark_headroom = |rgb: [u8; 3]| {
        contrast_ratio(WHITE, rgb) >= TEXT_TARGET + 0.5
            || contrast_ratio(BLACK, rgb) >= TEXT_TARGET + 0.5
    };
    if contrast_ratio(card_back.rgb, table) >= GRAPHIC_TARGET && has_mark_headroom(card_back.rgb) {
        return card_back;
    }
    let toward = if contrast_ratio(WHITE, table) >= contrast_ratio(BLACK, table) {
        WHITE
    } else {
        BLACK
    };
    let rgb = (1..=100)
        .map(|percent| mix(card_back.rgb, toward, percent))
        .find(|rgb| contrast_ratio(*rgb, table) >= GRAPHIC_TARGET && has_mark_headroom(*rgb))
        .unwrap_or(toward);
    let ansi16 = if rgb == WHITE || (toward == WHITE && luminance(rgb) > 0.25) {
        Color::White
    } else if rgb == BLACK || (toward == BLACK && luminance(rgb) < 0.05) {
        Color::Black
    } else {
        card_back.ansi16
    };
    Swatch { rgb, ansi16 }
}

/// Adapts `palette` so every text pair reaches 7:1, `text_muted` equals `text` (no dim text),
/// and every graphic/mark/edge pair reaches at least 3:1.
#[must_use]
pub fn remap(mut palette: Palette) -> Palette {
    palette.table = darken_for_headroom(palette.table, TABLE_WHITE_TARGET);
    palette.focus_bg = adjust_bg_for_pair(palette.focus_bg, TEXT_TARGET + 0.5);
    palette.card_back = adjust_card_back(palette.card_back, palette.table.rgb);

    let bg = palette.background.rgb;
    let surface = palette.surface.rgb;
    let table = palette.table.rgb;

    palette.text = ensure_against_both(palette.text, bg, surface, TEXT_TARGET);
    palette.text_muted = palette.text;
    palette.accent = ensure_against_both(palette.accent, bg, surface, TEXT_TARGET);
    palette.error = ensure_against_both(palette.error, bg, surface, TEXT_TARGET);
    palette.success = ensure_against_both(palette.success, bg, surface, TEXT_TARGET);
    palette.focus_fg = ensure_contrast(palette.focus_fg, palette.focus_bg.rgb, TEXT_TARGET);
    palette.table_text = ensure_contrast(palette.table_text, table, TEXT_TARGET);
    palette.card_red = ensure_contrast(palette.card_red, palette.card_face.rgb, TEXT_TARGET);
    palette.card_black = ensure_contrast(palette.card_black, palette.card_face.rgb, TEXT_TARGET);
    palette.card_back_mark =
        ensure_contrast(palette.card_back_mark, palette.card_back.rgb, TEXT_TARGET);
    palette.aura_focus = ensure_contrast(palette.aura_focus, table, TEXT_TARGET);
    palette.aura_carry = ensure_contrast(palette.aura_carry, table, TEXT_TARGET);

    palette.border_focus = ensure_contrast(palette.border_focus, table, GRAPHIC_TARGET);
    palette.slot = ensure_contrast(palette.slot, table, GRAPHIC_TARGET);
    palette.target = ensure_contrast(palette.target, table, GRAPHIC_TARGET);
    palette.selection = ensure_contrast(palette.selection, table, GRAPHIC_TARGET);
    palette.hint = ensure_contrast(palette.hint, table, GRAPHIC_TARGET);

    palette.field_edge = contrast_ratio(table, bg) < GRAPHIC_TARGET;
    palette.border = ensure_against_both(palette.border, bg, surface, GRAPHIC_TARGET);
    let _ = (TEXT_PAIRS, GRAPHIC_PAIRS, MARK_PAIRS, Token::Text);
    palette
}
