//! The controls strip under the board: two rows as wide as the board. `Now:` says what is focused
//! or carried, with the last event just left of its right end in the Success or Error token;
//! `Do:` lists the actions of this exact state with their keys, each a mouse button. `Now:` holds
//! no buttons: when `Do:` is narrow, its hints with the least priority are dropped (see
//! `priority.rs`), never moved to the `Now:` row.

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};

use super::guidance::Guidance;
use super::status::Tone;
use crate::action::Action;
use crate::screens::ViewCtx;
use crate::theme::Token;
use crate::widgets::footer::{self, FooterMap};
use crate::widgets::text::clip;

/// Longest event text, in columns; shorter events never push `Now:` out of its row.
const EVENT_WIDTH: usize = 40;

/// The buttons of the controls strip as drawn last: only the `Do:` row has any.
#[derive(Debug, Default)]
pub(super) struct Buttons {
    pub do_row: FooterMap,
}

impl Buttons {
    /// The action of the button under `cell`, if any.
    pub(super) fn action_at(&self, cell: (u16, u16)) -> Option<Action> {
        self.do_row.action_at(cell)
    }
}

fn event_token(tone: Tone) -> Token {
    match tone {
        Tone::Success => Token::Success,
        Tone::Error => Token::Error,
    }
}

fn draw_now(
    frame: &mut Frame,
    area: Rect,
    guidance: &Guidance,
    event: Option<&(String, Tone)>,
    ctx: &ViewCtx<'_>,
) {
    let width = usize::from(area.width);
    let shown = event.map(|(text, tone)| (clip(text, EVENT_WIDTH.min(width / 2)), *tone));
    let event_width = shown
        .as_ref()
        .map_or(0, |(text, _)| text.chars().count() + 1);
    let now = guidance.now_line(width.saturating_sub(event_width));
    frame.render_widget(
        Paragraph::new(Line::styled(
            now,
            ctx.theme.style(Token::Text, Token::Surface),
        )),
        area,
    );
    if let Some((text, tone)) = shown {
        let style = ctx.theme.style(event_token(tone), Token::Surface);
        frame.render_widget(
            Paragraph::new(Line::styled(text, style)).alignment(Alignment::Right),
            area,
        );
    }
}

/// Draws the `Do:` row as buttons and returns them. The entries that fit are the ones the row
/// shows; the rest are dropped by [`Guidance::do_segments`].
fn draw_do(buf: &mut Buffer, area: Rect, guidance: &Guidance, ctx: &ViewCtx<'_>) -> FooterMap {
    let segments = guidance.do_segments(usize::from(area.width));
    let surface = ctx.theme.style(Token::Text, Token::Surface);
    footer::draw_to(buf, area, &segments, Alignment::Left, surface, ctx)
}

/// Draws the two rows and returns their buttons. Without guidance (no game, or the board is
/// covered) they stay empty.
pub(super) fn draw(
    frame: &mut Frame,
    now: Rect,
    possible: Rect,
    guidance: Option<&Guidance>,
    event: Option<&(String, Tone)>,
    ctx: &ViewCtx<'_>,
) -> Buttons {
    let surface = ctx.theme.style(Token::Text, Token::Surface);
    frame.render_widget(Block::new().style(surface), now);
    frame.render_widget(Block::new().style(surface), possible);
    let Some(guidance) = guidance else {
        return Buttons::default();
    };
    draw_now(frame, now, guidance, event, ctx);
    let do_row = draw_do(frame.buffer_mut(), possible, guidance, ctx);
    Buttons { do_row }
}
