//! The key-hint line at the bottom of a screen, generated from the key table. Each entry is a
//! mouse button: a click runs the same action as the key. The hovered or Tab-focused entry is
//! shown in reverse video without color.

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use crate::action::Action;
use crate::i18n::t;
use crate::input::primary_key;
use crate::screens::ViewCtx;
use crate::theme::Token;

/// One entry in the footer strip: an action and the column span it occupies after the last draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FooterEntry {
    pub action: Action,
    /// Start column (inclusive) in terminal coordinates, set by `draw`.
    pub col_start: u16,
    /// End column (exclusive) in terminal coordinates, set by `draw`.
    pub col_end: u16,
}

/// The column-to-action map recorded after the last `draw`; used for click and hover routing.
#[derive(Debug, Default, Clone)]
pub struct FooterMap {
    pub row: u16,
    pub entries: Vec<FooterEntry>,
}

impl FooterMap {
    /// The action for a click at `(column, row)`, or `None` when it misses every entry.
    pub fn action_at(&self, column: u16, row: u16) -> Option<Action> {
        if row != self.row {
            return None;
        }
        self.entries
            .iter()
            .find(|e| column >= e.col_start && column < e.col_end)
            .map(|e| e.action)
    }

    /// The index of the entry at `column`, or `None`.
    pub fn index_at(&self, column: u16, row: u16) -> Option<usize> {
        if row != self.row {
            return None;
        }
        self.entries
            .iter()
            .position(|e| column >= e.col_start && column < e.col_end)
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the map is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// The footer text for `actions`, for example `Enter Select   Esc Back`.
pub fn hints(actions: &[Action]) -> String {
    actions
        .iter()
        .filter_map(|action| {
            primary_key(*action).map(|key| format!("{key} {}", t(action.label_key())))
        })
        .collect::<Vec<_>>()
        .join("   ")
}

/// Draws the hints of `actions` centered in `area`, makes each entry a clickable button, and
/// returns the hit map. The entry at `focused` (by index) is shown in reverse video.
pub fn draw(
    frame: &mut Frame,
    area: Rect,
    actions: &[Action],
    ctx: &ViewCtx<'_>,
    focused: Option<usize>,
) -> FooterMap {
    draw_to(frame.buffer_mut(), area, actions, ctx, focused)
}

/// Like [`draw`] but writes directly to `buf`.
pub fn draw_to(
    buf: &mut Buffer,
    area: Rect,
    actions: &[Action],
    ctx: &ViewCtx<'_>,
    focused: Option<usize>,
) -> FooterMap {
    let entries_text: Vec<(Action, String)> = actions
        .iter()
        .filter_map(|action| {
            primary_key(*action)
                .map(|key| (*action, format!("{key} {}", t(action.label_key()))))
        })
        .collect();

    let sep = "   ";
    let sep_len = sep.chars().count() as u16;

    let total_len: u16 = entries_text
        .iter()
        .enumerate()
        .map(|(i, (_, text))| {
            let w = text.chars().count() as u16;
            if i == 0 { w } else { w + sep_len }
        })
        .sum();

    let start_x = if total_len >= area.width {
        area.x
    } else {
        area.x + (area.width - total_len) / 2
    };

    let muted = ctx.theme.style(Token::TextMuted, Token::Background);
    let highlighted = muted.add_modifier(Modifier::REVERSED);

    let mut map = FooterMap {
        row: area.y,
        entries: Vec::with_capacity(entries_text.len()),
    };

    let mut x = start_x;
    for (i, (action, text)) in entries_text.iter().enumerate() {
        if i > 0 {
            Span::styled(sep, muted).render(Rect::new(x, area.y, sep_len, 1), buf);
            x += sep_len;
        }
        let w = text.chars().count() as u16;
        let w_clamped = w.min(area.right().saturating_sub(x));
        if w_clamped == 0 {
            break;
        }
        let style = if focused == Some(i) {
            highlighted
        } else {
            muted
        };
        Span::styled(text.as_str(), style)
            .render(Rect::new(x, area.y, w_clamped, 1), buf);
        map.entries.push(FooterEntry {
            action: *action,
            col_start: x,
            col_end: x + w_clamped,
        });
        x += w_clamped;
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hints_pair_each_key_with_its_label() {
        let text = hints(&[Action::Select, Action::Back]);
        assert_eq!(text, "Enter Select   Esc Back");
    }
}
