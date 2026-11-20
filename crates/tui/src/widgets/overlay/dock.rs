//! The docked help window: a column at one side of the screen, full height, beside the board.
//! It is as wide as the room beyond the board's minimum width (between 30 and 60 columns) and
//! exists only when that room is at least 30 columns. Its state, its placement and its keys and
//! mouse live here; [`dock_view`] draws it and builds its text.

use ratatui::crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use ratatui::layout::{Constraint, Layout, Rect};

use super::how_to_play::Rules;
use super::tabs::HelpTab;
use super::{help, nav};
use crate::action::Action;
use crate::app::Modal;
use crate::i18n::{t, tf};

#[cfg(test)]
mod dock_tests;
pub mod dock_view;
use crate::layout::BOARD_MIN_WIDTH;
use crate::mouse;
use crate::screens::Effect;
use crate::settings::DockSide;

/// Narrowest docked window, the width of the guide panel.
pub const MIN_WIDTH: u16 = 30;
/// Widest docked window; wider screens leave the rest to the board.
pub const MAX_WIDTH: u16 = 60;
const WHEEL_LINES: i32 = 3;
const ARROW_HIT: u16 = 2;

/// Where the keys go while the help window is open, and how far its docked text is scrolled.
/// `focused` is true when the docked window has the keys; the board has them otherwise.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DockState {
    pub focused: bool,
    pub scroll: u16,
}

/// The rows of a docked window: the tab row, the game row, the text and the footer. The text
/// rectangle includes the scroll bar column on its right.
pub struct Parts {
    pub top: Rect,
    pub games: Rect,
    pub text: Rect,
    pub footer: Rect,
}

/// Columns of a docked window on a screen of `area`: the room beyond the board's minimum width,
/// between [`MIN_WIDTH`] and [`MAX_WIDTH`]. `None` when the room is too small for the window.
pub fn width_in(area: Rect) -> Option<u16> {
    let room = area.width.checked_sub(BOARD_MIN_WIDTH)?;
    (room >= MIN_WIDTH).then_some(room.min(MAX_WIDTH))
}

/// The docked window's rectangle and the rest of the screen beside it, for `side` on `area`.
/// `None` when the window stays centred or the screen is too narrow to dock it.
pub fn split(area: Rect, side: DockSide) -> Option<(Rect, Rect)> {
    let width = width_in(area)?;
    let rest = area.width - width;
    match side {
        DockSide::Centre => None,
        DockSide::Right => Some((
            Rect::new(area.x + rest, area.y, width, area.height),
            Rect::new(area.x, area.y, rest, area.height),
        )),
        DockSide::Left => Some((
            Rect::new(area.x, area.y, width, area.height),
            Rect::new(area.x + width, area.y, rest, area.height),
        )),
    }
}

