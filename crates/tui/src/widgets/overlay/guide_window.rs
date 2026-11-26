//! The guide window: the content of the side panel (Game, Goal, Keys, Mouse, Legend) as a box over
//! the board, for screens narrower than the panel needs. It scrolls with Up, Down, PageUp, PageDown,
//! Home, End and the mouse wheel, and closes with the guide key, Esc, Enter or the Close button.

use ratatui::Frame;
use ratatui::crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use super::dialog::{self, Geometry};
use super::help;
use super::tabs::draw_footer_buttons;
use crate::action::Action;
use crate::i18n::{t, tf};
use crate::input::primary_key;
use crate::layout::{GuideWindowScale, InterfaceSize};
use crate::mouse;
use crate::screens::board::BoardScreen;
use crate::screens::{Effect, ViewCtx};
use crate::widgets::footer::{FooterMap, Segment};
use crate::widgets::scrollbar::{self, Metrics};

/// Text rows of the window at the default scale, below which it scrolls.
pub const TEXT_ROWS: usize = InterfaceSize::M.scale().guide_window.text_rows;
/// Columns the window text is laid out in at the default scale.
pub const TEXT_WIDTH: usize = InterfaceSize::M.scale().guide_window.text_width;
const PAGE_LINES: i32 = 15;
const WHEEL_LINES: i32 = 3;

/// The open window: how far it is scrolled and how many lines its content has.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuideWindow {
    pub scroll: u16,
    pub total: u16,
    drag: Option<u16>,
    view: u16,
}

impl GuideWindow {
    /// The window at the top of `total` lines.
    pub fn new(total: usize) -> Self {
        Self {
            scroll: 0,
            total: u16::try_from(total).unwrap_or(u16::MAX),
            drag: None,
            view: u16::try_from(TEXT_ROWS).unwrap_or(u16::MAX),
        }
    }

    /// Takes the number of text rows the box shows at the current scale and window size.
    pub fn set_view(&mut self, rows: u16) {
        self.view = rows;
    }

    fn max_scroll(&self) -> u16 {
        self.total.saturating_sub(self.view)
    }

    fn scroll_by(&mut self, lines: i32) {
        let target = i32::from(self.scroll).saturating_add(lines);
        let clamped = target.clamp(0, i32::from(self.max_scroll()));
        self.scroll = u16::try_from(clamped).unwrap_or(0);
    }

    fn scroll_to(&mut self, end: bool) {
        self.scroll = if end { self.max_scroll() } else { 0 };
    }

    fn metrics(&self, view: u16) -> Metrics {
        Metrics {
            total: usize::from(self.total),
            view: usize::from(view),
            offset: usize::from(self.scroll),
        }
    }

    fn scroll_to_line(&mut self, line: usize) {
        let clamped = line.min(usize::from(self.max_scroll()));
        self.scroll = u16::try_from(clamped).unwrap_or(0);
    }
}

fn scale_in(area: Rect, size: InterfaceSize) -> (InterfaceSize, &'static GuideWindowScale) {
    let resolved = size.fit(|scale| scale.guide_window.box_width <= area.width);
    (resolved, &resolved.scale().guide_window)
}

/// Columns the guide window wraps text to at `size` in `area`.
pub fn text_width_in(area: Rect, size: InterfaceSize) -> usize {
    scale_in(area, size).1.text_width
}

/// Rows the guide window displays at `size` in `area`.
pub fn text_rows_in(area: Rect, size: InterfaceSize) -> u16 {
    u16::try_from(scale_in(area, size).1.text_rows).unwrap_or(u16::MAX)
}

/// The box of the window, centered in `area` at the default interface scale.
pub fn geometry(area: Rect) -> Geometry {
    geometry_for(area, InterfaceSize::default())
}

/// The box of the window, centered in `area` at `size`. Its size does not depend on the content.
pub fn geometry_for(area: Rect, size: InterfaceSize) -> Geometry {
    let (resolved, scale) = scale_in(area, size);
    let rows = u16::try_from(scale.text_rows + 1).unwrap_or(u16::MAX);
    dialog::boxed_for(area, scale.box_width, rows, &help::buttons(), resolved)
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
    let size = ctx.settings.interface_size;
    let geometry = geometry_for(area, size);
    dialog::draw_box(frame, &geometry, &t("board.guide.window_title"), ctx);
    let (text, footer) = split(geometry.body);
    let all = board.guide_lines(text_width_in(area, size), ctx);
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
    scrollbar::draw(
        frame,
        scrollbar::track_of(text),
        state.metrics(text.height),
        ctx,
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
    dialog::draw_buttons(frame, &geometry, &help::buttons(), ctx);
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

/// Handles a mouse event at the default scale.
#[cfg(test)]
pub fn handle_mouse(state: &mut GuideWindow, area: Rect, event: &MouseEvent) -> Option<Effect> {
    handle_mouse_for(state, area, event, InterfaceSize::default())
}

/// Handles a mouse event: the wheel scrolls, a click on Close closes the window (`Some`).
pub fn handle_mouse_for(
    state: &mut GuideWindow,
    area: Rect,
    event: &MouseEvent,
    size: InterfaceSize,
) -> Option<Effect> {
    let geometry = geometry_for(area, size);
    let text = split(geometry.body).0;
    state.set_view(text.height);
    match mouse::wheel_action(event) {
        Some(Action::Up) => state.scroll_by(-WHEEL_LINES),
        Some(Action::Down) => state.scroll_by(WHEEL_LINES),
        _ => {}
    }
    let track = scrollbar::track_of(text);
    let metrics = state.metrics(track.height);
    if let Some(line) = scrollbar::handle_mouse(event, track, metrics, &mut state.drag) {
        state.scroll_to_line(line);
    }
    let cell = mouse::left_press(event)?;
    dialog::button_at(&geometry, cell).map(|_| Effect::None)
}
