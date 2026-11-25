//! The centre of a face-up card: one row of bold text with the rank and the suit symbol, above a
//! suit pictogram when the card is big enough. The text is the same label as the corner, so no rank
//! looks like another. The smallest card keeps a single suit symbol. In `Font` mode the pictogram
//! is one glyph of the oXySol font: the court emblem of a court card, else the suit.

use oxysol_core::Card;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use super::art::{Scale, suit_art};
use super::paint::{Painter, centered, width_of};
use super::rank_in;
use crate::glyphs::Glyphs;

/// Rows the text and the air around it take above the pictogram of the smallest card that has one.
const TEXT_BAND: u16 = 3;
const WIDE_INTERIOR: u16 = 11;

/// How much of a card's height is uncovered from below when drawing its centre.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Visibility {
    pub(super) rows: Option<u16>,
    pub(super) blocked: bool,
}

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
    let spaces = if interior.width >= WIDE_INTERIOR {
        2
    } else {
        1
    };
    format!(
        "{}{}{}",
        rank_in(card.rank, glyphs),
        " ".repeat(spaces),
        glyphs.suit(card.suit)
    )
}

fn font_glyph(card: Card, glyphs: &Glyphs) -> Option<&'static str> {
    let front = glyphs.front.filter(|_| glyphs.font)?;
    front
        .court_emblem(card.rank)
        .or_else(|| front.suit_glyph(card.suit))
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
    (show_text, show_pip): (bool, bool),
) {
    if show_text {
        let text = text_row(card, glyphs, interior);
        if width_of(&text) <= interior.width {
            let bold = style.add_modifier(Modifier::BOLD);
            painter.text(centered(interior, &text), interior.y, &text, bold);
        }
    }
    if !show_pip {
        return;
    }
    let top = interior.y + art_offset(interior, scale);
    if let Some(glyph) = font_glyph(card, glyphs) {
        let middle = top + scale.suit_height() / 2;
        painter.text(centered(interior, glyph), middle, glyph, style);
        return;
    }
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
    (card, glyphs): (Card, &Glyphs),
    style: Style,
    visible: Visibility,
) {
    let interior = Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2);
    let scale = scale_for(interior);
    let pip_bottom = match scale {
        Some(scale) => 1 + art_offset(interior, scale) + scale.suit_height(),
        None => area.height / 2 + 1,
    };
    let show_pip = !visible.blocked && visible.rows.is_none_or(|rows| rows >= pip_bottom);
    let show_text = visible.rows.is_none_or(|rows| rows >= 2);
    match scale {
        Some(scale) => paint_large(
            painter,
            interior,
            scale,
            (card, glyphs),
            style,
            (show_text, show_pip),
        ),
        None if show_pip => paint_compact(painter, area, card, glyphs, style),
        None => {}
    }
}
