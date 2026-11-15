//! The centre of a face-up card: a large rank above a multi-row suit pictogram when the card is
//! big enough, else one suit symbol (with the rank above it on tall cards).

use oxysol_core::Card;
use ratatui::layout::Rect;
use ratatui::style::Style;

use super::art::{Scale, suit_art};
use super::paint::{Painter, centered, width_of};
use super::rank_font;
use super::rank_text;
use crate::glyphs::Glyphs;

const GAP_ABOVE_SUIT: u16 = 1;

/// The scale that fits `interior`, the card area inside the border, or `None` for the compact face.
pub(super) fn scale_for(interior: Rect) -> Option<Scale> {
    let big = |scale: Scale, gap: u16| {
        interior.height >= rank_font::HEIGHT + gap + scale.suit_height()
            && interior.width >= scale.suit_width()
    };
    [
        (Scale::Large, GAP_ABOVE_SUIT),
        (Scale::Medium, GAP_ABOVE_SUIT),
        (Scale::Small, 0),
    ]
    .into_iter()
    .find(|(scale, gap)| big(*scale, *gap))
    .map(|(scale, _)| scale)
}

fn paint_rows(painter: &mut Painter<'_>, area: Rect, top: u16, rows: &[String], style: Style) {
    for (offset, row) in (0..).zip(rows) {
        painter.text(centered(area, row), top + offset, row, style);
    }
}

fn paint_large(
    painter: &mut Painter<'_>,
    interior: Rect,
    scale: Scale,
    card: Card,
    glyphs: &Glyphs,
    style: Style,
) {
    let label = rank_text(card.rank);
    let rank = rank_font::render(&label, glyphs.unicode)
        .filter(|rows| rows.iter().all(|row| width_of(row) <= interior.width));
    let suit: Vec<String> = suit_art(scale, card.suit, glyphs.unicode)
        .iter()
        .map(|row| (*row).to_owned())
        .collect();
    let gap = if scale == Scale::Small {
        0
    } else {
        GAP_ABOVE_SUIT
    };
    let rank_rows = rank.as_ref().map_or(1, |_| rank_font::HEIGHT);
    let total = rank_rows + gap + scale.suit_height();
    let top = interior.y + interior.height.saturating_sub(total) / 2;
    match rank {
        Some(rows) => paint_rows(painter, interior, top, &rows, style),
        None => paint_rows(painter, interior, top, &[label], style),
    }
    paint_rows(painter, interior, top + rank_rows + gap, &suit, style);
}

fn paint_compact(painter: &mut Painter<'_>, area: Rect, card: Card, glyphs: &Glyphs, style: Style) {
    let middle = area.y + area.height / 2;
    let suit = glyphs.suit(card.suit);
    painter.text(centered(area, suit), middle, suit, style);
    if area.height >= 7 {
        let rank = rank_text(card.rank);
        painter.text(centered(area, &rank), middle - 1, &rank, style);
    }
}

/// Draws the centre of a face-up card of `area`, the card rectangle including its border.
pub(super) fn paint_center(
    painter: &mut Painter<'_>,
    area: Rect,
    card: Card,
    glyphs: &Glyphs,
    style: Style,
) {
    let interior = Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2);
    match scale_for(interior) {
        Some(scale) => paint_large(painter, interior, scale, card, glyphs, style),
        None => paint_compact(painter, area, card, glyphs, style),
    }
}
