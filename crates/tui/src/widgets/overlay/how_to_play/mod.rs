//! The "How to play" window: the rules of one game in short sections (goal, layout, moves,
//! controls, tips), with tabs to switch between the four games and a scrollable text. Left and
//! Right (or Tab) change the game, Up, Down, PageUp, PageDown, Home, End and the mouse wheel
//! scroll, Esc, Enter or the Close button close it. The controls are read from the key table.

#[cfg(test)]
mod bar_tests;
pub(super) mod content;
#[cfg(test)]
mod focus_tests;
mod pages;
#[cfg(test)]
mod render_tests;
mod state;
#[cfg(test)]
mod tests;
mod view;

use ratatui::crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use ratatui::layout::Rect;

pub use self::state::{Focus, HowToPlay, Rules};
pub use self::view::{draw, geometry};
use super::dialog;
use crate::action::Action;
use crate::mouse;
use crate::screens::Effect;
use crate::widgets::scrollbar;

const PAGE_LINES: i32 = 19;
const WHEEL_LINES: i32 = 3;

/// Handles a key while the window is open. `None` keeps it open; `Some(effect)` closes it.
pub fn handle(state: &mut HowToPlay, key: &KeyEvent, action: Option<Action>) -> Option<Effect> {
    match key.code {
        KeyCode::PageUp => state.scroll_by(-PAGE_LINES),
        KeyCode::PageDown => state.scroll_by(PAGE_LINES),
        KeyCode::Home => state.scroll_to(false),
        KeyCode::End => state.scroll_to(true),
        KeyCode::Tab => {
            state.step_focus(true);
        }
        KeyCode::BackTab => {
            state.step_focus(false);
        }
        _ => match action? {
            Action::Up => state.up(),
            Action::Down => state.down(),
            Action::Left if state.focus == Focus::Games => state.switch(false),
            Action::Right if state.focus == Focus::Games => state.switch(true),
            Action::Select | Action::Back => return Some(Effect::None),
            _ => {}
        },
    }
    None
}

/// Handles a mouse event: the wheel scrolls, a click on an arrow or a tab switches the game, a
/// click on Close closes the window (`Some`). `area` is the whole screen.
pub fn handle_mouse(state: &mut HowToPlay, area: Rect, event: &MouseEvent) -> Option<Effect> {
    match mouse::wheel_action(event) {
        Some(Action::Up) => state.scroll_by(-WHEEL_LINES),
        Some(Action::Down) => state.scroll_by(WHEEL_LINES),
        _ => {}
    }
    let geometry = geometry(area);
    let parts = view::parts(geometry.body);
    let track = scrollbar::track_of(parts.text);
    let metrics = state.metrics(track.height);
    if let Some(line) = scrollbar::handle_mouse(event, track, metrics, &mut state.drag) {
        state.scroll_to_line(line);
        state.focus = Focus::Text;
    }
    let cell = mouse::left_press(event)?;
    if dialog::button_at(&geometry, cell).is_some() {
        return Some(Effect::None);
    }
    if mouse::contains(parts.text, cell) {
        state.focus = Focus::Text;
    }
    click_games(state, view::tabs(parts.game_tabs), cell);
    None
}

fn click_games(state: &mut HowToPlay, placed: view::Tabs, cell: (u16, u16)) {
    if mouse::contains(placed.previous, cell) {
        state.focus = Focus::Games;
        state.switch(false);
    } else if mouse::contains(placed.next, cell) {
        state.focus = Focus::Games;
        state.switch(true);
    } else if let Some((rules, _)) = placed
        .games
        .iter()
        .find(|(_, rect)| mouse::contains(*rect, cell))
    {
        state.focus = Focus::Games;
        state.show(*rules);
    }
}
