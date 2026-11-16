//! What an empty slot shows: a placeholder that says what the slot is for, or the target marker.

use ratatui::layout::Rect;
use ratatui::style::Style;

use super::paint::{Painter, centered};
use crate::glyphs::Glyphs;
use crate::i18n::t;

/// The placeholder of an empty slot.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Placeholder {
    /// Nothing: a cell or a column.
    #[default]
    None,
    /// A foundation, which starts with an ace.
    Ace,
    /// An empty stock that can be turned over again.
    Recycle,
    /// An empty stock with no pass left.
    Spent,
}

fn text(placeholder: Placeholder, glyphs: &Glyphs) -> Option<String> {
    match placeholder {
        Placeholder::None => None,
        Placeholder::Ace => Some(t("board.placeholder.foundation").into_owned()),
        Placeholder::Recycle => Some(glyphs.recycle.to_owned()),
        Placeholder::Spent => Some(glyphs.spent.to_owned()),
    }
}

/// Draws the placeholder, or `+` when the slot is a drop target, in the middle of `area`.
pub(super) fn paint_slot(
    painter: &mut Painter<'_>,
    area: Rect,
    (placeholder, target): (Placeholder, bool),
    glyphs: &Glyphs,
    (muted, marked): (Style, Style),
) {
    let (shown, style) = if target {
        (Some(glyphs.target.to_owned()), marked)
    } else {
        (text(placeholder, glyphs), muted)
    };
    if let Some(shown) = shown {
        painter.text(
            centered(area, &shown),
            area.y + area.height / 2,
            &shown,
            style,
        );
    }
}
