//! The key-hint line at the bottom of a screen, generated from the key table.

use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use crate::action::Action;
use crate::i18n::t;
use crate::input::primary_key;
use crate::screens::ViewCtx;
use crate::theme::Token;

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

/// Draws the hints of `actions` centered in `area`.
pub fn draw(frame: &mut Frame, area: Rect, actions: &[Action], ctx: &ViewCtx<'_>) {
    let style = ctx.theme.style(Token::TextMuted, Token::Background);
    let line = Line::styled(hints(actions), style);
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
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
