use oxysol_core::{SuitCount, Variant};
use ratatui::crossterm::event::KeyCode;

use crate::app::Screen;
use crate::settings::Settings;
use crate::storage::Outcome;
use crate::storage::memory::{HistoryRow, MemoryStorage};
use crate::test_support::{Harness, TRUE_COLOR};

const UNDO_ON: &str = "[ Undo: on ]";
const UNDO_OFF: &str = "[ Undo: off ]";

fn select() -> Harness {
    let mut h = Harness::new();
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::GameSelect);
    h
}

#[test]
fn the_hint_line_follows_the_focused_variant_and_names_its_difficulty() {
    let mut h = select();
    assert!(h.text().contains("Easy: Build the foundations"));
    let spider_four = Variant::ALL
        .iter()
        .position(|variant| *variant == Variant::Spider(SuitCount::Four))
        .unwrap_or_default();
    for _ in 0..spider_four {
        h.press(KeyCode::Down);
    }
    assert!(
        h.text()
            .contains("Hard: Build eight runs from king down to ace")
    );
}

#[test]
fn the_toggle_is_visible_and_starts_on() {
    let mut h = select();
    assert!(h.text().contains(UNDO_ON));
    assert!(h.text().contains("u or a click switches undo"));
}

#[test]
fn the_u_key_switches_the_mode_and_it_is_remembered_in_the_settings() {
    let mut h = select();
    h.type_char('u');
    assert!(h.text().contains(UNDO_OFF));
    assert!(!h.app.settings().undo_allowed);
    assert!(
        h.storage
            .borrow()
            .settings
            .as_ref()
            .is_some_and(|settings| !settings.undo_allowed)
    );
    h.type_char('u');
    assert!(h.text().contains(UNDO_ON));
    assert!(h.app.settings().undo_allowed);
}

#[test]
fn a_click_on_the_toggle_switches_the_mode() {
    let mut h = select();
    let (column, row) = h.find(UNDO_ON).unwrap_or_default();
    h.click(column + 2, row);
    assert!(h.text().contains(UNDO_OFF));
    assert!(!h.app.settings().undo_allowed);
    assert_eq!(h.app.screen(), Screen::GameSelect);
}

#[test]
fn the_chosen_mode_reaches_the_new_game_and_the_history() {
    let mut h = select();
    h.type_char('u');
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::Board);
    assert!(!h.app.board().undo_allowed());
    assert!(!h.storage.borrow().history[0].undo_allowed);
}

#[test]
fn the_default_mode_allows_undo() {
    let mut h = select();
    h.press(KeyCode::Enter);
    assert!(h.app.board().undo_allowed());
    assert!(h.storage.borrow().history[0].undo_allowed);
}

#[test]
fn the_last_choice_is_offered_again_on_the_next_start() {
    let storage = MemoryStorage {
        settings: Some(Settings {
            undo_allowed: false,
            ..Settings::default()
        }),
        ..MemoryStorage::default()
    };
    let mut h = Harness::with(storage, TRUE_COLOR, 120, 40);
    h.press(KeyCode::Enter);
    assert!(h.text().contains(UNDO_OFF));
}

#[test]
fn a_game_dealt_from_a_command_line_seed_uses_the_chosen_mode() {
    let mut h = Harness::starting_at_seed(7);
    assert!(h.text().contains(UNDO_ON));
    h.type_char('u');
    h.press(KeyCode::Enter);
    assert!(h.app.board().session().is_some_and(|s| s.seed() == 7));
    assert!(!h.app.board().undo_allowed());
    assert!(!h.storage.borrow().history[0].undo_allowed);
}

#[test]
fn a_game_dealt_from_a_typed_seed_uses_the_chosen_mode() {
    let mut h = Harness::new();
    h.press(KeyCode::Down);
    h.press(KeyCode::Enter);
    h.type_char('4');
    h.type_char('2');
    h.press(KeyCode::Enter);
    h.type_char('u');
    h.press(KeyCode::Enter);
    assert!(!h.app.board().undo_allowed());
    assert_eq!(h.storage.borrow().history[0].seed, 42);
}

#[test]
fn replaying_a_game_from_the_statistics_uses_the_mode_of_that_game() {
    let mut row = HistoryRow::finished(1, Variant::FreeCell, 99, Outcome::Won, 60_000);
    row.undo_allowed = false;
    let storage = MemoryStorage {
        history: vec![row],
        ..MemoryStorage::default()
    };
    let mut h = Harness::with(storage, TRUE_COLOR, 120, 40);
    h.press(KeyCode::Down);
    h.press(KeyCode::Down);
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::Stats);
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::Board);
    assert!(h.app.settings().undo_allowed);
    assert!(!h.app.board().undo_allowed());
    assert_eq!(h.storage.borrow().history.len(), 2);
    assert!(!h.storage.borrow().history[1].undo_allowed);
}
