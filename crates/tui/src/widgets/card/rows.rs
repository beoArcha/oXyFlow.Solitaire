//! The top and bottom border rows of a card: the corner labels, the marker symbols and the badge.

use ratatui::layout::Rect;

use super::paint::{Painter, width_of};
use super::{Face, Look, Mark, label};
use crate::glyphs::Glyphs;

fn fitting(symbols: Vec<&'static str>, room: u16) -> String {
    let mut shown = String::new();
    let mut used = 0;
    for symbol in symbols {
        used += width_of(symbol);
        if used > room {
            break;
        }
        shown.push_str(symbol);
    }
    shown
}

pub(super) fn paint_top_row(
    painter: &mut Painter<'_>,
    area: Rect,
    (face, mark): (Face, Mark),
    glyphs: &Glyphs,
    look: &Look,
    low_vision: bool,
) {
    let inner = area.width.saturating_sub(2);
    let label_width = match face {
        Face::Up(card) => {
            let text = label(card, glyphs);
            painter.text(area.x + 1, area.y, &text, look.fill);
            width_of(&text)
        }
        _ => 0,
    };
    let shown = fitting(
        mark.symbols(glyphs, low_vision),
        inner.saturating_sub(label_width),
    );
    let x = area.right() - 1 - width_of(&shown);
    painter.text(x, area.y, &shown, look.border);
}

/// The text of a stock badge: the count of cards, marked so that it is not read as a rank.
pub(super) fn badge_text(count: usize) -> String {
    format!("x{count}")
}

pub(super) fn paint_bottom_row(
    painter: &mut Painter<'_>,
    area: Rect,
    (face, badge): (Face, Option<usize>),
    glyphs: &Glyphs,
    look: &Look,
) {
    let text = match (badge, face) {
        (Some(count), _) => Some(badge_text(count)),
        (None, Face::Up(card)) => Some(label(card, glyphs)),
        _ => None,
    };
    let Some(text) = text else { return };
    let x = (area.right() - 1).saturating_sub(width_of(&text));
    let style = match face {
        Face::Slot(_) => look.border,
        _ => look.fill,
    };
    painter.text(x.max(area.x + 1), area.bottom() - 1, &text, style);
}
