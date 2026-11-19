//! Focus rings: how Tab and Shift+Tab move the focus through the elements of a screen or a dialog,
//! and how the focused element is drawn. Every ring follows reading order and wraps at both ends.
//! The focused element has a marker in front of it and, for buttons, reverse video, so the focus
//! is visible without color.

use ratatui::style::{Modifier, Style};

use super::ViewCtx;
use crate::action::Action;
use crate::theme::Token;

/// Direction of a Tab press: `Some(true)` for Tab, `Some(false)` for Shift+Tab, `None` for any
/// other action.
pub const fn tab_forward(action: Action) -> Option<bool> {
    match action {
        Action::FocusNext => Some(true),
        Action::FocusPrevious => Some(false),
        _ => None,
    }
}

/// The index after `index` in a ring of `len` elements, one step forward or back, wrapping at
/// both ends. An empty ring stays at zero.
pub const fn step(index: usize, len: usize, forward: bool) -> usize {
    if len == 0 {
        0
    } else if forward {
        (index + 1) % len
    } else {
        (index + len - 1) % len
    }
}

/// The mark in front of an element: the focus glyph (`►`, or `>` in ASCII) when it has the focus,
/// else a blank of the same width so that the labels stay in one column.
pub fn marker(ctx: &ViewCtx<'_>, focused: bool) -> &'static str {
    if focused { ctx.glyphs.focus_left } else { " " }
}

/// The style of a button: Accent on Background, reversed while it has the focus.
pub fn button_style(ctx: &ViewCtx<'_>, focused: bool) -> Style {
    let style = ctx.theme.style(Token::Accent, Token::Background);
    if focused {
        style.add_modifier(Modifier::REVERSED)
    } else {
        style
    }
}

/// The text after the first focus marker on the screen, trimmed: what the focused element shows.
#[cfg(test)]
pub fn marked_text(lines: &[String], mark: &str) -> Option<String> {
    lines.iter().find_map(|line| {
        line.split_once(mark)
            .map(|(_, after)| after.trim().to_owned())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_moves_forward_and_shift_tab_back() {
        assert_eq!(tab_forward(Action::FocusNext), Some(true));
        assert_eq!(tab_forward(Action::FocusPrevious), Some(false));
        assert_eq!(tab_forward(Action::Down), None);
    }

    #[test]
    fn steps_wrap_at_both_ends() {
        assert_eq!(step(0, 3, true), 1);
        assert_eq!(step(2, 3, true), 0);
        assert_eq!(step(0, 3, false), 2);
        assert_eq!(step(2, 3, false), 1);
    }

    #[test]
    fn an_empty_ring_stays_at_zero() {
        assert_eq!(step(0, 0, true), 0);
        assert_eq!(step(0, 0, false), 0);
    }
}
