//! The notice shown when the terminal is smaller than the minimum grid (decision T-11).

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Wrap};

use crate::i18n::{t, tf};
use crate::screens::ViewCtx;
use crate::theme::Token;

/// Minimum number of columns.
pub const MIN_WIDTH: u16 = 100;
/// Minimum number of rows.
pub const MIN_HEIGHT: u16 = 30;

/// Whether `area` is smaller than the minimum grid.
pub fn is_too_small(area: Rect) -> bool {
    area.width < MIN_WIDTH || area.height < MIN_HEIGHT
}

/// Draws the notice with the current and the required size.
pub fn draw(frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>) {
    let size = |key: &str, width: u16, height: u16| {
        tf(
            key,
            &[
                ("width", &width.to_string()),
                ("height", &height.to_string()),
            ],
        )
    };
    let lines = vec![
        Line::styled(
            t("common.size.title").into_owned(),
            ctx.theme.style(Token::Error, Token::Background),
        ),
        Line::raw(size("common.size.current", area.width, area.height)),
        Line::raw(size("common.size.required", MIN_WIDTH, MIN_HEIGHT)),
        Line::raw(String::new()),
        Line::raw(t("common.size.hint").into_owned()),
    ];
    let [_, middle, _] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(u16::try_from(lines.len()).unwrap_or(u16::MAX)),
        Constraint::Fill(1),
    ])
    .areas(area);
    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .style(ctx.theme.base()),
        middle,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_limit_is_100_by_30() {
        assert!(!is_too_small(Rect::new(0, 0, 100, 30)));
        assert!(is_too_small(Rect::new(0, 0, 99, 30)));
        assert!(is_too_small(Rect::new(0, 0, 100, 29)));
        assert!(is_too_small(Rect::new(0, 0, 0, 0)));
    }
}
