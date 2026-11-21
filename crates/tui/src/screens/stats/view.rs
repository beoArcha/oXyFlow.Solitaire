//! Drawing the statistics screen and remembering where its rows and buttons are for the mouse.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::format::{
    LONGEST_STREAK_MIN_WIDTH, WINNABLE_TEXT_MIN_WIDTH, history_header, history_row, legend_line,
    summary_lines,
};
use super::{Button, StatsScreen, Stop};
use crate::action::Action;
use crate::i18n::{t, tf};
use crate::screens::ViewCtx;
use crate::screens::focus::{button_style, marker};
use crate::theme::Token;
use crate::widgets::footer::{self, Segment};
use crate::widgets::scrollbar::{self, Metrics};

/// Rows of the screen that are not the summary or the history list: title, two gaps, the history
/// heading and column header, the legend, the buttons, the status line and the key line.
const FIXED_ROWS: u16 = 9;
/// History rows that always stay visible. The statistics screen is fitted at 100 × 60 (T-40): the
/// summary of every game in both undo modes is longer than a 30-row screen leaves room for.
const MIN_LIST_ROWS: u16 = 4;
const BUTTON_GAP: u16 = 2;
const MARKER_COLUMNS: u16 = 4;

pub(super) fn centered(frame: &mut Frame, area: Rect, text: String, style: Style) {
    frame.render_widget(
        Paragraph::new(Line::styled(text, style)).alignment(Alignment::Center),
        area,
    );
}

fn text_width(text: &str) -> u16 {
    u16::try_from(text.chars().count()).unwrap_or(u16::MAX)
}

fn block_x(area: Rect, width: u16) -> u16 {
    area.x + area.width.saturating_sub(width) / 2
}

/// The key line: the list keys as text, then the three keys that are buttons.
fn key_line() -> Vec<Segment> {
    vec![
        Segment::plain(t("stats.footer_list")),
        Segment::plain("   "),
        Segment::button(Action::Select, t("stats.footer_replay")),
        Segment::plain("   "),
        Segment::button(Action::CopySeed, t("stats.footer_copy")),
        Segment::button(Action::Back, t("stats.footer_back")),
    ]
}

fn button_label(button: Button) -> String {
    let key = match button {
        Button::Replay => "stats.button.replay",
        Button::Copy => "stats.button.copy",
        Button::Check => "stats.button.check",
        Button::CheckAll => "stats.button.check_all",
        Button::Back => "stats.button.back",
    };
    t(key).into_owned()
}

impl StatsScreen {
    fn summary_height(&self, screen: u16) -> u16 {
        if self.summaries.is_empty() {
            return 0;
        }
        let rows = u16::try_from(self.summaries.len() + 1).unwrap_or(u16::MAX);
        rows.min(screen.saturating_sub(FIXED_ROWS + MIN_LIST_ROWS))
    }

    /// Draws the title, the summary, the history, the buttons, the status line and the key line.
    pub fn draw(&mut self, frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>) {
        let [
            title_area,
            _,
            summary_area,
            _,
            heading_area,
            header_area,
            list_area,
            legend_area,
            buttons_area,
            status_area,
            footer_area,
        ] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(self.summary_height(area.height)),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(area);
        let accent = ctx.theme.style(Token::Accent, Token::Background);
        centered(frame, title_area, t("stats.title").into_owned(), accent);
        self.rows.clear();
        if self.records.is_empty() {
            self.draw_empty(frame, list_area, ctx);
        } else {
            if summary_area.height > 0 {
                self.draw_summary(frame, summary_area, ctx);
            }
            self.draw_history(frame, [heading_area, header_area, list_area], ctx);
            self.draw_legend(frame, legend_area, ctx);
        }
        self.draw_buttons(frame, buttons_area, ctx);
        self.draw_status(frame, status_area, ctx);
        self.footer = footer::draw(
            frame,
            footer_area,
            &key_line(),
            Alignment::Center,
            footer::key_style(ctx),
            ctx,
        );
    }

    fn draw_empty(&self, frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>) {
        let (key, token) = if self.load_failed {
            ("stats.load_failed", Token::Error)
        } else {
            ("stats.empty", Token::Text)
        };
        let style = ctx.theme.style(token, Token::Background);
        centered(
            frame,
            Rect { height: 1, ..area },
            t(key).into_owned(),
            style,
        );
    }

