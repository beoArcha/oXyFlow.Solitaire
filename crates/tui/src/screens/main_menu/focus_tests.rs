//! Tab and Shift+Tab on the main menu: the items, the seed field and its two buttons.

use ratatui::crossterm::event::KeyCode;

use crate::app::Screen;
use crate::i18n::Locale;
use crate::screens::focus::marked_text;
use crate::settings::Settings;
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, PLAIN};

const FIRST: &str = "New game";
const SEED_ITEM: &str = "New game from seed";
const STATISTICS: &str = "Statistics";
const QUIT: &str = "Quit";

fn focused(h: &mut Harness) -> Option<String> {
    marked_text(&h.lines(), "►")
}

fn seed_entry() -> Harness {
    let mut h = Harness::new();
    h.press(KeyCode::Tab);
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::MainMenu);
    h
}

#[test]
fn tab_moves_the_focus_down_the_items() {
    let mut h = Harness::new();
    assert_eq!(focused(&mut h).as_deref(), Some(FIRST));
    h.press(KeyCode::Tab);
    assert_eq!(focused(&mut h).as_deref(), Some(SEED_ITEM));
    h.press(KeyCode::Tab);
    assert_eq!(focused(&mut h).as_deref(), Some(STATISTICS));
}

#[test]
fn shift_tab_from_the_first_item_wraps_to_the_last() {
    let mut h = Harness::new();
    h.press(KeyCode::BackTab);
    assert_eq!(focused(&mut h).as_deref(), Some(QUIT));
    h.press(KeyCode::BackTab);
    assert_eq!(focused(&mut h).as_deref(), Some("Help"));
}

#[test]
fn tab_from_the_last_item_wraps_to_the_first() {
    let mut h = Harness::new();
    h.press(KeyCode::BackTab);
    h.press(KeyCode::Tab);
    assert_eq!(focused(&mut h).as_deref(), Some(FIRST));
}

#[test]
fn enter_and_space_activate_the_focused() {
    let mut h = Harness::new();
    h.press(KeyCode::Tab);
    h.press(KeyCode::Tab);
    h.type_char(' ');
    assert_eq!(h.app.screen(), Screen::Stats);
    h.press(KeyCode::Esc);
    h.press(KeyCode::BackTab);
    h.press(KeyCode::BackTab);
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::GameSelect);
}

#[test]
fn the_marker_sits_directly_before_the_focused_label() {
    let mut h = Harness::new();
    h.press(KeyCode::Tab);
    h.press(KeyCode::Tab);
    let lines = h.lines();
    assert!(lines.iter().any(|line| line.contains("► Statistics")));
    assert_eq!(lines.iter().filter(|line| line.contains('►')).count(), 1);
}

#[test]
fn tab_reaches_every_item_of_the_menu_once() {
    let mut h = Harness::new();
    let mut seen = Vec::new();
    for _ in 0..7 {
        seen.extend(focused(&mut h));
        h.press(KeyCode::Tab);
    }
    assert_eq!(seen.len(), 7, "{seen:?}");
    seen.sort();
    seen.dedup();
    assert_eq!(seen.len(), 7);
}

#[test]
fn tab_into_the_seed_entry_moves_from_the_field_to_the_buttons_and_back() {
    let mut h = seed_entry();
    assert!(focused(&mut h).is_some_and(|text| text.starts_with("[ _")));
    h.press(KeyCode::Tab);
    assert!(focused(&mut h).is_some_and(|text| text.starts_with("[ Continue ]")));
    h.press(KeyCode::Tab);
    assert_eq!(focused(&mut h).as_deref(), Some("[ Back ]"));
    h.press(KeyCode::Tab);
    assert!(focused(&mut h).is_some_and(|text| text.starts_with("[ _")));
}

#[test]
fn shift_tab_from_the_field_wraps_to_back() {
    let mut h = seed_entry();
    h.press(KeyCode::BackTab);
    assert_eq!(focused(&mut h).as_deref(), Some("[ Back ]"));
}

#[test]
fn typed_digits_go_to_the_field_only_while_it_has_the_focus() {
    let mut h = seed_entry();
    h.type_char('4');
    h.type_char('2');
    assert!(h.text().contains("[ 42_"));
    h.press(KeyCode::Tab);
    h.type_char('9');
    assert!(h.text().contains("[ 42 "));
    assert!(!h.text().contains("429"));
    h.press(KeyCode::BackTab);
    h.type_char('9');
    assert!(h.text().contains("[ 429_"));
}

#[test]
fn enter_on_continue_submits_the_seed_and_enter_again_deals_the_game() {
    let mut h = seed_entry();
    for digit in "771".chars() {
        h.type_char(digit);
    }
    h.press(KeyCode::Tab);
    h.press(KeyCode::Enter);
    assert!(h.text().contains("Choose a game for seed 771"));
    h.press(KeyCode::Enter);
    assert_eq!(h.app.board().session().map(|s| s.seed()), Some(771));
}

#[test]
fn space_presses_the_focused_continue_button() {
    let mut h = seed_entry();
    h.type_char('5');
    h.press(KeyCode::Tab);
    h.type_char(' ');
    assert!(h.text().contains("Choose a game for seed 5"));
}

#[test]
fn back_returns_to_the_menu_items_by_enter_and_by_space() {
    let mut h = seed_entry();
    h.press(KeyCode::Tab);
    h.press(KeyCode::Tab);
    h.press(KeyCode::Enter);
    assert!(h.text().contains(STATISTICS));
    assert_eq!(h.app.screen(), Screen::MainMenu);
    h.press(KeyCode::Enter);
    h.press(KeyCode::Tab);
    h.press(KeyCode::Tab);
    h.type_char(' ');
    assert!(h.text().contains(STATISTICS));
    assert_eq!(h.app.screen(), Screen::MainMenu);
}

#[test]
fn left_and_right_switch_between_the_two_buttons() {
    let mut h = seed_entry();
    h.press(KeyCode::Tab);
    h.press(KeyCode::Right);
    assert_eq!(focused(&mut h).as_deref(), Some("[ Back ]"));
    h.press(KeyCode::Left);
    assert!(focused(&mut h).is_some_and(|text| text.starts_with("[ Continue ]")));
}

#[test]
fn the_seed_entry_marks_its_focus_in_ascii_and_in_polish() {
    let storage = MemoryStorage {
        settings: Some(Settings {
            locale: Locale::PlPl,
            ..Settings::for_caps(&PLAIN)
        }),
        ..MemoryStorage::new()
    };
    let mut h = Harness::with(storage, PLAIN, 100, 30);
    h.press(KeyCode::Tab);
    h.press(KeyCode::Enter);
    assert!(h.text().contains("> [ _"));
    h.press(KeyCode::Tab);
    let continue_label = crate::i18n::t("menu.seed.continue").into_owned();
    assert!(h.text().contains(&format!("> [ {continue_label} ]")));
    assert!(!h.text().contains('►'));
}
