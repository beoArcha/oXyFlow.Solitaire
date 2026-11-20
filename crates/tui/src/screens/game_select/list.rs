//! The scrolling list of the game menu: headings and variants, one row each. Only variants take
//! the focus. The focused row has a marker before it and reverse video; a variant's marks (the
//! favourite sign and the popularity star) are drawn before its name, and a click on them toggles
//! the favourite. The list scrolls to keep the focused variant and its heading in view.

use oxysol_core::Variant;
use ratatui::Frame;
use ratatui::crossterm::event::MouseEvent;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use super::catalog::is_popular;
use super::rows::{Heading, Row, items};
use crate::action::Action;
use crate::i18n::t;
use crate::mouse::{contains, left_press, wheel_action};
use crate::screens::ViewCtx;
use crate::screens::focus::{marker, step, tab_forward};
use crate::theme::Token;

const MARKS_WIDTH: usize = 6;
const MARKS_OFFSET: u16 = 2;
const MARKS_CELLS: u16 = 3;

/// A click on a variant: the mark cells toggle the favourite, the rest starts the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Click {
    pub index: usize,
    pub on_mark: bool,
}

#[derive(Debug, Clone, Copy)]
struct Hit {
    row: Rect,
    mark: Rect,
    index: usize,
}

/// What the list shows: the rows, the favourites and the name of each variant.
pub(super) struct Content<'a> {
    pub rows: &'a [Row],
    pub favourites: &'a [Variant],
    pub label_of: &'a dyn Fn(Variant) -> String,
}

/// The focus, the scroll position and the rectangles of the rows drawn last.
#[derive(Debug, Default)]
pub(super) struct ListView {
    focus: usize,
    scroll: usize,
    hits: Vec<Hit>,
}

fn forward_of(action: Action) -> Option<bool> {
    match action {
        Action::Down => Some(true),
        Action::Up => Some(false),
        other => tab_forward(other),
    }
}

fn block_width(content: &Content<'_>) -> u16 {
    let widest = content
        .rows
        .iter()
        .map(|row| match row {
            Row::Heading(heading) => 2 + t(heading.title_key()).chars().count(),
            Row::Item(variant) => MARKS_WIDTH + (content.label_of)(*variant).chars().count(),
        })
        .max()
        .unwrap_or(0);
    u16::try_from(widest + 2).unwrap_or(u16::MAX)
}

fn draw_heading(frame: &mut Frame, line: Rect, heading: Heading, ctx: &ViewCtx<'_>) {
    let style = ctx
        .theme
        .style(Token::Accent, Token::Background)
        .add_modifier(Modifier::BOLD);
    let text = format!("  {}", t(heading.title_key()));
    frame.render_widget(Paragraph::new(Line::styled(text, style)), line);
}

fn draw_item(
    frame: &mut Frame,
    line: Rect,
    variant: Variant,
    focused: bool,
    content: &Content<'_>,
    ctx: &ViewCtx<'_>,
) {
    let style = if focused {
        ctx.theme.focus()
    } else {
        ctx.theme.base()
    };
    let favourite = if content.favourites.contains(&variant) {
        ctx.glyphs.favourite
    } else {
        ctx.glyphs.not_favourite
    };
    let star = if is_popular(variant) {
        ctx.glyphs.star
    } else {
        " "
    };
    let text = format!(
        "{} {favourite} {star} {}",
        marker(ctx, focused),
        (content.label_of)(variant)
    );
    frame.render_widget(Paragraph::new(Line::styled(text, style)), line);
}

impl ListView {
    pub(super) fn focus(&self) -> usize {
        self.focus
    }

    pub(super) fn set_focus(&mut self, index: usize) {
        self.focus = index;
    }

    pub(super) fn reset(&mut self) {
        self.focus = 0;
        self.scroll = 0;
    }

    /// Moves the focus (wrapping). Returns the index when the focused variant is activated.
    pub(super) fn on_action(&mut self, action: Action, len: usize) -> Option<usize> {
        if len == 0 {
            return None;
        }
        if let Some(forward) = forward_of(action) {
            self.focus = step(self.focus.min(len - 1), len, forward);
        }
        (action == Action::Select).then_some(self.focus.min(len - 1))
    }

    /// Handles the wheel and left clicks; a click moves the focus to the clicked variant.
    pub(super) fn on_mouse(&mut self, event: &MouseEvent, len: usize) -> Option<Click> {
        if let Some(action) = wheel_action(event) {
            self.on_action(action, len);
            return None;
        }
        let cell = left_press(event)?;
        let hit = self.hits.iter().find(|hit| contains(hit.row, cell))?;
        self.focus = hit.index;
        Some(Click {
            index: hit.index,
            on_mark: contains(hit.mark, cell),
        })
    }

    fn keep_in_view(&mut self, rows: &[Row], height: usize) {
        let position = rows
            .iter()
            .enumerate()
            .filter(|(_, row)| matches!(row, Row::Item(_)))
            .nth(self.focus)
            .map_or(0, |(position, _)| position);
        let top = if position > 0 && matches!(rows.get(position - 1), Some(Row::Heading(_))) {
            position - 1
        } else {
            position
        };
        if top < self.scroll {
            self.scroll = top;
        }
        if position >= self.scroll + height {
            self.scroll = position + 1 - height;
        }
        self.scroll = self.scroll.min(rows.len().saturating_sub(height));
    }

    /// Draws the visible rows in `area` and remembers where they are. The focused variant is
    /// marked and highlighted only while `active`.
    pub(super) fn draw(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        content: &Content<'_>,
        active: bool,
        ctx: &ViewCtx<'_>,
    ) {
        self.hits.clear();
        let rows = content.rows;
        self.focus = self.focus.min(items(rows).len().saturating_sub(1));
        let height = usize::from(area.height).max(1);
        self.keep_in_view(rows, height);
        let width = block_width(content).min(area.width);
        let x = area.x + (area.width - width) / 2;
        let mut index = rows
            .iter()
            .take(self.scroll)
            .filter(|row| matches!(row, Row::Item(_)))
            .count();
        for (offset, row) in rows.iter().skip(self.scroll).take(height).enumerate() {
            let Ok(offset) = u16::try_from(offset) else {
                break;
            };
            let line = Rect::new(x, area.y + offset, width, 1);
            match row {
                Row::Heading(heading) => draw_heading(frame, line, *heading, ctx),
                Row::Item(variant) => {
                    let focused = active && index == self.focus;
                    draw_item(frame, line, *variant, focused, content, ctx);
                    self.hits.push(Hit {
                        row: line,
                        mark: Rect::new(x + MARKS_OFFSET, line.y, MARKS_CELLS, 1)
                            .intersection(line),
                        index,
                    });
                    index += 1;
                }
            }
        }
        self.draw_edges(frame, area, rows.len(), x + width.saturating_sub(1), ctx);
    }

    fn draw_edges(&self, frame: &mut Frame, area: Rect, total: usize, x: u16, ctx: &ViewCtx<'_>) {
        let (up, down) = if ctx.glyphs.unicode {
            ("↑", "↓")
        } else {
            ("^", "v")
        };
        let muted = ctx.theme.style(Token::TextMuted, Token::Background);
        if self.scroll > 0 {
            let cell = Rect::new(x, area.y, 1, 1);
            frame.render_widget(Paragraph::new(Line::styled(up, muted)), cell);
        }
        if self.scroll + usize::from(area.height) < total && area.height > 0 {
            let cell = Rect::new(x, area.y + area.height - 1, 1, 1);
            frame.render_widget(Paragraph::new(Line::styled(down, muted)), cell);
        }
    }
}

#[cfg(test)]
mod tests;
