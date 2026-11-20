//! The controls strip under the board: two rows as wide as the board. `Now:` says what is focused
//! or carried, with the last event just left of its right end in the Success or Error token;
//! `Do:` lists the actions of this exact state with their keys. Each entry of both rows is a mouse
//! button. When `Do:` has no room for Pause or Possible moves, `Now:` ends with them, so they stay
//! clickable at every width.

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
use crate::widgets::footer::{self, FooterMap, Segment};
use crate::widgets::text::clip;

/// Longest event text, in columns; shorter events never push `Now:` out of its row.
const EVENT_WIDTH: usize = 40;

/// Columns between the last event and the fallback buttons of `Now:`.
const GAP: u16 = 3;

/// The buttons of the two rows as drawn last.
#[derive(Debug, Default)]
pub(super) struct Buttons {
    pub do_row: FooterMap,
    pub now_row: FooterMap,
}

impl Buttons {
    /// The action of the button under `cell`, if any.
    pub(super) fn action_at(&self, cell: (u16, u16)) -> Option<Action> {
        self.do_row
            .action_at(cell)
            .or_else(|| self.now_row.action_at(cell))
    }
}

fn event_token(tone: Tone) -> Token {
    match tone {
        Tone::Success => Token::Success,
        Tone::Error => Token::Error,
    }
}

/// The fallback buttons for `Now:`: the Pause and Moves parts that the `Do:` row of `width`
/// columns has no room for, separated by three spaces.
fn fallback_segments(guidance: &Guidance, width: usize) -> Vec<Segment> {
    let mut segments = Vec::new();
    for (index, part) in guidance.fallback(width).iter().enumerate() {
        if index > 0 {
            segments.push(Segment::plain("   "));
        }
        segments.push(match part.action {
            Some(action) => Segment::button(action, part.text.as_str()),
            None => Segment::plain(part.text.as_str()),
        });
    }
    segments
}

fn segments_width(segments: &[Segment]) -> u16 {
    let columns: usize = segments
        .iter()
        .map(|segment| segment.text.chars().count())
        .sum();
    u16::try_from(columns).unwrap_or(u16::MAX)
}

fn draw_now(
    frame: &mut Frame,
    area: Rect,
    guidance: &Guidance,
    event: Option<&(String, Tone)>,
    fallback: &[Segment],
    ctx: &ViewCtx<'_>,
) -> FooterMap {
    let reserved = segments_width(fallback).min(area.width.saturating_sub(GAP));
    let gap = if reserved == 0 { 0 } else { GAP };
    let text_width = area.width - reserved - gap;
    let text_area = Rect::new(area.x, area.y, text_width, area.height);
    let width = usize::from(text_width);
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
        text_area,
    );
    if let Some((text, tone)) = shown {
        let style = ctx.theme.style(event_token(tone), Token::Surface);
        frame.render_widget(
            Paragraph::new(Line::styled(text, style)).alignment(Alignment::Right),
            text_area,
        );
    }
    if reserved == 0 {
        return FooterMap::default();
    }
    let buttons = Rect::new(area.x + text_width + gap, area.y, reserved, area.height);
    let surface = ctx.theme.style(Token::Text, Token::Surface);
    footer::draw_to(
        frame.buffer_mut(),
        buttons,
        fallback,
        Alignment::Left,
        surface,
        ctx,
    )
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
    let fallback = fallback_segments(guidance, usize::from(possible.width));
    let now_row = draw_now(frame, now, guidance, event, &fallback, ctx);
    let do_row = draw_do(frame.buffer_mut(), possible, guidance, ctx);
    Buttons { do_row, now_row }
}
