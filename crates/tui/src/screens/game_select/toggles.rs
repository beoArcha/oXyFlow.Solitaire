//! The two toggles under the variants list and the hints that explain them.

use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use crate::action::Action;
use crate::i18n::{t, tf};
use crate::input::primary_key;
use crate::screens::ViewCtx;
use crate::screens::focus::{button_style, marker};
use crate::theme::Token;

fn state_text(on: bool) -> String {
    if on {
        t("common.select.on").into_owned()
    } else {
        t("common.select.off").into_owned()
    }
}

/// Draws one toggle centered in `area` and returns the cells that answer a click.
fn draw_toggle(
    frame: &mut Frame,
    area: Rect,
    text: String,
    focused: bool,
    ctx: &ViewCtx<'_>,
) -> Rect {
    let text = format!("{} {text}", marker(ctx, focused));
    let width = u16::try_from(text.chars().count()).unwrap_or(u16::MAX);
    let x = area.x + area.width.saturating_sub(width) / 2;
    frame.render_widget(
        Paragraph::new(Line::styled(text, button_style(ctx, focused))).alignment(Alignment::Center),
        area,
    );
    Rect::new(x, area.y, width, 1).intersection(area)
}

pub(super) fn draw_undo(frame: &mut Frame, area: Rect, focused: bool, ctx: &ViewCtx<'_>) -> Rect {
    let state = state_text(ctx.settings.undo_allowed);
    let label = tf("common.select.undo", &[("state", &state)]);
    draw_toggle(frame, area, label, focused, ctx)
}

pub(super) fn draw_winnable(
    frame: &mut Frame,
    area: Rect,
    focused: bool,
    ctx: &ViewCtx<'_>,
) -> Rect {
    let state = state_text(ctx.settings.winnable_only);
    let label = tf("common.select.winnable", &[("state", &state)]);
    draw_toggle(frame, area, label, focused, ctx)
}

pub(super) fn draw_hints(frame: &mut Frame, areas: [Rect; 2], ctx: &ViewCtx<'_>) {
    let key = primary_key(Action::Undo).unwrap_or_default();
    let hints = [
        tf("common.select.undo_hint", &[("key", &key)]),
        t("common.select.winnable_hint").into_owned(),
    ];
    let muted = ctx.theme.style(Token::TextMuted, Token::Background);
    for (hint, area) in hints.into_iter().zip(areas) {
        frame.render_widget(
            Paragraph::new(Line::styled(hint, muted)).alignment(Alignment::Center),
            area,
        );
    }
}