    fn draw_summary(&self, frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>) {
        let with_longest = area.width >= LONGEST_STREAK_MIN_WIDTH;
        let lines = summary_lines(&self.summaries, self.show_modes, with_longest);
        let width = lines.first().map_or(0, |line| text_width(line));
        let x = block_x(area, width);
        for (offset, line) in lines.into_iter().take(usize::from(area.height)).enumerate() {
            let Ok(row) = u16::try_from(offset) else {
                break;
            };
            let style = if offset == 0 {
                ctx.theme.style(Token::Accent, Token::Background)
            } else {
                ctx.theme.base()
            };
            let cell = Rect::new(x, area.y + row, width.min(area.width), 1);
            frame.render_widget(Paragraph::new(Line::styled(line, style)), cell);
        }
    }

    fn draw_history(&mut self, frame: &mut Frame, areas: [Rect; 3], ctx: &ViewCtx<'_>) {
        let [heading_area, header_area, list_area] = areas;
        let position = tf(
            "stats.position",
            &[
                ("current", &(self.list.selected() + 1).to_string()),
                ("total", &self.records.len().to_string()),
            ],
        );
        let heading = format!("{}   {position}", t("stats.history.title"));
        let accent = ctx.theme.style(Token::Accent, Token::Background);
        centered(frame, heading_area, heading, accent);
        let short = list_area.width < WINNABLE_TEXT_MIN_WIDTH;
        let header = history_header(ctx.glyphs, short);
        let width = text_width(&header) + MARKER_COLUMNS;
        let x = block_x(list_area, width);
        let header_cell = Rect::new(x, header_area.y, width.min(header_area.width), 1);
        let muted = ctx.theme.style(Token::TextMuted, Token::Background);
        let header_line = Line::styled(format!("  {header}  "), muted);
        frame.render_widget(Paragraph::new(header_line), header_cell);
        let track = Rect::new(
            x + width.saturating_sub(1),
            list_area.y,
            1,
            list_area.height,
        );
        self.track = track;
        let window = self
            .list
            .visible(usize::from(list_area.height), self.records.len());
        let list_focused = self.stop == Stop::List;
        let row_width = width.saturating_sub(1).min(list_area.width);
        for (offset, index) in window.enumerate() {
            let (Some(record), Ok(row)) = (self.records.get(index), u16::try_from(offset)) else {
                break;
            };
            let focused = list_focused && index == self.list.selected();
            let style = if focused {
                ctx.theme.focus()
            } else {
                ctx.theme.base()
            };
            let cell = Rect::new(x, list_area.y + row, row_width, 1);
            let text = format!(
                "{} {}",
                marker(ctx, focused),
                history_row(record, ctx.glyphs, short)
            );
            frame.render_widget(Paragraph::new(Line::styled(text, style)), cell);
            self.rows.push((index, cell));
        }
        let metrics = Metrics {
            total: self.records.len(),
            view: usize::from(list_area.height),
            offset: self.list.offset(),
        };
        scrollbar::draw(frame, track, metrics, ctx);
    }

    fn draw_legend(&self, frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>) {
        if area.height == 0 {
            return;
        }
        let text = legend_line(ctx.glyphs);
        let style = ctx.theme.style(Token::TextMuted, Token::Background);
        centered(frame, area, text, style);
    }

    fn draw_buttons(&mut self, frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>) {
        let labels: Vec<(Button, String)> = self
            .visible_buttons()
            .into_iter()
            .map(|button| {
                let focused = self.stop == Stop::Button(button);
                let text = format!("{} [ {} ]", marker(ctx, focused), button_label(button));
                (button, text)
            })
            .collect();
        let gaps = BUTTON_GAP * u16::try_from(labels.len().saturating_sub(1)).unwrap_or(0);
        let total: u16 = labels
            .iter()
            .map(|(_, label)| text_width(label))
            .sum::<u16>()
            + gaps;
        let mut x = block_x(area, total);
        self.buttons.clear();
        let mut spans = Vec::new();
        for (index, (button, label)) in labels.into_iter().enumerate() {
            let width = text_width(&label);
            if index > 0 {
                spans.push(Span::raw(" ".repeat(usize::from(BUTTON_GAP))));
                x += BUTTON_GAP;
            }
            self.buttons.push((button, Rect::new(x, area.y, width, 1)));
            let focused = self.stop == Stop::Button(button);
            spans.push(Span::styled(label, button_style(ctx, focused)));
            x += width;
        }
        frame.render_widget(
            Paragraph::new(Line::from(spans)).alignment(Alignment::Center),
            area,
        );
    }
}
