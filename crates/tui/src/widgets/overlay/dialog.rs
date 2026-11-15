//! A centered dialog box: title, text lines and a row of buttons. The focused button carries
//! markers on both sides, so it is visible without color.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};

use crate::screens::ViewCtx;
use crate::theme::Token;

/// One button of a dialog.
pub struct Button {
    pub label: String,
    pub focused: bool,
}

const FRAME_COLUMNS: usize = 6;
const FRAME_ROWS: usize = 2;
const MIN_WIDTH: usize = 24;

fn clamp_to(area_len: u16, wanted: usize) -> u16 {
    u16::try_from(wanted).unwrap_or(u16::MAX).min(area_len)
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

fn button_line(buttons: &[Button], ctx: &ViewCtx<'_>) -> Line<'static> {
    let mut spans = Vec::new();
    for button in buttons {
        let (left, right, style) = if button.focused {
            (
                ctx.glyphs.focus_left,
                ctx.glyphs.focus_right,
                ctx.theme.focus(),
            )
        } else {
            (" ", " ", ctx.theme.surface())
        };
        spans.push(Span::styled(
            format!("{left} {} {right}", button.label),
            style,
        ));
        spans.push(Span::raw("  "));
    }
    spans.pop();
    Line::from(spans)
}

/// Draws the dialog over whatever is in `area`.
pub fn draw(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    body: &[String],
    buttons: &[Button],
    ctx: &ViewCtx<'_>,
) {
    let widest = body
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0);
    let buttons_width: usize = buttons.iter().map(|b| b.label.chars().count() + 6).sum();
    let wide = widest.max(buttons_width).max(title.chars().count());
    let button_rows = if buttons.is_empty() { 0 } else { 2 };
    let rect = centered(
        area,
        clamp_to(area.width, (wide + FRAME_COLUMNS).max(MIN_WIDTH)),
        clamp_to(area.height, body.len() + button_rows + FRAME_ROWS),
    );
    let block = Block::bordered()
        .border_type(ctx.glyphs.border)
        .border_style(ctx.theme.style(Token::BorderFocus, Token::Surface))
        .title(Line::styled(
            format!(" {title} "),
            ctx.theme.style(Token::Accent, Token::Surface),
        ))
        .style(ctx.theme.surface());
    let inner = block.inner(rect);
    frame.render_widget(Clear, rect);
    frame.render_widget(block, rect);
    let [text_area, button_area] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(u16::from(button_rows > 0)),
    ])
    .areas(inner);
    let lines: Vec<Line> = body.iter().map(|line| Line::raw(line.clone())).collect();
    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .style(ctx.theme.surface()),
        text_area,
    );
    if !buttons.is_empty() {
        frame.render_widget(
            Paragraph::new(button_line(buttons, ctx)).alignment(Alignment::Center),
            button_area,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centered_rect_is_inside_and_centered() {
        let area = Rect::new(0, 0, 100, 30);
        let rect = centered(area, 40, 10);
        assert_eq!(rect, Rect::new(30, 10, 40, 10));
    }

    #[test]
    fn centered_rect_is_clamped_to_the_area() {
        let area = Rect::new(5, 5, 20, 8);
        assert_eq!(centered(area, 99, 99), area);
    }
}
