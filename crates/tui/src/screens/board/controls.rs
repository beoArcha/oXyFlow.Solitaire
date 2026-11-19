//! The controls strip under the board: two rows as wide as the board. `Now:` says what is focused
//! or carried, with the last event at its right end in the Success or Error token; `Do:` lists
//! the actions of this exact state with their keys. Each entry in the `Do:` row is a mouse button.

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Widget};

use super::guidance::{Guidance, Part};
use super::status::Tone;
use crate::action::Action;
use crate::i18n::{t, tf};
use crate::screens::ViewCtx;
use crate::theme::Token;
use crate::widgets::text::clip;

/// Longest event text, in columns; shorter events never push `Now:` out of its row.
const EVENT_WIDTH: usize = 40;

/// One clickable entry in the `Do:` row, with its column range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct DoEntry {
    pub action: Action,
    pub col_start: u16,
    pub col_end: u16,
}

/// The click/focus map for the `Do:` row, recorded after each draw.
#[derive(Debug, Default, Clone)]
pub(super) struct DoMap {
    pub row: u16,
    pub entries: Vec<DoEntry>,
}

impl DoMap {
    /// The action for a click at `(column, row)`, or `None` when it misses.
    pub fn action_at(&self, column: u16, row: u16) -> Option<Action> {
        if row != self.row {
            return None;
        }
        self.entries
            .iter()
            .find(|e| column >= e.col_start && column < e.col_end)
            .map(|e| e.action)
    }

    /// The index of the entry at `column`, or `None`.
    pub fn index_at(&self, column: u16, row: u16) -> Option<usize> {
        if row != self.row {
            return None;
        }
        self.entries
            .iter()
            .position(|e| column >= e.col_start && column < e.col_end)
    }

    /// Number of clickable entries.
    pub fn len(&self) -> usize {
        self.entries.len()
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

/// Renders the `Do:` row as a sequence of clickable spans with the separator between them.
/// Returns the hit map. The entry at `focused` is shown in reverse video.
fn draw_do(
    buf: &mut Buffer,
    area: Rect,
    guidance: &Guidance,
    focused: Option<usize>,
    ctx: &ViewCtx<'_>,
) -> DoMap {
    let width = usize::from(area.width);
    let parts: Vec<&Part> = guidance.visible_parts(width);

    let prefix = t("board.action.possible");
    let prefix_text = prefix.trim_end_matches("{text}");

    let sep = t("board.action.separator");
    let sep_str = sep.as_ref();

    let surface = ctx.theme.style(Token::Text, Token::Surface);
    let highlighted = surface.add_modifier(Modifier::REVERSED);

    let mut map = DoMap {
        row: area.y,
        entries: Vec::with_capacity(parts.len()),
    };

    let mut x = area.x;

    let prefix_len = prefix_text.chars().count() as u16;
    if prefix_len > 0 && x + prefix_len <= area.right() {
        Span::styled(prefix_text, surface).render(Rect::new(x, area.y, prefix_len, 1), buf);
        x += prefix_len;
    }

    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            let sep_len = sep_str.chars().count() as u16;
            let avail = area.right().saturating_sub(x);
            let w = sep_len.min(avail);
            if w > 0 {
                Span::styled(sep_str, surface).render(Rect::new(x, area.y, w, 1), buf);
                x += w;
            }
        }
        let avail = area.right().saturating_sub(x);
        if avail == 0 {
            break;
        }
        let entry_len = part.text.chars().count() as u16;
        let w = entry_len.min(avail);
        let style = match (part.action, focused) {
            (Some(_), Some(fi)) if fi == map.entries.len() => highlighted,
            _ => surface,
        };
        Span::styled(part.text.as_str(), style).render(Rect::new(x, area.y, w, 1), buf);
        if let Some(action) = part.action {
            map.entries.push(DoEntry {
                action,
                col_start: x,
                col_end: x + w,
            });
        }
        x += w;
    }

    map
}

/// Draws the two rows. Without guidance (no game, or the board is covered) they stay empty.
/// Returns the click map for the `Do:` row; `focused` is the Tab-focused entry index.
pub(super) fn draw(
    frame: &mut Frame,
    now: Rect,
    possible: Rect,
    guidance: Option<&Guidance>,
    event: Option<&(String, Tone)>,
    focused: Option<usize>,
    ctx: &ViewCtx<'_>,
) -> DoMap {
    let surface = ctx.theme.style(Token::Text, Token::Surface);
    frame.render_widget(Block::new().style(surface), now);
    frame.render_widget(Block::new().style(surface), possible);
    let Some(guidance) = guidance else {
        return DoMap::default();
    };
    draw_now(frame, now, guidance, event, ctx);
    draw_do(frame.buffer_mut(), possible, guidance, focused, ctx)
}
