use oxysol_core::{Variant, YukonRules};
use ratatui::crossterm::event::KeyCode;

use crate::app::Screen;
use crate::i18n::Locale;
use crate::settings::Settings;
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, TRUE_COLOR};

const OFF: &str = "[ Winnable deals only: off ]";
const ON: &str = "[ Winnable deals only: on ]";
const SLOW: &str = "(the search can take long)";
const NO_SEARCH: &str = "(no winnable-deal search yet: a random deal)";

fn select() -> Harness {
    let mut h = Harness::new();
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::GameSelect);
    h
}

fn stored_winnable(h: &Harness) -> bool {
    h.storage
        .borrow()
        .settings
        .as_ref()
        .is_some_and(|settings| settings.winnable_only)
}

#[test]
fn the_option_is_visible_and_starts_off() {
    let mut h = select();
    assert!(h.text().contains(OFF));
    assert!(h.text().contains("a typed seed is dealt as typed"));
}

#[test]
fn enter_on_the_focused_toggle_switches_it_and_it_is_remembered() {
    let mut h = select();
    h.press(KeyCode::Tab);
    h.press(KeyCode::Tab);
    h.press(KeyCode::Enter);
    assert!(h.text().contains(ON));
    assert!(h.app.settings().winnable_only);
    assert!(stored_winnable(&h));
    assert_eq!(h.app.screen(), Screen::GameSelect);
    h.type_char(' ');
    assert!(h.text().contains(OFF));
    assert!(!stored_winnable(&h));
}

#[test]
fn a_click_on_the_toggle_switches_it_without_starting_a_game() {
    let mut h = select();
    let (column, row) = h.find(OFF).unwrap_or_default();
    h.click(column + 4, row);
    assert!(h.text().contains(ON));
    assert!(stored_winnable(&h));
    assert_eq!(h.app.screen(), Screen::GameSelect);
    assert!(h.storage.borrow().history.is_empty());
}

#[test]
fn the_last_choice_is_offered_again_on_the_next_start() {
    let storage = MemoryStorage {
        settings: Some(Settings {
            winnable_only: true,
            ..Settings::default()
        }),
        ..MemoryStorage::default()
    };
    let mut h = Harness::with(storage, TRUE_COLOR, 120, 40);
    h.press(KeyCode::Enter);
    assert!(h.text().contains(ON));
}

#[test]
fn the_variants_that_are_rarely_proven_say_the_search_can_take_long() {
    let mut h = select();
    assert!(!h.text().contains(SLOW));
    h.press(KeyCode::Tab);
    h.press(KeyCode::Tab);
    h.press(KeyCode::Enter);
    let lines = h.lines();
    let noted: Vec<&String> = lines.iter().filter(|line| line.contains(SLOW)).collect();
    assert_eq!(noted.len(), 3, "{lines:#?}");
    let names = ["Spider, 2 suits", "Spider, 4 suits", "Pyramid, Classic"];
    for name in names {
        assert!(noted.iter().any(|line| line.contains(name)), "{name}");
    }
}

#[test]
fn a_game_without_a_solver_says_so_and_is_dealt_at_once_without_a_search() {
    let storage = MemoryStorage {
        settings: Some(Settings {
            winnable_only: true,
            ..Settings::default()
        }),
        ..MemoryStorage::default()
    };
    let mut h = Harness::with(storage, TRUE_COLOR, 120, 40);
    h.press(KeyCode::Enter);
    let lines = h.lines();
    let noted: Vec<&String> = lines
        .iter()
        .filter(|line| line.contains(NO_SEARCH))
        .collect();
    assert_eq!(noted.len(), 3, "{lines:#?}");
    assert!(noted[0].contains("Yukon"));
    assert!(noted[1].contains("Russian Solitaire"));
    assert!(noted[2].contains("Alaska"));
    let yukon = Variant::Yukon(YukonRules::Classic);
    let index = Variant::ALL.iter().position(|v| *v == yukon).unwrap_or(0);
    for _ in 0..index {
        h.press(KeyCode::Down);
    }
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::Board);
    assert!(h.app.modal().is_none());
    assert_eq!(h.app.board().session().map(|s| s.variant()), Some(yukon));
    assert!(!h.text().contains("Winnable:"));
}

#[test]
fn the_polish_texts_exist_for_the_option_and_the_note() {
    let storage = MemoryStorage {
        settings: Some(Settings {
            locale: Locale::PlPl,
            winnable_only: true,
            ..Settings::default()
        }),
        ..MemoryStorage::default()
    };
    let mut h = Harness::with(storage, TRUE_COLOR, 120, 40);
    h.press(KeyCode::Enter);
    let text = h.text();
    assert!(text.contains("[ Tylko do wygrania: włączone ]"), "{text}");
    assert!(text.contains("szukanie może długo trwać"), "{text}");
}
