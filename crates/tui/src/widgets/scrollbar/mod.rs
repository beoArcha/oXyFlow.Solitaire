//! A vertical scroll bar for the scrolling modals: a track one column wide with a thumb whose
//! length is the visible share of the text and whose place is the scroll position. The thumb and
//! the track are different symbols (WGL4 blocks, `#` and `:` in ASCII), so the bar reads without
//! color. The mouse pages by a click on the track and scrolls by dragging the thumb.

use ratatui::Frame;
use ratatui::crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

use crate::mouse::contains;
use crate::screens::ViewCtx;
use crate::theme::Token;

/// How much text there is, how much of it shows and where the first shown line is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Metrics {
    pub total: usize,
    pub view: usize,
    pub offset: usize,
}

impl Metrics {
    /// Whether the text is longer than the view, so that there is anything to scroll.
    pub const fn scrolls(&self) -> bool {
        self.total > self.view
    }

    fn max_offset(&self) -> usize {
        self.total.saturating_sub(self.view)
    }

    fn page(&self) -> usize {
        self.view.saturating_sub(1).max(1)
    }

    /// The first row and the length of the thumb on a track of `len` rows; `None` when the whole
    /// text shows or the track has no rows.
    pub fn thumb(&self, len: u16) -> Option<(u16, u16)> {
        if !self.scrolls() || len == 0 {
            return None;
        }
        let rows = usize::from(len);
        let size = rounded_div(rows * self.view, self.total).clamp(1, rows);
        let start = rounded_div(
            (rows - size) * self.offset.min(self.max_offset()),
            self.max_offset(),
        );
        Some((
            u16::try_from(start).unwrap_or(0),
            u16::try_from(size).unwrap_or(len),
        ))
    }

    fn offset_for_thumb_start(&self, len: u16, size: u16, start: u16) -> usize {
        let travel = usize::from(len.saturating_sub(size));
        if travel == 0 {
            return 0;
        }
        rounded_div(
            usize::from(start.min(len - size)) * self.max_offset(),
            travel,
        )
    }
}

fn rounded_div(numerator: usize, denominator: usize) -> usize {
    if denominator == 0 {
        0
    } else {
        (numerator * 2 + denominator) / (denominator * 2)
    }
}

/// The track of a text area: its last column. The dialogs keep that column free as a margin.
pub fn track_of(text: Rect) -> Rect {
    Rect::new(
        text.right().saturating_sub(1).max(text.x),
        text.y,
        u16::from(text.width > 0),
        text.height,
    )
}

/// Draws the bar on `track`; nothing is drawn when the whole text shows.
pub fn draw(frame: &mut Frame, track: Rect, metrics: Metrics, ctx: &ViewCtx<'_>) {
    let Some((start, size)) = metrics.thumb(track.height) else {
        return;
    };
    let groove = ctx.theme.style(Token::TextMuted, Token::Surface);
    let thumb = ctx.theme.style(Token::Accent, Token::Surface);
    let buffer = frame.buffer_mut();
    for row in 0..track.height {
        let on_thumb = row >= start && row < start + size;
        let (symbol, style) = if on_thumb {
            (ctx.glyphs.scroll_thumb, thumb)
        } else {
            (ctx.glyphs.scroll_track, groove)
        };
        buffer.set_string(track.x, track.y + row, symbol, style);
    }
}

/// Handles a mouse event on the bar. A click on the track above or below the thumb pages up or
/// down, a press on the thumb starts a drag that `drag` remembers (the row grabbed inside the
/// thumb), a drag moves the thumb, a release ends it. The result is the new first line, if the
/// event scrolls; the caller keeps it inside the text.
pub fn handle_mouse(
    event: &MouseEvent,
    track: Rect,
    metrics: Metrics,
    drag: &mut Option<u16>,
) -> Option<usize> {
    let (start, size) = metrics.thumb(track.height)?;
    let row_in_track = event.row.saturating_sub(track.y);
    match event.kind {
        MouseEventKind::Down(MouseButton::Left) if contains(track, (event.column, event.row)) => {
            if row_in_track < start {
                Some(metrics.offset.saturating_sub(metrics.page()))
            } else if row_in_track >= start + size {
                Some((metrics.offset + metrics.page()).min(metrics.max_offset()))
            } else {
                *drag = Some(row_in_track - start);
                None
            }
        }
        MouseEventKind::Drag(MouseButton::Left) => {
            let grab = (*drag)?;
            let target = row_in_track.saturating_sub(grab);
            Some(metrics.offset_for_thumb_start(track.height, size, target))
        }
        MouseEventKind::Up(MouseButton::Left) => {
            *drag = None;
            None
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests;
