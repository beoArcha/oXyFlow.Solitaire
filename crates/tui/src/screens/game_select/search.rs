//! The search field and the legend of the game menu. The field is a button while closed (`/` or a
//! click opens it) and shows what was typed with a cursor while open. Both forms are text, so the
//! state is visible without color.

use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::action::Action;
use crate::i18n::{t, tf};
use crate::input::primary_key;
use crate::screens::ViewCtx;
use crate::screens::focus::button_style;
use crate::theme::Token;

const CURSOR: &str = "_";

/// Draws the field centered in `area`; `text` is `None` while the field is closed. Returns the
/// cells that open the field on a click.
pub(super) fn draw_field(
    frame: &mut Frame,
    area: Rect,
    text: Option<&str>,
    ctx: &ViewCtx<'_>,
) -> Rect {
    let label = t("menu.search.label");
    let field = match text {
        Some(typed) => format!("[ / {label}: {typed}{CURSOR} ]"),
        None => format!("[ / {label} ]"),
    };
    let tail = text.map_or_else(String::new, |_| format!("  {}", t("menu.search.clear")));
    let width = u16::try_from(field.chars().count()).unwrap_or(u16::MAX);
    let total = u16::try_from(field.chars().count() + tail.chars().count()).unwrap_or(u16::MAX);
    let x = area.x + area.width.saturating_sub(total) / 2;
    let muted = ctx.theme.style(Token::TextMuted, Token::Background);
    let line = Line::from(vec![
        Span::styled(field, button_style(ctx, text.is_some())),
        Span::styled(tail, muted),
    ]);
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
    Rect::new(x, area.y, width, 1).intersection(area)
}

/// Draws the legend of the two marks.
pub(super) fn draw_legend(frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>) {
    let key = primary_key(Action::Finish).unwrap_or_default();
    let text = tf(
        "menu.legend",
        &[
            ("star", ctx.glyphs.star),
            ("favourite", ctx.glyphs.favourite),
            ("key", key.as_str()),
        ],
    );
    let muted = ctx.theme.style(Token::TextMuted, Token::Background);
    frame.render_widget(
        Paragraph::new(Line::styled(text, muted)).alignment(Alignment::Center),
        area,
    );
}

/// Draws the message that no game matches `text`.
pub(super) fn draw_no_match(frame: &mut Frame, area: Rect, text: &str, ctx: &ViewCtx<'_>) {
    let message = tf("menu.no_match", &[("text", text)]);
    let muted = ctx.theme.style(Token::TextMuted, Token::Background);
    frame.render_widget(
        Paragraph::new(Line::styled(message, muted)).alignment(Alignment::Center),
        area,
    );
}
