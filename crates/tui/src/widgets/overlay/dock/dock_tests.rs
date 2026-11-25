use ratatui::crossterm::event::{
    KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::layout::Rect;

use super::super::{CommandLine, HowToPlay, Rules};
use super::dock_view;
use super::{self as dock, DockState};
use crate::app::Modal;

#[path = "keys_tests.rs"]
mod keys_tests;
use crate::input::action_for_key;
use crate::screens::Effect;
use crate::settings::DockSide;

const SCREEN_130: Rect = Rect::new(0, 0, 130, 40);
const SCREEN_200: Rect = Rect::new(0, 0, 200, 60);
const SCREEN_100: Rect = Rect::new(0, 0, 100, 30);

fn press(state: &mut DockState, modal: &mut Modal, rect: Rect, code: KeyCode) -> Option<Effect> {
    let key = KeyEvent::new(code, KeyModifiers::NONE);
    dock::handle_key(
        state,
        modal,
        rect,
        Rules::Klondike,
        &key,
        action_for_key(&key),
    )
}

fn mouse(kind: MouseEventKind, column: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind,
        column,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

fn click(column: u16, row: u16) -> MouseEvent {
    mouse(MouseEventKind::Down(MouseButton::Left), column, row)
}

fn dock_at(side: DockSide, screen: Rect) -> Rect {
    dock::split(screen, side).map_or(Rect::default(), |(rect, _)| rect)
}

#[test]
fn a_docked_window_takes_the_room_beyond_the_board_minimum() {
    assert_eq!(
        dock::split(SCREEN_130, DockSide::Right),
        Some((Rect::new(100, 0, 30, 40), Rect::new(0, 0, 100, 40)))
    );
    assert_eq!(
        dock::split(SCREEN_130, DockSide::Left),
        Some((Rect::new(0, 0, 30, 40), Rect::new(30, 0, 100, 40)))
    );
    assert_eq!(
        dock::split(SCREEN_200, DockSide::Right),
        Some((Rect::new(140, 0, 60, 60), Rect::new(0, 0, 140, 60)))
    );
}

#[test]
fn docking_falls_back_to_centre_on_a_narrow_screen_and_for_the_centre_side() {
    assert_eq!(dock::split(SCREEN_100, DockSide::Right), None);
    assert_eq!(dock::split(SCREEN_100, DockSide::Left), None);
    assert_eq!(dock::split(Rect::new(0, 0, 129, 40), DockSide::Right), None);
    assert_eq!(dock::split(SCREEN_200, DockSide::Centre), None);
}

#[test]
fn only_the_help_window_is_dockable() {
    assert!(dock_view::is_dockable(&Modal::help()));
    assert!(dock_view::is_dockable(&Modal::HowToPlay(HowToPlay::new(
        Rules::Klondike
    ))));
    assert!(dock_view::is_dockable(&Modal::CommandLine(
        CommandLine::new()
    )));
    assert!(!dock_view::is_dockable(&Modal::pause(true)));
    assert!(dock_view::centred_frame(&Modal::pause(true), SCREEN_130).is_none());
}

#[test]
fn the_placement_button_sits_on_the_top_border_and_is_clickable() {
    let frame = dock_view::centred_frame(&Modal::help(), SCREEN_130);
    let frame = frame.unwrap_or_default();
    let button = dock::button_rect(frame, DockSide::Centre);
    assert_eq!(button.y, frame.y);
    assert_eq!(button.right(), frame.right() - 1);
    assert!(dock::button_hit(
        frame,
        DockSide::Centre,
        (button.x, button.y)
    ));
    assert!(!dock::button_hit(
        frame,
        DockSide::Centre,
        (frame.x, frame.y + 5)
    ));
}

#[test]
fn every_wrapped_row_fits_the_docked_width() {
    let modals = [
        Modal::help(),
        Modal::HowToPlay(HowToPlay::new(Rules::Klondike)),
        Modal::HowToPlay(HowToPlay::new(Rules::Pyramid)),
        Modal::CommandLine(CommandLine::new()),
    ];
    for width in [28_u16, 58] {
        for modal in &modals {
            for row in dock_view::body(modal, width) {
                let used = row.lead.chars().count() + row.text.chars().count();
                assert!(used <= usize::from(width), "{used} > {width}: {row:?}");
            }
        }
    }
}

#[test]
fn the_keys_tab_is_one_column_that_needs_scrolling_at_thirty_columns() {
    let rows = dock_view::body(&Modal::help(), 28);
    assert!(rows.len() > 35, "{}", rows.len());
}

#[test]
fn end_and_home_reach_the_last_and_the_first_line_and_stop_there() {
    let rect = dock_at(DockSide::Right, SCREEN_130);
    let mut modal = Modal::help();
    let mut state = DockState::default();
    let max = dock::max_scroll(&modal, rect);
    assert!(max > 0);
    press(&mut state, &mut modal, rect, KeyCode::End);
    assert_eq!(state.scroll, max);
    press(&mut state, &mut modal, rect, KeyCode::Down);
    assert_eq!(state.scroll, max);
    press(&mut state, &mut modal, rect, KeyCode::Home);
    assert_eq!(state.scroll, 0);
    press(&mut state, &mut modal, rect, KeyCode::Up);
    assert_eq!(state.scroll, 0);
}

#[test]
fn tab_walks_the_three_tabs_and_wraps() {
    let rect = dock_at(DockSide::Right, SCREEN_130);
    let mut modal = Modal::help();
    let mut state = DockState::default();
    press(&mut state, &mut modal, rect, KeyCode::Tab);
    assert!(matches!(modal, Modal::HowToPlay(_)));
    press(&mut state, &mut modal, rect, KeyCode::Tab);
    assert!(matches!(modal, Modal::CommandLine(_)));
    press(&mut state, &mut modal, rect, KeyCode::Tab);
    assert!(matches!(modal, Modal::Help(_)));
}

#[test]
fn left_and_right_switch_games_on_the_rules_tab_and_tabs_elsewhere() {
    let rect = dock_at(DockSide::Right, SCREEN_130);
    let mut modal = Modal::help();
    let mut state = DockState::default();
    press(&mut state, &mut modal, rect, KeyCode::Right);
    assert!(matches!(modal, Modal::HowToPlay(ref s) if s.rules == Rules::Klondike));
    press(&mut state, &mut modal, rect, KeyCode::Right);
    assert!(matches!(modal, Modal::HowToPlay(ref s) if s.rules == Rules::ALL[1]));
    press(&mut state, &mut modal, rect, KeyCode::Left);
    assert!(matches!(modal, Modal::HowToPlay(ref s) if s.rules == Rules::Klondike));
}

#[test]
fn question_mark_and_i_jump_to_their_tabs_and_escape_or_enter_close() {
    let rect = dock_at(DockSide::Left, SCREEN_130);
    let mut modal = Modal::CommandLine(CommandLine::new());
    let mut state = DockState::default();
    let key = KeyEvent::new(KeyCode::Char('?'), KeyModifiers::SHIFT);
    dock::handle_key(
        &mut state,
        &mut modal,
        rect,
        Rules::Klondike,
        &key,
        action_for_key(&key),
    );
    assert!(matches!(modal, Modal::Help(_)));
    assert_eq!(
        press(&mut state, &mut modal, rect, KeyCode::Esc),
        Some(Effect::None)
    );
    assert_eq!(
        press(&mut state, &mut modal, rect, KeyCode::Enter),
        Some(Effect::None)
    );
}

#[test]
fn the_wheel_scrolls_only_when_the_pointer_is_over_the_window() {
    let rect = dock_at(DockSide::Right, SCREEN_130);
    let mut modal = Modal::help();
    let mut state = DockState::default();
    let wheel_down = MouseEventKind::ScrollDown;
    let over = mouse(wheel_down, rect.x + 2, rect.y + 5);
    dock::handle_mouse(&mut state, &mut modal, rect, Rules::Klondike, &over);
    assert_eq!(state.scroll, 3);
    let away = mouse(wheel_down, 10, 5);
    dock::handle_mouse(&mut state, &mut modal, rect, Rules::Klondike, &away);
    assert_eq!(state.scroll, 3);
}

#[test]
fn a_click_on_the_next_arrow_of_the_tab_row_switches_the_tab() {
    let rect = dock_at(DockSide::Right, SCREEN_130);
    let top = dock::parts(rect).top;
    let name_width = 4_u16;
    let end = top.x + (top.width - (name_width + 4)) / 2 + name_width + 4;
    let mut modal = Modal::help();
    let mut state = DockState::default();
    let outcome = dock::handle_mouse(
        &mut state,
        &mut modal,
        rect,
        Rules::Klondike,
        &click(end - 1, top.y),
    );
    assert_eq!(outcome, None);
    assert!(matches!(modal, Modal::HowToPlay(_)));
}

#[test]
fn a_click_on_close_closes_the_window() {
    let rect = dock_at(DockSide::Right, SCREEN_130);
    let close = dock::close_rect(rect);
    let mut modal = Modal::help();
    let mut state = DockState::default();
    let outcome = dock::handle_mouse(
        &mut state,
        &mut modal,
        rect,
        Rules::Klondike,
        &click(close.x, close.y),
    );
    assert_eq!(outcome, Some(Effect::None));
}