/// The rows of a docked window inside its border.
pub fn parts(rect: Rect) -> Parts {
    let inner = Rect::new(
        rect.x.saturating_add(1),
        rect.y.saturating_add(1),
        rect.width.saturating_sub(2),
        rect.height.saturating_sub(2),
    );
    let [top, games, text, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(inner);
    Parts {
        top,
        games,
        text,
        footer,
    }
}

/// Columns the docked text wraps to: the window inside its border, less the margin and the scroll
/// bar column.
pub fn text_width(rect: Rect) -> u16 {
    parts(rect).text.width.saturating_sub(2)
}

/// The label of the placement button in the title bar, for example `Dock: right`.
pub fn button_label(side: DockSide) -> String {
    tf("help.dock.button", &[("side", &t(side.label_key()))])
}

/// The rectangle of the placement button on the top border of a box at `frame`, just inside the
/// right corner.
pub fn button_rect(frame: Rect, side: DockSide) -> Rect {
    let width = u16::try_from(button_label(side).chars().count()).unwrap_or(u16::MAX);
    let corner = frame.right().saturating_sub(1);
    let x = corner.saturating_sub(width).max(frame.x);
    Rect::new(x, frame.y, width.min(corner.saturating_sub(x)), 1)
}

/// Whether `cell` lies on the placement button of the box at `frame`.
pub fn button_hit(frame: Rect, side: DockSide, cell: (u16, u16)) -> bool {
    mouse::contains(button_rect(frame, side), cell)
}

/// The text of the Close button, as the help window words it.
pub fn close_label() -> String {
    help::buttons()
        .first()
        .map(|button| button.label.clone())
        .unwrap_or_default()
}

/// The rectangle of the Close button on the footer row of a docked window at `rect`.
pub fn close_rect(rect: Rect) -> Rect {
    let footer = parts(rect).footer;
    let width = u16::try_from(close_label().chars().count()).unwrap_or(u16::MAX);
    let x = footer.x + footer.width.saturating_sub(width) / 2;
    Rect::new(x, footer.y, width.min(footer.width), 1)
}

fn rules_of(modal: &Modal) -> Option<Rules> {
    match modal {
        Modal::HowToPlay(state) => Some(state.rules),
        _ => None,
    }
}

/// The largest scroll offset of the docked text, in wrapped lines.
pub fn max_scroll(modal: &Modal, rect: Rect) -> u16 {
    let total = dock_view::body(modal, text_width(rect)).len();
    let view = usize::from(parts(rect).text.height);
    u16::try_from(total.saturating_sub(view)).unwrap_or(u16::MAX)
}

fn scroll(state: &mut DockState, modal: &Modal, rect: Rect, lines: i32) {
    let max = i32::from(max_scroll(modal, rect));
    let target = i32::from(state.scroll).saturating_add(lines).clamp(0, max);
    state.scroll = u16::try_from(target).unwrap_or(0);
}

fn show(state: &mut DockState, modal: &mut Modal, rules: Rules, tab: HelpTab) {
    let rules = rules_of(modal).unwrap_or(rules);
    *modal = nav::modal_for(tab, rules, None);
    state.scroll = 0;
}

fn switch_tab(state: &mut DockState, modal: &mut Modal, rules: Rules, forward: bool) {
    let Some(current) = nav::tab_of(modal) else {
        return;
    };
    let tab = if forward {
        current.next()
    } else {
        current.previous()
    };
    show(state, modal, rules, tab);
}

fn go_to(state: &mut DockState, modal: &mut Modal, rules: Rules, tab: HelpTab) {
    if nav::tab_of(modal) != Some(tab) {
        show(state, modal, rules, tab);
    }
}

/// Left or Right: the next or previous game on the Rules tab, the next or previous tab otherwise.
fn step(state: &mut DockState, modal: &mut Modal, rules: Rules, forward: bool) {
    if let Modal::HowToPlay(game) = modal {
        game.switch(forward);
        state.scroll = 0;
    } else {
        switch_tab(state, modal, rules, forward);
    }
}

/// Handles a key while the docked window has the keys. `None` keeps it open; `Some(effect)`
/// closes it. Tab and Shift+Tab step the tabs, Left and Right the games or the tabs, Up, Down,
/// PageUp, PageDown, Home and End scroll, `?` and `i` jump to the Keys and Rules tabs.
pub fn handle_key(
    state: &mut DockState,
    modal: &mut Modal,
    rect: Rect,
    rules: Rules,
    key: &KeyEvent,
    action: Option<Action>,
) -> Option<Effect> {
    let page = i32::from(parts(rect).text.height.max(2)) - 1;
    match key.code {
        KeyCode::PageUp => scroll(state, modal, rect, -page),
        KeyCode::PageDown => scroll(state, modal, rect, page),
        KeyCode::Home => state.scroll = 0,
        KeyCode::End => state.scroll = max_scroll(modal, rect),
        KeyCode::Tab => switch_tab(state, modal, rules, true),
        KeyCode::BackTab => switch_tab(state, modal, rules, false),
        KeyCode::Esc | KeyCode::Enter => return Some(Effect::None),
        _ => match action? {
            Action::Up => scroll(state, modal, rect, -1),
            Action::Down => scroll(state, modal, rect, 1),
            Action::Left => step(state, modal, rules, false),
            Action::Right => step(state, modal, rules, true),
            Action::Select | Action::Back => return Some(Effect::None),
            Action::Help => go_to(state, modal, rules, HelpTab::Keys),
            Action::Rules => go_to(state, modal, rules, HelpTab::Rules),
            _ => {}
        },
    }
    None
}

/// The first column and the end column (exclusive) of a row's `◄ name ►` text, centred in `row`.
fn span(row: Rect, name: &str) -> (u16, u16) {
    let total = u16::try_from(name.chars().count())
        .unwrap_or(u16::MAX)
        .saturating_add(4);
    let start = row.x.saturating_add(row.width.saturating_sub(total) / 2);
    (start, start.saturating_add(total))
}

/// The arrow of a row that a click lands on: `Some(true)` for the next one, `Some(false)` for the
/// previous one.
fn arrow_at(row: Rect, name: &str, cell: (u16, u16)) -> Option<bool> {
    if !mouse::contains(row, cell) {
        return None;
    }
    let (start, end) = span(row, name);
    if cell.0 >= start && cell.0 < start.saturating_add(ARROW_HIT) {
        Some(false)
    } else if cell.0 < end && cell.0 >= end.saturating_sub(ARROW_HIT) {
        Some(true)
    } else {
        None
    }
}

/// Handles a mouse event on the docked window at `rect`: the wheel scrolls, the arrows of the tab
/// row and of the game row switch tabs and games, and Close closes the window (`Some`). Events
/// outside `rect` are not the window's.
pub fn handle_mouse(
    state: &mut DockState,
    modal: &mut Modal,
    rect: Rect,
    rules: Rules,
    event: &MouseEvent,
) -> Option<Effect> {
    let cell = (event.column, event.row);
    if !mouse::contains(rect, cell) {
        return None;
    }
    match mouse::wheel_action(event) {
        Some(Action::Up) => scroll(state, modal, rect, -WHEEL_LINES),
        Some(Action::Down) => scroll(state, modal, rect, WHEEL_LINES),
        _ => {}
    }
    let press = mouse::left_press(event)?;
    if mouse::contains(close_rect(rect), press) {
        return Some(Effect::None);
    }
    let places = parts(rect);
    if let Some(tab) = nav::tab_of(modal)
        && let Some(forward) = arrow_at(places.top, &t(tab.label_key()), press)
    {
        switch_tab(state, modal, rules, forward);
    }
    if let Some(game) = rules_of(modal)
        && let Some(forward) = arrow_at(places.games, &t(game.name_key()), press)
    {
        step(state, modal, rules, forward);
    }
    None
}
