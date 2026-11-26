//! The Keys tab of the help window, scrollable: the state of its scroll and its scroll bar, and its
//! keys and mouse. It is drawn by [`super::help`], whose two columns it scrolls together.

use ratatui::crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use ratatui::layout::Rect;

use super::dialog;
use super::help::{Columns, Section, VIEW_ROWS, geometry_for, parts, view_rows_for};
use super::nav;
use crate::action::Action;
use crate::layout::InterfaceSize;
use crate::mouse;
use crate::screens::Effect;
use crate::widgets::scrollbar::{self, Metrics};

const WHEEL_LINES: i32 = 3;

const PAGE_LINES: i32 = 20;

/// The Keys tab of the help window: its first visible line and the text rows the box shows. Both
/// columns scroll together, and the tab scrolls only when the taller column does not fit the box.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HelpKeys {
    pub scroll: u16,
    drag: Option<u16>,
    view: u16,
}

impl Default for HelpKeys {
    fn default() -> Self {
        Self {
            scroll: 0,
            drag: None,
            view: VIEW_ROWS as u16,
        }
    }
}

/// Lines of a column: each topic's heading and its rows.
fn height(sections: &[Section]) -> usize {
    sections.iter().map(|section| 1 + section.rows.len()).sum()
}

impl HelpKeys {
    /// The lines of the taller column, the scroll bar's total.
    fn total() -> usize {
        let columns = Columns::new();
        height(&columns.left).max(height(&columns.right))
    }

    /// The largest scroll: the taller column ends on the last text row.
    pub fn max_scroll(&self) -> u16 {
        u16::try_from(Self::total().saturating_sub(usize::from(self.view))).unwrap_or(u16::MAX)
    }

    /// Takes the number of text rows the box shows at the current size.
    fn set_view(&mut self, rows: u16) {
        self.view = rows;
    }

    fn scroll_by(&mut self, lines: i32) {
        let target = i32::from(self.scroll).saturating_add(lines);
        let clamped = target.clamp(0, i32::from(self.max_scroll()));
        self.scroll = u16::try_from(clamped).unwrap_or(0);
    }

    fn scroll_to(&mut self, end: bool) {
        self.scroll = if end { self.max_scroll() } else { 0 };
    }

    fn scroll_to_line(&mut self, line: usize) {
        let clamped = line.min(usize::from(self.max_scroll()));
        self.scroll = u16::try_from(clamped).unwrap_or(0);
    }

    pub(super) fn metrics(&self) -> Metrics {
        Metrics {
            total: Self::total(),
            view: usize::from(self.view),
            offset: usize::from(self.scroll),
        }
    }
}

/// Handles a key while the Keys tab is open: Up, Down, PageUp, PageDown, Home and End scroll; Esc,
/// Enter and the close key close the window (`Some`). Tab and the other keys are the tab keys.
pub fn handle_key_for(
    state: &mut HelpKeys,
    key: &KeyEvent,
    action: Option<Action>,
    area: Rect,
    size: InterfaceSize,
) -> Option<Effect> {
    state.set_view(view_rows_for(area, size));
    match key.code {
        KeyCode::PageUp => state.scroll_by(-PAGE_LINES),
        KeyCode::PageDown => state.scroll_by(PAGE_LINES),
        KeyCode::Home => state.scroll_to(false),
        KeyCode::End => state.scroll_to(true),
        _ => match action {
            Some(Action::Up) => state.scroll_by(-1),
            Some(Action::Down) => state.scroll_by(1),
            _ => return nav::close_help(key, action),
        },
    }
    None
}

/// Handles a mouse event: the wheel and the scroll bar scroll, a click on Close closes the window.
pub fn handle_mouse_for(
    state: &mut HelpKeys,
    area: Rect,
    event: &MouseEvent,
    size: InterfaceSize,
) -> Option<Effect> {
    let geometry = geometry_for(area, size);
    let (_, text, _, _) = parts(geometry.body);
    state.set_view(text.height);
    match mouse::wheel_action(event) {
        Some(Action::Up) => state.scroll_by(-WHEEL_LINES),
        Some(Action::Down) => state.scroll_by(WHEEL_LINES),
        _ => {}
    }
    let track = scrollbar::track_of(text);
    let metrics = state.metrics();
    if let Some(line) = scrollbar::handle_mouse(event, track, metrics, &mut state.drag) {
        state.scroll_to_line(line);
    }
    let cell = mouse::left_press(event)?;
    dialog::button_at(&geometry, cell).map(|_| Effect::None)
}
