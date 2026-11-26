//! The scrolling list of the game menu: headings and variants, one row each. Both headings and
//! variants take the focus. The focused row has a marker before it and reverse video; a variant's
//! marks (the favourite sign and the popularity star) are drawn before its name, and a click on
//! them toggles the favourite. Wheel and edge-mark clicks scroll without moving focus off a
//! visible row; keyboard navigation keeps the focused row and its heading in view.

use oxysol_core::Variant;
use ratatui::Frame;
use ratatui::crossterm::event::MouseEvent;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use super::catalog::is_popular;
use super::rows::{FocusItem, Heading, Row, default_position, heading_text, resolve_position};
use crate::action::Action;
use crate::mouse::{contains, left_press, wheel_action};
use crate::screens::ViewCtx;
use crate::screens::focus::marker;
use crate::theme::Token;

const MARKS_WIDTH: usize = 6;
const MARKS_OFFSET: u16 = 2;
const MARKS_CELLS: u16 = 3;
const WHEEL_ROWS: isize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MouseOutcome {
    Handled,
    Fold(Heading),
    ToggleFavourite(Variant),
    Start(Variant),
}

#[derive(Debug, Clone, Copy)]
enum HitTarget {
    Heading(Heading),
    Item(Variant),
}

#[derive(Debug, Clone, Copy)]
struct Hit {
    row: Rect,
    mark: Rect,
    target: HitTarget,
}

/// What the list shows: the rows, the favourites and the name of each variant.
pub(super) struct Content<'a> {
    pub rows: &'a [Row],
    pub favourites: &'a [Variant],
    pub label_of: &'a dyn Fn(Variant) -> String,
}

/// The focus, the scroll position and the rectangles of the rows and edge marks drawn last.
#[derive(Debug, Default)]
pub(super) struct ListView {
    focused: Option<FocusItem>,
    scroll: usize,
    area: Rect,
    view_height: usize,
    up_mark: Option<Rect>,
    down_mark: Option<Rect>,
    hits: Vec<Hit>,
}

fn block_width(content: &Content<'_>) -> u16 {
    let widest = content
        .rows
        .iter()
        .map(|row| match row {
            Row::Heading {
                heading,
                folded,
                count,
            } => 2 + heading_text(*heading, *folded, *count).chars().count(),
            Row::Item(variant) => MARKS_WIDTH + (content.label_of)(*variant).chars().count(),
            Row::Gap => 0,
        })
        .max()
        .unwrap_or(0);
    u16::try_from(widest + 2).unwrap_or(u16::MAX)
}

