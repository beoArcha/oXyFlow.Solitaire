//! The "How to play" window: the rules of one game in short sections (goal, layout, moves,
//! controls, tips), with tabs to switch between the games and a scrollable text. Left and Right
//! change the game, Tab and Shift+Tab switch the main Help tab, Up, Down, PageUp, PageDown, Home,
//! End and the mouse wheel scroll, Esc, Enter or the Close button close it. The controls are read
//! from the key table.

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

pub use self::state::{HowToPlay, Rules};
#[cfg(test)]
pub use self::view::geometry;
pub use self::view::{draw, geometry_for};
use super::dialog;
use crate::action::Action;
use crate::layout::InterfaceSize;
use crate::mouse;
use crate::screens::Effect;
use crate::widgets::scrollbar;

const PAGE_LINES: i32 = 19;
const WHEEL_LINES: i32 = 3;

/// Handles a key while the window is open at the default scale.
#[cfg(test)]
pub fn handle(
    state: &mut HowToPlay,
    key: &KeyEvent,
    action: Option<Action>,
    area: Rect,
) -> Option<Effect> {
    handle_for(state, key, action, area, InterfaceSize::default())
}

/// Handles a key while the window is open. `None` keeps it open; `Some(effect)` closes it.
pub fn handle_for(
    state: &mut HowToPlay,
    key: &KeyEvent,
    action: Option<Action>,
    area: Rect,
    size: InterfaceSize,
) -> Option<Effect> {
    let scale = view::scale_in(area, size);
    state.set_layout(
        view::text_rows_for(area, size),
        scale.how_to_play.text_width,
    );
    match key.code {
        KeyCode::PageUp => state.scroll_by(-PAGE_LINES),
        KeyCode::PageDown => state.scroll_by(PAGE_LINES),
        KeyCode::Home => state.scroll_to(false),
        KeyCode::End => state.scroll_to(true),
        _ => match action? {
            Action::Up => state.scroll_by(-1),
            Action::Down => state.scroll_by(1),
            Action::Left => state.switch(false),
            Action::Right => state.switch(true),
            Action::Select | Action::Back => return Some(Effect::None),
            _ => {}
        },
    }
    None
}

/// Handles a mouse event: the wheel scrolls, a click on an arrow or a tab switches the game, a
/// click on Close closes the window (`Some`). `area` is the whole screen.
pub fn handle_mouse_for(
    state: &mut HowToPlay,
    area: Rect,
    event: &MouseEvent,
    size: InterfaceSize,
) -> Option<Effect> {
    let scale = view::scale_in(area, size);
    let geometry = geometry_for(area, size);
    let parts = view::parts(geometry.body);
    let text_width = scale.how_to_play.text_width;
    state.set_layout(parts.text.height, text_width);
    match mouse::wheel_action(event) {
        Some(Action::Up) => state.scroll_by(-WHEEL_LINES),
        Some(Action::Down) => state.scroll_by(WHEEL_LINES),
        _ => {}
    }
    let track = scrollbar::track_of(parts.text);
    let metrics = state.metrics_for(track.height, text_width);
    if let Some(line) = scrollbar::handle_mouse(event, track, metrics, &mut state.drag) {
        state.scroll_to_line(line);
    }
    let cell = mouse::left_press(event)?;
    if dialog::button_at(&geometry, cell).is_some() {
        return Some(Effect::None);
    }
    click_games(
        state,
        view::tabs_for(parts.game_tabs, &scale.how_to_play),
        cell,
    );
    None
}

fn click_games(state: &mut HowToPlay, placed: view::Tabs, cell: (u16, u16)) {
    if mouse::contains(placed.previous, cell) {
        state.switch(false);
    } else if mouse::contains(placed.next, cell) {
        state.switch(true);
    } else if let Some((rules, _)) = placed
        .games
        .iter()
        .find(|(_, rect)| mouse::contains(*rect, cell))
    {
        state.show(*rules);
    }
}
