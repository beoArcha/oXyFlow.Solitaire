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
    /// A foundation of Spider, which is built down from a king.
    King,
    /// A pile of Pyramid that collects the pairs that add up to 13.
    Thirteen,
    /// An empty stock that can be turned over again.
    Recycle,
    /// An empty stock with no pass left.
    Spent,
}

/// The mark of a Spider foundation. Plain ASCII, so it is WGL4 and ASCII at once.
const KING_MARK: &str = "K";
/// The mark of a Pyramid pile. Plain ASCII, so it is WGL4 and ASCII at once.
const THIRTEEN_MARK: &str = "13";

fn text(placeholder: Placeholder, glyphs: &Glyphs) -> Option<String> {
    match placeholder {
        Placeholder::None => None,
        Placeholder::Ace => Some(t("board.placeholder.foundation").into_owned()),
        Placeholder::King => Some(KING_MARK.to_owned()),
        Placeholder::Thirteen => Some(THIRTEEN_MARK.to_owned()),
        Placeholder::Recycle => Some(glyphs.recycle.to_owned()),
        Placeholder::Spent => Some(glyphs.spent.to_owned()),
    }
}

/// Draws the placeholder, or `+` (`+++` in low-vision mode when wide enough) when the slot is a
/// drop target, in the middle of `area`.
pub(super) fn paint_slot(
    painter: &mut Painter<'_>,
    area: Rect,
    (placeholder, target): (Placeholder, bool),
    glyphs: &Glyphs,
    (muted, marked): (Style, Style),
    low_vision: bool,
) {
    let (shown, style) = if target {
        let mark = if low_vision && area.width >= 5 {
            format!("{0}{0}{0}", glyphs.target)
        } else {
            glyphs.target.to_owned()
        };
        (Some(mark), marked)
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
