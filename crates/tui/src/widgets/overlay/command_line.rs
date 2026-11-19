//! The "Command line" tab of the help window: the text printed by `oxysol --help`, read from the
//! `cli.help` localization key so the two never differ. It scrolls inside the same box as the
//! other tabs: Up, Down, PageUp, PageDown, Home, End and the mouse wheel.

use ratatui::Frame;
use ratatui::crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::style::Modifier;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use super::dialog::{self, Geometry};
use super::help;
use super::tabs::{HelpTab, draw_footer_buttons, draw_top_tabs};
use crate::action::Action;
use crate::i18n::{t, tf};
use crate::input::primary_key;
use crate::mouse;
use crate::screens::{Effect, ViewCtx};
use crate::widgets::footer::{FooterMap, Segment};
use crate::widgets::scrollbar::{self, Metrics};

/// Rows of text visible at once: the box body minus the tab row and the footer row.
pub const VIEW_ROWS: usize = 21;

const PAGE_LINES: i32 = 20;
const WHEEL_LINES: i32 = 3;
const HEADING_SUFFIX: char = ':';

/// The open tab: the first visible line of the text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CommandLine {
    pub scroll: u16,
    drag: Option<u16>,
}

impl CommandLine {
    /// The tab opened at the top.
    pub const fn new() -> Self {
        Self {
            scroll: 0,
            drag: None,
        }
    }

    /// The largest value of `scroll`: the last line sits on the last row.
    pub fn max_scroll(&self) -> u16 {
        u16::try_from(lines().len().saturating_sub(VIEW_ROWS)).unwrap_or(u16::MAX)
    }

    /// Scrolls by `lines` (negative goes up), staying inside the text.
    pub fn scroll_by(&mut self, lines: i32) {
        let target = i32::from(self.scroll).saturating_add(lines);
        let clamped = target.clamp(0, i32::from(self.max_scroll()));
        self.scroll = u16::try_from(clamped).unwrap_or(0);
    }

    /// Scrolls to the first (`false`) or the last line.
    pub fn scroll_to(&mut self, end: bool) {
        self.scroll = if end { self.max_scroll() } else { 0 };
    }

    fn metrics(&self, view: u16) -> Metrics {
        Metrics {
            total: lines().len(),
            view: usize::from(view),
            offset: usize::from(self.scroll),
        }
    }

    fn scroll_to_line(&mut self, line: usize) {
        let clamped = line.min(usize::from(self.max_scroll()));
        self.scroll = u16::try_from(clamped).unwrap_or(0);
    }
}

/// The lines of the text: the `--help` output of the current language.
pub fn lines() -> Vec<String> {
    t("cli.help").lines().map(str::to_owned).collect()
}

fn is_heading(line: &str) -> bool {
    line.ends_with(HEADING_SUFFIX) && !line.starts_with(' ')
}

/// Handles a key while the tab is open. `None` keeps it open; `Some(effect)` closes it.
pub fn handle(state: &mut CommandLine, key: &KeyEvent, action: Option<Action>) -> Option<Effect> {
    match key.code {
        KeyCode::PageUp => state.scroll_by(-PAGE_LINES),
        KeyCode::PageDown => state.scroll_by(PAGE_LINES),
        KeyCode::Home => state.scroll_to(false),
        KeyCode::End => state.scroll_to(true),
        _ => match action? {
            Action::Up => state.scroll_by(-1),
            Action::Down => state.scroll_by(1),
            Action::Select | Action::Back => return Some(Effect::None),
            _ => {}
        },
    }
    None
}

/// Handles a mouse event: the wheel scrolls, a click on Close closes the tab (`Some`). `area` is
/// the whole screen.
pub fn handle_mouse(state: &mut CommandLine, area: Rect, event: &MouseEvent) -> Option<Effect> {
    match mouse::wheel_action(event) {
        Some(Action::Up) => state.scroll_by(-WHEEL_LINES),
        Some(Action::Down) => state.scroll_by(WHEEL_LINES),
        _ => {}
    }
    let geometry = geometry(area);
    let track = scrollbar::track_of(parts(geometry.body).text);
    let metrics = state.metrics(track.height);
    if let Some(line) = scrollbar::handle_mouse(event, track, metrics, &mut state.drag) {
        state.scroll_to_line(line);
    }
    let cell = mouse::left_press(event)?;
    dialog::button_at(&geometry, cell).map(|_| Effect::None)
}

fn parts(body: Rect) -> Parts {
    let [tabs, text, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(body);
    Parts { tabs, text, footer }
}

struct Parts {
    tabs: Rect,
    text: Rect,
    footer: Rect,
}

/// The box of the tab: the same as the other tabs, so switching does not move the window.
pub fn geometry(area: Rect) -> Geometry {
    help::geometry(area)
}

fn key_name(action: Action) -> String {
    primary_key(action).unwrap_or_default()
}

fn footer_segments() -> Vec<Segment> {
    let move_keys = tf(
        "help.command_line.footer_move",
        &[
            ("up", &key_name(Action::Up)),
            ("down", &key_name(Action::Down)),
        ],
    );
    vec![
        Segment::button(
            Action::FocusNext,
            tf(
                "help.command_line.footer_tab",
                &[("key", &key_name(Action::FocusNext))],
            ),
        ),
        Segment::plain("   "),
        Segment::plain(move_keys),
        Segment::plain("   "),
        Segment::button(
            Action::Back,
            tf(
                "help.command_line.footer_close",
                &[("back", &key_name(Action::Back))],
            ),
        ),
    ]
}

/// Draws the tab over `area` and returns its footer buttons.
pub fn draw(frame: &mut Frame, area: Rect, state: &CommandLine, ctx: &ViewCtx<'_>) -> FooterMap {
    let geometry = geometry(area);
    dialog::draw_box(frame, &geometry, &t("common.dialog.help_title"), ctx);
    let Parts { tabs, text, footer } = parts(geometry.body);
    draw_top_tabs(frame, tabs, HelpTab::CommandLine, true, ctx);
    let all = lines();
    let first = usize::from(state.scroll);
    let plain = ctx.theme.surface();
    let shown: Vec<Line> = all
        .iter()
        .skip(first)
        .take(usize::from(text.height))
        .map(|line| {
            if is_heading(line) {
                Line::styled(line.clone(), plain.add_modifier(Modifier::BOLD))
            } else {
                Line::styled(line.clone(), plain)
            }
        })
        .collect();
    let count = shown.len();
    frame.render_widget(
        Paragraph::new(shown).style(plain),
        text.inner(Margin::new(1, 0)),
    );
    scrollbar::draw(
        frame,
        scrollbar::track_of(text),
        state.metrics(text.height),
        ctx,
    );
    let buttons_drawn = draw_footer_buttons(
        frame,
        footer,
        &footer_segments(),
        (first + 1, first + count, all.len()),
        ctx,
    );
    dialog::draw_buttons(frame, &geometry, &help::buttons(), ctx);
    buttons_drawn
}
