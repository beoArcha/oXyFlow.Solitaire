//! The controls strip under the board: two rows as wide as the board. `Now:` says what is focused
//! or carried, with the last event at its right end in the Success or Error token; `Do:` lists
//! the actions of this exact state with their keys.

use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};

use super::guidance::Guidance;
use super::status::Tone;
use crate::screens::ViewCtx;
use crate::theme::Token;
use crate::widgets::text::clip;

/// Longest event text, in columns; shorter events never push `Now:` out of its row.
const EVENT_WIDTH: usize = 40;

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

/// Draws the two rows. Without guidance (no game, or the board is covered) they stay empty.
pub(super) fn draw(
    frame: &mut Frame,
    now: Rect,
    possible: Rect,
    guidance: Option<&Guidance>,
    event: Option<&(String, Tone)>,
    ctx: &ViewCtx<'_>,
) {
    let surface = ctx.theme.style(Token::Text, Token::Surface);
    frame.render_widget(Block::new().style(surface), now);
    frame.render_widget(Block::new().style(surface), possible);
    let Some(guidance) = guidance else {
        return;
    };
    draw_now(frame, now, guidance, event, ctx);
    frame.render_widget(
        Paragraph::new(Line::styled(
            guidance.possible_line(usize::from(possible.width)),
            surface,
        )),
        possible,
    );
}
