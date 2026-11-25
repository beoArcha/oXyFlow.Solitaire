//! The two toggles under the variants list and the hints that explain them.

use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use super::WINNABLE_KEY;
use crate::action::Action;
use crate::i18n::{t, tf};
use crate::input::{KeySpec, primary_key};
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
    let key = KeySpec::ch(WINNABLE_KEY).display();
    let label = tf("menu.winnable.button", &[("key", &key), ("state", &state)]);
    draw_toggle(frame, area, label, focused, ctx)
}

fn winnable_hint(on: bool) -> String {
    if on {
        t("menu.winnable.on_hint").into_owned()
    } else {
        t("menu.winnable.off_hint").into_owned()
    }
}

pub(super) fn draw_hints(
    frame: &mut Frame,
    undo_area: Rect,
    winnable_area: Option<Rect>,
    ctx: &ViewCtx<'_>,
) {
    let key = primary_key(Action::Undo).unwrap_or_default();
    let muted = ctx.theme.style(Token::TextMuted, Token::Background);
    let undo = tf("common.select.undo_hint", &[("key", &key)]);
    draw_centered(frame, undo_area, undo, muted);
    if let Some(area) = winnable_area {
        draw_centered(
            frame,
            area,
            winnable_hint(ctx.settings.winnable_only),
            muted,
        );
    }
}

fn draw_centered(frame: &mut Frame, area: Rect, text: String, style: Style) {
    frame.render_widget(
        Paragraph::new(Line::styled(text, style)).alignment(Alignment::Center),
        area,
    );
}
