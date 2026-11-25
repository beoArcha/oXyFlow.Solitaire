//! The new game button of the `Game` compartment of the guide panel: a button on the heading row,
//! right aligned, that a click or `n` runs. The panel is drawn only from 130 columns, so the button is
//! too; the mouse meets it at the rectangle [`button`] gives, the same one it is drawn in.

use ratatui::layout::Rect;

use super::{FRAME_ROWS, SIDE};
use crate::action::Action;
use crate::i18n::{t, tf};
use crate::input::primary_key;

pub fn text() -> String {
    let key = primary_key(Action::NewGame).unwrap_or_default();
    let label = t("board.guide.new_game");
    tf(
        "board.guide.new_game_button",
        &[("key", &key), ("label", &label)],
    )
}

/// The rectangle of the new game button of the `Game` compartment of the panel `area`: right
/// aligned on the heading row, as wide as its text. `None` when the panel has no room for it beside
/// the heading.
pub fn button(area: Rect) -> Option<Rect> {
    if area.width <= 2 * SIDE || area.height <= FRAME_ROWS {
        return None;
    }
    let inner = area.width - 2 * SIDE;
    let width = u16::try_from(text().chars().count()).unwrap_or(u16::MAX);
    let heading = u16::try_from(t("board.guide.game_title").chars().count()).unwrap_or(u16::MAX);
    if heading.saturating_add(width).saturating_add(1) > inner {
        return None;
    }
    Some(Rect::new(area.right() - SIDE - width, area.y + 1, width, 1))
}
