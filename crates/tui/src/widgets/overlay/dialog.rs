//! Dialog chrome: a centered box with a title, text lines and a row of buttons. The focused button
//! is framed by markers and drawn bold and reversed, so focus is visible without color. Borders
//! and markers use only WGL4 characters in Unicode mode and plain ASCII otherwise.

use ratatui::Frame;
use ratatui::layout::{Alignment, Margin, Rect};
use ratatui::style::Modifier;
use ratatui::symbols::border;
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};

use crate::mouse::contains;
use crate::screens::ViewCtx;
use crate::theme::Token;

/// One button of a dialog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Button {
    pub label: String,
    pub focused: bool,
}

/// Where a dialog and its parts are drawn: the box, the area for text and one rectangle per button.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Geometry {
    pub frame: Rect,
    pub body: Rect,
    pub buttons: Vec<Rect>,
}

/// A dialog made of a title, text lines and buttons.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Text {
    pub title: String,
    pub lines: Vec<String>,
    pub buttons: Vec<Button>,
}

const BORDER_ROWS: u16 = 2;
const CHROME_COLUMNS: u16 = 4;
const BUTTON_ROWS: u16 = 2;
const BUTTON_GAP: u16 = 2;
const MARKER_COLUMNS: u16 = 4;
const MAX_TEXT_COLUMNS: u16 = 64;
const MIN_COLUMNS: u16 = 24;

const ASCII_BORDER: border::Set<'static> = border::Set {
    top_left: "+",
    top_right: "+",
    bottom_left: "+",
    bottom_right: "+",
    vertical_left: "|",
    vertical_right: "|",
    horizontal_top: "-",
    horizontal_bottom: "-",
};

fn columns_of(text: &str) -> u16 {
    u16::try_from(text.chars().count()).unwrap_or(u16::MAX)
}

fn button_width(button: &Button) -> u16 {
    columns_of(&button.label).saturating_add(MARKER_COLUMNS)
}

fn buttons_width(buttons: &[Button]) -> u16 {
    let labels: u16 = buttons.iter().map(button_width).sum();
    labels.saturating_add(BUTTON_GAP * u16::try_from(buttons.len().saturating_sub(1)).unwrap_or(0))
}

/// A rectangle of the given size centered in `area` (clamped to `area`).
pub fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    )
}

/// How many rows `text` takes when word-wrapped to `width` columns; an empty line takes one row.
pub fn wrapped_rows(text: &str, width: u16) -> u16 {
    let width = usize::from(width.max(1));
    let mut rows = 1_u16;
    let mut used = 0_usize;
    for word in text.split_whitespace() {
        let len = word.chars().count();
        if used == 0 {
            used = len;
        } else if used + 1 + len <= width {
            used += 1 + len;
        } else {
            rows = rows.saturating_add(1);
            used = len;
        }
    }
    rows
}

fn button_rects(inner: Rect, buttons: &[Button]) -> Vec<Rect> {
    if buttons.is_empty() || inner.height < BUTTON_ROWS {
        return Vec::new();
    }
    let y = inner.bottom() - 1;
    let spare = inner.width.saturating_sub(buttons_width(buttons));
    let mut x = inner.x.saturating_add(spare / 2);
    buttons
        .iter()
        .map(|button| {
            let width = button_width(button);
            let rect = Rect::new(x, y, width, 1).intersection(inner);
            x = x.saturating_add(width).saturating_add(BUTTON_GAP);
            rect
        })
        .collect()
}

/// A box of `width` columns and `body_rows` text rows, plus the buttons' row when there are
/// buttons, centered in `area`. The text area is the box without its border and button row.
pub fn boxed(area: Rect, width: u16, body_rows: u16, buttons: &[Button]) -> Geometry {
    let button_rows = if buttons.is_empty() { 0 } else { BUTTON_ROWS };
    let frame = centered(area, width, body_rows + BORDER_ROWS + button_rows);
    let inner = Rect::new(
        frame.x + 1,
        frame.y + 1,
        frame.width.saturating_sub(2),
        frame.height.saturating_sub(2),
    );
    let body = Rect::new(
        inner.x,
        inner.y,
        inner.width,
        inner.height.saturating_sub(button_rows),
    );
    Geometry {
        frame,
        body,
        buttons: button_rects(inner, buttons),
    }
}

impl Text {
    /// The box that fits the text and the buttons, centered in `area`.
    pub fn geometry(&self, area: Rect) -> Geometry {
        let widest = self
            .lines
            .iter()
            .map(|line| columns_of(line))
            .max()
            .unwrap_or(0)
            .min(MAX_TEXT_COLUMNS);
        let wanted = widest
            .max(buttons_width(&self.buttons))
            .max(columns_of(&self.title).saturating_add(2))
            .max(MIN_COLUMNS)
            .saturating_add(CHROME_COLUMNS);
        let text_width = wanted.saturating_sub(CHROME_COLUMNS);
        let rows: u16 = self
            .lines
            .iter()
            .map(|line| wrapped_rows(line, text_width))
            .sum();
        boxed(area, wanted, rows, &self.buttons)
    }