fn draw_heading(
    frame: &mut Frame,
    line: Rect,
    heading: Heading,
    folded: bool,
    count: usize,
    focused: bool,
    ctx: &ViewCtx<'_>,
) {
    let style = if focused {
        ctx.theme.focus()
    } else {
        ctx.theme
            .style(Token::Accent, Token::Background)
            .add_modifier(Modifier::BOLD)
    };
    let text = format!(
        "{} {}",
        marker(ctx, focused),
        heading_text(heading, folded, count)
    );
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

fn step_focus(rows: &[Row], pos: usize, forward: bool) -> Option<usize> {
    if forward {
        (pos + 1..rows.len())
            .find(|&i| rows[i].is_focusable())
            .or_else(|| (0..=pos).find(|&i| rows[i].is_focusable()))
    } else {
        (0..pos)
            .rfind(|&i| rows[i].is_focusable())
            .or_else(|| (pos..rows.len()).rfind(|&i| rows[i].is_focusable()))
    }
}

fn page_target(rows: &[Row], pos: usize, page: usize, up: bool) -> Option<usize> {
    if rows.is_empty() {
        return None;
    }
    let raw = if up {
        pos.saturating_sub(page)
    } else {
        (pos + page).min(rows.len() - 1)
    };
    if rows[raw].is_focusable() {
        return Some(raw);
    }
    if up {
        rows[..raw]
            .iter()
            .rposition(|row| row.is_focusable())
            .or_else(|| {
                rows[raw..]
                    .iter()
                    .position(|row| row.is_focusable())
                    .map(|d| raw + d)
            })
    } else {
        rows[raw..]
            .iter()
            .position(|row| row.is_focusable())
            .map(|d| raw + d)
            .or_else(|| rows[..raw].iter().rposition(|row| row.is_focusable()))
    }
}

fn nearest_visible_target(
    rows: &[Row],
    scroll: usize,
    height: usize,
    current: usize,
) -> Option<usize> {
    let end = (scroll + height).min(rows.len());
    let visible = scroll..end;
    if visible.contains(&current) && rows.get(current).is_some_and(|r| r.is_focusable()) {
        return None;
    }
    visible
        .clone()
        .filter(|&i| matches!(rows[i], Row::Item(_)))
        .min_by_key(|&i| i.abs_diff(current))
        .or_else(|| {
            visible
                .filter(|&i| rows[i].is_focusable())
                .min_by_key(|&i| i.abs_diff(current))
        })
}

impl ListView {
    fn page_size(&self) -> usize {
        self.view_height.max(1)
    }

    pub(super) fn focused_row(&self, rows: &[Row], favourites: &[Variant]) -> Option<usize> {
        resolve_position(rows, self.focused, favourites)
    }

    pub(super) fn focused_item(&self, rows: &[Row], favourites: &[Variant]) -> Option<FocusItem> {
        self.focused_row(rows, favourites)
            .and_then(|pos| rows.get(pos).and_then(|row| row.focus_item()))
    }

    pub(super) fn set_focus_item(&mut self, item: FocusItem) {
        self.focused = Some(item);
    }

    pub(super) fn reset(&mut self) {
        self.focused = None;
        self.scroll = 0;
    }

    pub(super) fn focus_first(&mut self, rows: &[Row]) {
        if let Some(pos) = rows.iter().position(|row| row.is_focusable()) {
            self.focused = rows[pos].focus_item();
            self.keep_position_in_view(rows, pos, self.view_height);
        }
    }

    pub(super) fn focus_last(&mut self, rows: &[Row]) {
        if let Some(pos) = rows.iter().rposition(|row| row.is_focusable()) {
            self.focused = rows[pos].focus_item();
            self.keep_position_in_view(rows, pos, self.view_height);
        }
    }

    pub(super) fn step_action(&mut self, action: Action, rows: &[Row], favourites: &[Variant]) {
        let forward = match action {
            Action::Down => true,
            Action::Up => false,
            _ => return,
        };
        let Some(pos) = self.focused_row(rows, favourites) else {
            return;
        };
        if let Some(next) = step_focus(rows, pos, forward) {
            self.focused = rows[next].focus_item();
            self.keep_position_in_view(rows, next, self.view_height);
        }
    }

    pub(super) fn page_by(&mut self, up: bool, rows: &[Row], favourites: &[Variant]) {
        let Some(pos) = self.focused_row(rows, favourites) else {
            return;
        };
        let page = self.page_size();
        if let Some(next) = page_target(rows, pos, page, up) {
            self.focused = rows[next].focus_item();
            self.keep_position_in_view(rows, next, self.view_height);
        }
    }

    pub(super) fn home(&mut self, rows: &[Row]) {
        if let Some(pos) = default_position(rows) {
            self.focused = rows[pos].focus_item();
            self.keep_position_in_view(rows, pos, self.view_height);
        }
    }

    pub(super) fn end(&mut self, rows: &[Row]) {
        let target = rows
            .iter()
            .rposition(|row| matches!(row, Row::Item(_)))
            .or_else(|| rows.iter().rposition(|row| row.is_focusable()));
        if let Some(pos) = target {
            self.focused = rows[pos].focus_item();
            self.keep_position_in_view(rows, pos, self.view_height);
        }
    }

    pub(super) fn scroll_by(&mut self, delta: isize, rows: &[Row], favourites: &[Variant]) {
        let height = self.page_size();
        let max_scroll = rows.len().saturating_sub(height);
        self.scroll = self.scroll.saturating_add_signed(delta).min(max_scroll);
        if let Some(pos) = self.focused_row(rows, favourites)
            && let Some(next) = nearest_visible_target(rows, self.scroll, height, pos)
        {
            self.focused = rows[next].focus_item();
        }
    }

    pub(super) fn keep_resolved_in_view(&mut self, rows: &[Row], favourites: &[Variant]) {
        if let Some(pos) = self.focused_row(rows, favourites) {
            self.keep_position_in_view(rows, pos, self.view_height);
        } else {
            self.scroll = 0;
        }
    }

    /// Handles the wheel over the list, edge-mark clicks, and row clicks.
    pub(super) fn on_mouse(
        &mut self,
        event: &MouseEvent,
        rows: &[Row],
        favourites: &[Variant],
    ) -> Option<MouseOutcome> {
        if let Some(action) = wheel_action(event) {
            if !contains(self.area, (event.column, event.row)) {
                return None;
            }
            let delta = if action == Action::Up {
                -WHEEL_ROWS
            } else {
                WHEEL_ROWS
            };
            self.scroll_by(delta, rows, favourites);
            return Some(MouseOutcome::Handled);
        }
        let cell = left_press(event)?;
        let page = isize::try_from(self.page_size()).unwrap_or(isize::MAX);
        if self.up_mark.is_some_and(|mark| contains(mark, cell)) {
            self.scroll_by(-page, rows, favourites);
            return Some(MouseOutcome::Handled);
        }
        if self.down_mark.is_some_and(|mark| contains(mark, cell)) {
            self.scroll_by(page, rows, favourites);
            return Some(MouseOutcome::Handled);
        }
        let hit = self.hits.iter().find(|hit| contains(hit.row, cell))?;
        match hit.target {
            HitTarget::Heading(heading) => {
                self.focused = Some(FocusItem::Heading(heading));
                Some(MouseOutcome::Fold(heading))
            }
            HitTarget::Item(variant) => {
                self.focused = Some(FocusItem::Variant(variant));
                if contains(hit.mark, cell) {
                    Some(MouseOutcome::ToggleFavourite(variant))
                } else {
                    Some(MouseOutcome::Start(variant))
                }
            }
        }
    }

    fn keep_position_in_view(&mut self, rows: &[Row], position: usize, height: usize) {
        if height == 0 {
            return;
        }
        let top = if position > 0 && matches!(rows.get(position - 1), Some(Row::Heading { .. })) {
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

    fn clamp_in_view(&mut self, rows: &[Row], height: usize, favourites: &[Variant]) {
        self.scroll = self.scroll.min(rows.len().saturating_sub(height));
        if let Some(position) = self.focused_row(rows, favourites) {
            if position < self.scroll {
                self.scroll = position;
            }
            if position >= self.scroll + height {
                self.scroll = position + 1 - height;
            }
            self.scroll = self.scroll.min(rows.len().saturating_sub(height));
        }
    }

    /// Draws the visible rows in `area` and remembers where they and the edge marks are.
    pub(super) fn draw(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        content: &Content<'_>,
        active: bool,
        ctx: &ViewCtx<'_>,
    ) {
        self.area = area;
        let height = usize::from(area.height).max(1);
        self.view_height = height;
        self.hits.clear();
        self.up_mark = None;
        self.down_mark = None;
        let rows = content.rows;
        self.clamp_in_view(rows, height, content.favourites);
        let focus_pos = self.focused_row(rows, content.favourites);
        let width = block_width(content).min(area.width);
        let x = area.x + (area.width - width) / 2;
        for (offset, (row_idx, row)) in rows
            .iter()
            .enumerate()
            .skip(self.scroll)
            .take(height)
            .enumerate()
        {
            let Ok(offset) = u16::try_from(offset) else {
                break;
            };
            let line = Rect::new(x, area.y + offset, width, 1);
            let focused = active && focus_pos == Some(row_idx);
            match row {
                Row::Heading {
                    heading,
                    folded,
                    count,
                } => {
                    draw_heading(frame, line, *heading, *folded, *count, focused, ctx);
                    self.hits.push(Hit {
                        row: line,
                        mark: Rect::default(),
                        target: HitTarget::Heading(*heading),
                    });
                }
                Row::Gap => {}
                Row::Item(variant) => {
                    draw_item(frame, line, *variant, focused, content, ctx);
                    self.hits.push(Hit {
                        row: line,
                        mark: Rect::new(x + MARKS_OFFSET, line.y, MARKS_CELLS, 1)
                            .intersection(line),
                        target: HitTarget::Item(*variant),
                    });
                }
            }
        }
        self.draw_edges(frame, area, rows.len(), x + width.saturating_sub(1), ctx);
    }

    fn draw_edges(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        total: usize,
        x: u16,
        ctx: &ViewCtx<'_>,
    ) {
        let (up, down) = if ctx.glyphs.unicode {
            ("↑", "↓")
        } else {
            ("^", "v")
        };
        let muted = ctx.theme.style(Token::TextMuted, Token::Background);
        if self.scroll > 0 {
            let cell = Rect::new(x, area.y, 1, 1);
            frame.render_widget(Paragraph::new(Line::styled(up, muted)), cell);
            self.up_mark = Some(cell);
        }
        if self.scroll + usize::from(area.height) < total && area.height > 0 {
            let cell = Rect::new(x, area.y + area.height - 1, 1, 1);
            frame.render_widget(Paragraph::new(Line::styled(down, muted)), cell);
            self.down_mark = Some(cell);
        }
    }
}

#[cfg(test)]
mod tests;
