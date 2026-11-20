//! Tab and Shift+Tab on the statistics screen: the history list, then the buttons, wrapping at both
//! ends. Enter and Space press the focused element.

use ratatui::crossterm::event::KeyCode;

use super::support::{open_stats, three_games};
use crate::app::Screen;
use crate::screens::focus::marked_text;
use crate::storage::memory::MemoryStorage;
use crate::test_support::Harness;

const REPLAY: &str = "[ Play this seed again ]";
const COPY: &str = "[ Copy seed ]";
const CHECK: &str = "[ Check ]";
const CHECK_ALL: &str = "[ Check all ]";
const BACK: &str = "[ Back ]";

fn focused(h: &mut Harness) -> Option<String> {
    marked_text(&h.lines(), "►")
}

fn on_list(text: Option<String>) -> bool {
    text.is_some_and(|text| text.contains("33"))
}

fn on_button(text: Option<String>, label: &str) -> bool {
    text.is_some_and(|text| text.starts_with(label))
}

#[test]
fn the_newest_game_has_the_focus_at_first() {
    let mut h = open_stats(three_games(), 100, 30);
    assert!(on_list(focused(&mut h)));
}

#[test]
fn tab_walks_the_list_then_the_buttons_and_wraps_to_the_list() {
    let mut h = open_stats(three_games(), 100, 30);
    h.press(KeyCode::Tab);
    assert!(on_button(focused(&mut h), REPLAY));
    h.press(KeyCode::Tab);
    assert!(on_button(focused(&mut h), COPY));
    h.press(KeyCode::Tab);
    assert!(on_button(focused(&mut h), CHECK));
    h.press(KeyCode::Tab);
    assert!(on_button(focused(&mut h), CHECK_ALL));
    h.press(KeyCode::Tab);
    assert_eq!(focused(&mut h).as_deref(), Some(BACK));
    h.press(KeyCode::Tab);
    assert!(on_list(focused(&mut h)));
}

#[test]
fn shift_tab_from_the_list_wraps_to_back() {
    let mut h = open_stats(three_games(), 100, 30);
    h.press(KeyCode::BackTab);
    assert_eq!(focused(&mut h).as_deref(), Some(BACK));
    h.press(KeyCode::BackTab);
    assert!(on_button(focused(&mut h), CHECK_ALL));
}

#[test]
fn enter_on_the_list_and_on_play_again_both_replay_the_selected_seed() {
    let mut h = open_stats(three_games(), 100, 30);
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::Board);
    assert_eq!(h.app.board().session().map(|s| s.seed()), Some(33));

    let mut h = open_stats(three_games(), 100, 30);
    h.press(KeyCode::Tab);
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::Board);
    assert_eq!(h.app.board().session().map(|s| s.seed()), Some(33));
}

#[test]
fn space_on_copy_seed_copies_the_selected_seed() {
    let mut h = open_stats(three_games(), 100, 30);
    h.press(KeyCode::Tab);
    h.press(KeyCode::Tab);
    h.type_char(' ');
    assert_eq!(h.clipboard.borrow().copies, vec!["33".to_owned()]);
    assert_eq!(h.app.screen(), Screen::Stats);
}

#[test]
fn enter_on_back_returns_to_the_menu() {
    let mut h = open_stats(three_games(), 100, 30);
    h.press(KeyCode::BackTab);
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::MainMenu);
}

#[test]
fn down_takes_the_focus_back_to_the_list_and_moves_the_selection() {
    let mut h = open_stats(three_games(), 100, 30);
    h.press(KeyCode::Tab);
    h.press(KeyCode::Down);
    assert!(focused(&mut h).is_some_and(|text| text.contains("22")));
}

#[test]
fn without_games_the_list_and_back_are_the_only_stops() {
    let mut h = open_stats(MemoryStorage::new(), 100, 30);
    h.press(KeyCode::Tab);
    assert_eq!(focused(&mut h).as_deref(), Some(BACK));
    h.press(KeyCode::Tab);
    assert_eq!(focused(&mut h), None);
}