    /// Draws the box, the text and the buttons.
    pub fn draw(&self, frame: &mut Frame, geometry: &Geometry, ctx: &ViewCtx<'_>) {
        draw_box(frame, geometry, &self.title, ctx);
        draw_lines(frame, geometry.body, &self.lines, ctx);
        draw_buttons(frame, geometry, &self.buttons, ctx);
    }
}

/// The button under a cell, if the cell lies on one.
pub fn button_at(geometry: &Geometry, cell: (u16, u16)) -> Option<usize> {
    geometry
        .buttons
        .iter()
        .position(|rect| contains(*rect, cell))
}

/// Clears the box area and draws its border and title.
pub fn draw_box(frame: &mut Frame, geometry: &Geometry, title: &str, ctx: &ViewCtx<'_>) {
    let block = Block::bordered()
        .border_set(border_set(ctx))
        .border_style(ctx.theme.style(Token::BorderFocus, Token::Surface))
        .title(Line::styled(
            format!(" {title} "),
            ctx.theme.style(Token::Accent, Token::Surface),
        ))
        .style(ctx.theme.surface());
    frame.render_widget(Clear, geometry.frame);
    frame.render_widget(block, geometry.frame);
}

/// Draws the text lines centered in `area`, with one column of margin on each side.
pub fn draw_lines(frame: &mut Frame, area: Rect, lines: &[String], ctx: &ViewCtx<'_>) {
    let area = area.inner(Margin::new(1, 0));
    let lines: Vec<Line> = lines.iter().map(|line| Line::raw(line.as_str())).collect();
    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .style(ctx.theme.surface()),
        area,
    );
}

/// Draws each button in its rectangle. The focused one is framed by markers and drawn bold and
/// reversed; the others keep the same width with blanks in place of the markers.
pub fn draw_buttons(frame: &mut Frame, geometry: &Geometry, buttons: &[Button], ctx: &ViewCtx<'_>) {
    let (left, right) = markers(ctx);
    for (button, rect) in buttons.iter().zip(&geometry.buttons) {
        let (text, style) = if button.focused {
            (
                format!("{left} {} {right}", button.label),
                ctx.theme
                    .surface()
                    .add_modifier(Modifier::BOLD | Modifier::REVERSED),
            )
        } else {
            (format!("  {}  ", button.label), ctx.theme.surface())
        };
        frame.render_widget(
            Paragraph::new(Line::styled(text, style)).alignment(Alignment::Center),
            *rect,
        );
    }
}

/// The markers around the focused button: `►` and `◄` in Unicode mode, `>` and `<` in ASCII.
pub fn markers(ctx: &ViewCtx<'_>) -> (&'static str, &'static str) {
    if ctx.glyphs.unicode {
        ("►", "◄")
    } else {
        (">", "<")
    }
}

pub(super) fn border_set(ctx: &ViewCtx<'_>) -> border::Set<'static> {
    if ctx.glyphs.unicode {
        border::DOUBLE
    } else {
        ASCII_BORDER
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn button(label: &str, focused: bool) -> Button {
        Button {
            label: label.to_owned(),
            focused,
        }
    }

    #[test]
    fn centered_rect_is_inside_and_centered() {
        let area = Rect::new(0, 0, 100, 30);
        assert_eq!(centered(area, 40, 10), Rect::new(30, 10, 40, 10));
    }

    #[test]
    fn centered_rect_is_clamped_to_the_area() {
        let area = Rect::new(5, 5, 20, 8);
        assert_eq!(centered(area, 99, 99), area);
    }

    #[test]
    fn wrapping_counts_rows_by_words() {
        assert_eq!(wrapped_rows("", 10), 1);
        assert_eq!(wrapped_rows("one two", 7), 1);
        assert_eq!(wrapped_rows("one two three", 7), 2);
        assert_eq!(wrapped_rows("a verylongword", 4), 2);
    }

    #[test]
    fn buttons_are_centered_and_do_not_overlap() {
        let buttons = [button("Yes", true), button("No", false)];
        let geometry = boxed(Rect::new(0, 0, 40, 10), 30, 1, &buttons);
        assert_eq!(geometry.buttons.len(), 2);
        let (first, second) = (geometry.buttons[0], geometry.buttons[1]);
        assert_eq!(first.width, 7);
        assert_eq!(second.width, 6);
        assert!(first.right() <= second.x);
        assert!(
            geometry
                .buttons
                .iter()
                .all(|rect| contains(geometry.frame, (rect.x, rect.y)))
        );
    }

    #[test]
    fn a_click_finds_the_button_under_it() {
        let buttons = [button("Yes", true), button("No", false)];
        let geometry = boxed(Rect::new(0, 0, 40, 10), 30, 1, &buttons);
        let second = geometry.buttons[1];
        assert_eq!(button_at(&geometry, (second.x, second.y)), Some(1));
        assert_eq!(button_at(&geometry, (0, 0)), None);
    }
}
