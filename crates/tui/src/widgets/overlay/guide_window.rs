//! The guide window: the content of the side panel (Game, Goal, Keys, Mouse, Legend) as a box over
//! the board, for screens narrower than the panel needs. It scrolls with Up, Down, PageUp, PageDown,
//! Home, End and the mouse wheel, and closes with the guide key, Esc, Enter or the Close button.

use ratatui::Frame;
use ratatui::crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use super::dialog::{self, Button, Geometry};
use super::tabs::draw_footer_buttons;
use crate::action::Action;
use crate::i18n::{t, tf};
use crate::input::primary_key;
use crate::mouse;
use crate::screens::board::BoardScreen;
use crate::screens::{Effect, ViewCtx};
use crate::widgets::footer::{FooterMap, Segment};

/// Text rows of the window, below which it scrolls.
pub const TEXT_ROWS: usize = 17;
/// Columns the window text is laid out in.
pub const TEXT_WIDTH: usize = 58;
const BOX_WIDTH: u16 = 62;
const PAGE_LINES: i32 = 15;
const WHEEL_LINES: i32 = 3;

/// The open window: how far it is scrolled and how many lines its content has.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuideWindow {
    pub scroll: u16,
    pub total: u16,
}

impl GuideWindow {
    /// The window at the top of `total` lines.
    pub fn new(total: usize) -> Self {
        Self {
            scroll: 0,
            total: u16::try_from(total).unwrap_or(u16::MAX),
        }
    }

    fn max_scroll(&self) -> u16 {
        self.total
            .saturating_sub(u16::try_from(TEXT_ROWS).unwrap_or(u16::MAX))
    }

    fn scroll_by(&mut self, lines: i32) {
        let target = i32::from(self.scroll).saturating_add(lines);
        let clamped = target.clamp(0, i32::from(self.max_scroll()));
        self.scroll = u16::try_from(clamped).unwrap_or(0);
    }

    fn scroll_to(&mut self, end: bool) {
        self.scroll = if end { self.max_scroll() } else { 0 };
    }
}

fn buttons() -> Vec<Button> {
    vec![Button {
        label: t("help.rules.close").into_owned(),
        focused: true,
    }]
}

/// The box of the window, centered in `area`. Its size does not depend on the content.
pub fn geometry(area: Rect) -> Geometry {
    let rows = u16::try_from(TEXT_ROWS + 1).unwrap_or(u16::MAX);
    dialog::boxed(area, BOX_WIDTH, rows, &buttons())
}

fn split(body: Rect) -> (Rect, Rect) {
    let [text, footer] = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(body);
    (text, footer)
}

fn key_name(action: Action) -> String {
    primary_key(action).unwrap_or_default()
}

/// Draws the window over `area` and returns its footer buttons.
pub fn draw(
    frame: &mut Frame,
    area: Rect,
    state: &GuideWindow,
    board: &BoardScreen,
    ctx: &ViewCtx<'_>,
) -> FooterMap {
    let geometry = geometry(area);
    dialog::draw_box(frame, &geometry, &t("board.guide.window_title"), ctx);
    let (text, footer) = split(geometry.body);
    let all = board.guide_lines(TEXT_WIDTH, ctx);
    let first = usize::from(state.scroll);
    let lines: Vec<Line> = all
        .iter()
        .skip(first)
        .take(usize::from(text.height))
        .cloned()
        .collect();
    let shown = lines.len();
    frame.render_widget(
        Paragraph::new(lines).style(ctx.theme.surface()),
        text.inner(Margin::new(1, 0)),
    );
    let scroll = tf(
        "board.guide.window_scroll",
        &[
            ("up", &key_name(Action::Up)),
            ("down", &key_name(Action::Down)),
        ],
    );
    let close = tf(
        "board.guide.window_close",
        &[
            ("key", &key_name(Action::Guide)),
            ("back", &key_name(Action::Back)),
        ],
    );
    let segments = [
        Segment::plain(scroll),
        Segment::plain("   "),
        Segment::button(Action::Guide, close),
    ];
    let buttons_drawn = draw_footer_buttons(
        frame,
        footer,
        &segments,
        (first + 1, first + shown, all.len()),
        ctx,
    );
    dialog::draw_buttons(frame, &geometry, &buttons(), ctx);
    buttons_drawn
}

/// Handles a key while the window is open. `None` keeps it open; `Some(effect)` closes it.
pub fn handle(state: &mut GuideWindow, key: &KeyEvent, action: Option<Action>) -> Option<Effect> {
    match key.code {
        KeyCode::PageUp => state.scroll_by(-PAGE_LINES),
        KeyCode::PageDown => state.scroll_by(PAGE_LINES),
        KeyCode::Home => state.scroll_to(false),
        KeyCode::End => state.scroll_to(true),
        KeyCode::Esc | KeyCode::Enter => return Some(Effect::None),
        _ => match action? {
            Action::Up => state.scroll_by(-1),
            Action::Down => state.scroll_by(1),
            Action::Select | Action::Back | Action::Guide => return Some(Effect::None),
            _ => {}
        },
    }
    None
}

/// Handles a mouse event: the wheel scrolls, a click on Close closes the window (`Some`).
pub fn handle_mouse(state: &mut GuideWindow, area: Rect, event: &MouseEvent) -> Option<Effect> {
    match mouse::wheel_action(event) {
        Some(Action::Up) => state.scroll_by(-WHEEL_LINES),
        Some(Action::Down) => state.scroll_by(WHEEL_LINES),
        _ => {}
    }
    let cell = mouse::left_press(event)?;
    dialog::button_at(&geometry(area), cell).map(|_| Effect::None)
}
