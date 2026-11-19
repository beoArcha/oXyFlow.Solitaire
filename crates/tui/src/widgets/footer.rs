//! The key-hint line of a screen and the `Do:` row of the board: a row of segments, where each
//! button is a mouse button and a click runs the same action as its key. The drawn columns of
//! every button are recorded, so the mouse meets the row as it is drawn. The button under the
//! pointer is shown in reverse video, so the hover is visible without color.

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::Widget;

use crate::action::Action;
use crate::i18n::t;
use crate::input::primary_key;
use crate::mouse::contains;
use crate::screens::ViewCtx;
use crate::theme::Token;

/// One piece of a row: text, and the action a click on it runs when it is a button.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    pub text: String,
    pub action: Option<Action>,
}

impl Segment {
    /// Text that is shown but does nothing when clicked.
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            action: None,
        }
    }

    /// A button: a click runs `action`.
    pub fn button(action: Action, text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            action: Some(action),
        }
    }
}

/// A button drawn on the last draw: its action and the columns it covers, `col_start` inclusive
/// and `col_end` exclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FooterEntry {
    pub action: Action,
    pub col_start: u16,
    pub col_end: u16,
}

/// The buttons of the last draw of a row, all on `row`.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FooterMap {
    pub row: u16,
    pub entries: Vec<FooterEntry>,
}

impl FooterMap {
    /// The action of the button at the cell `(column, row)`, or `None` when the cell misses.
    pub fn action_at(&self, cell: (u16, u16)) -> Option<Action> {
        let (column, row) = cell;
        if row != self.row {
            return None;
        }
        self.entries
            .iter()
            .find(|entry| column >= entry.col_start && column < entry.col_end)
            .map(|entry| entry.action)
    }
}

/// One entry of a key line: the action and the localization key of the label it has on that
/// screen. Each screen keeps its own table of these, so `Enter` reads `Open` in the menu and
/// `Change` in the settings.
pub type KeyHint = (Action, &'static str);

/// The segments of a key line: each action as `key label`, separated by three spaces. Actions
/// without a key are left out.
pub fn key_segments(hints: &[KeyHint]) -> Vec<Segment> {
    let mut segments = Vec::with_capacity(hints.len() * 2);
    for (action, label) in hints {
        let Some(key) = primary_key(*action) else {
            continue;
        };
        if !segments.is_empty() {
            segments.push(Segment::plain("   "));
        }
        segments.push(Segment::button(*action, format!("{key} {}", t(label))));
    }
    segments
}

/// Draws `segments` in `area` aligned by `align` and returns the buttons. `base` is the style of
/// the text; a button under the pointer is drawn with it reversed.
pub fn draw(
    frame: &mut Frame,
    area: Rect,
    segments: &[Segment],
    align: Alignment,
    base: Style,
    ctx: &ViewCtx<'_>,
) -> FooterMap {
    draw_to(frame.buffer_mut(), area, segments, align, base, ctx)
}

/// Like [`draw`] but writes directly to `buf`.
pub fn draw_to(
    buf: &mut Buffer,
    area: Rect,
    segments: &[Segment],
    align: Alignment,
    base: Style,
    ctx: &ViewCtx<'_>,
) -> FooterMap {
    let width: u16 = segments
        .iter()
        .map(|segment| text_width(&segment.text))
        .sum();
    let mut x = match align {
        Alignment::Center => area.x + area.width.saturating_sub(width) / 2,
        Alignment::Left | Alignment::Right => area.x,
    };
    let hovered = base.add_modifier(Modifier::REVERSED);
    let mut map = FooterMap {
        row: area.y,
        entries: Vec::with_capacity(segments.len()),
    };
    for segment in segments {
        let room = area.right().saturating_sub(x);
        let w = text_width(&segment.text).min(room);
        if w == 0 {
            break;
        }
        let rect = Rect::new(x, area.y, w, 1);
        let style = match segment.action {
            Some(action) => {
                let over = ctx.pointer.is_some_and(|cell| contains(rect, cell));
                map.entries.push(FooterEntry {
                    action,
                    col_start: x,
                    col_end: x + w,
                });
                if over { hovered } else { base }
            }
            None => base,
        };
        Span::styled(segment.text.as_str(), style).render(rect, buf);
        x += w;
    }
    map
}

fn text_width(text: &str) -> u16 {
    u16::try_from(text.chars().count()).unwrap_or(u16::MAX)
}

/// The muted style of a key line on the background.
pub fn key_style(ctx: &ViewCtx<'_>) -> Style {
    ctx.theme.style(Token::TextMuted, Token::Background)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cell_inside_a_button_gives_its_action() {
        let map = FooterMap {
            row: 5,
            entries: vec![FooterEntry {
                action: Action::Hint,
                col_start: 3,
                col_end: 9,
            }],
        };
        assert_eq!(map.action_at((3, 5)), Some(Action::Hint));
        assert_eq!(map.action_at((8, 5)), Some(Action::Hint));
        assert_eq!(map.action_at((9, 5)), None);
        assert_eq!(map.action_at((4, 6)), None);
    }

    #[test]
    fn key_segments_pair_each_key_with_its_label_and_separate_them() {
        let segments = key_segments(&[
            (Action::Select, "help.action.select"),
            (Action::Back, "help.action.back"),
        ]);
        let texts: Vec<&str> = segments.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(texts, ["Enter Select", "   ", "Esc Back"]);
        assert_eq!(segments[0].action, Some(Action::Select));
        assert_eq!(segments[1].action, None);
    }
}
