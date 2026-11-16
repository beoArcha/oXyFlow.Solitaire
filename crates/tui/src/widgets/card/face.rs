//! The centre of a face-up card: one row of bold text with the rank and the suit symbol, above a
//! suit pictogram when the card is big enough. The text is the same localized label as the corner,
//! so no rank looks like another. The smallest card keeps a single suit symbol.

use oxysol_core::Card;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use super::art::{Scale, suit_art};
use super::paint::{Painter, centered, width_of};
use super::rank_text;
use crate::glyphs::Glyphs;

/// Rows the text and the air around it take above the pictogram of the smallest card that has one.
const TEXT_BAND: u16 = 3;
const WIDE_INTERIOR: u16 = 11;

/// The scale that fits `interior`, the card area inside the border, or `None` for the compact face.
pub(super) fn scale_for(interior: Rect) -> Option<Scale> {
    let big = |scale: Scale, gap: u16| {
        interior.height >= TEXT_BAND + gap + scale.suit_height()
            && interior.width >= scale.suit_width()
    };
    [(Scale::Large, 1), (Scale::Medium, 1), (Scale::Small, 0)]
        .into_iter()
        .find(|(scale, gap)| big(*scale, *gap))
        .map(|(scale, _)| scale)
}

fn text_row(card: Card, glyphs: &Glyphs, interior: Rect) -> String {
    let spaces = if interior.width >= WIDE_INTERIOR { 2 } else { 1 };
    format!(
        "{}{}{}",
        rank_text(card.rank),
        " ".repeat(spaces),
        glyphs.suit(card.suit)
    )
}

fn art_offset(interior: Rect, scale: Scale) -> u16 {
    let spare = interior.height.saturating_sub(1 + scale.suit_height());
    1 + (spare / 2).max(1)
}

fn paint_large(
    painter: &mut Painter<'_>,
    interior: Rect,
    scale: Scale,
    (card, glyphs): (Card, &Glyphs),
    style: Style,
) {
    let text = text_row(card, glyphs, interior);
    if width_of(&text) <= interior.width {
        let bold = style.add_modifier(Modifier::BOLD);
        painter.text(centered(interior, &text), interior.y, &text, bold);
    }
    let top = interior.y + art_offset(interior, scale);
    for (offset, row) in (0..).zip(suit_art(scale, card.suit, glyphs.unicode)) {
        painter.text(centered(interior, row), top + offset, row, style);
    }
}

fn paint_compact(painter: &mut Painter<'_>, area: Rect, card: Card, glyphs: &Glyphs, style: Style) {
    let suit = glyphs.suit(card.suit);
    painter.text(centered(area, suit), area.y + area.height / 2, suit, style);
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
        Some(scale) => paint_large(painter, interior, scale, (card, glyphs), style),
        None => paint_compact(painter, area, card, glyphs, style),
    }
}
