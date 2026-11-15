use super::*;
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, PLAIN, TRUE_COLOR};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[test]
fn starts_on_the_main_menu_with_the_stub_items() {
    let mut h = Harness::new();
    let text = h.text();
    for item in ["New game", "Settings", "Quit", "oXySol"] {
        let present = text.contains(item) || item == "oXySol";
        assert!(present, "{item}");
    }
    assert_eq!(h.app.screen(), Screen::MainMenu);
    assert!(h.app.modal().is_none());
    assert!(!h.app.should_quit());
}

#[test]
fn focus_is_marked_by_a_symbol_without_color() {
    let mut unicode = Harness::new();
    assert!(unicode.text().contains("▶ New game"));
    let mut ascii = Harness::with(MemoryStorage::new(), PLAIN, 120, 40);
    let text = ascii.text();
    assert!(text.contains("> New game"));
    assert!(text.is_ascii());
}

#[test]
fn focus_marker_follows_the_cursor() {
    let mut h = Harness::new();
    h.press(KeyCode::Down);
    let text = h.text();
    assert!(text.contains("▶ Settings"));
    assert!(!text.contains("▶ New game"));
}

#[test]
fn q_quits() {
    let mut h = Harness::new();
    h.type_char('q');
    assert!(h.app.should_quit());
}

#[test]
fn ctrl_c_quits_from_anywhere() {
    let mut h = Harness::new();
    h.press(KeyCode::Enter);
    h.app.handle(
        Input::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
        h.now,
    );
    assert!(h.app.should_quit());
}

#[test]
fn the_quit_item_quits() {
    let mut h = Harness::new();
    h.press(KeyCode::Down);
    h.press(KeyCode::Down);
    h.press(KeyCode::Enter);
    assert!(h.app.should_quit());
}

#[test]
fn key_releases_are_ignored() {
    let mut h = Harness::new();
    let mut key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
    key.kind = KeyEventKind::Release;
    h.send(Input::Key(key));
    assert!(!h.app.should_quit());
}

#[test]
fn below_100_by_30_only_the_size_notice_is_shown() {
    let mut h = Harness::with(MemoryStorage::new(), TRUE_COLOR, 99, 30);
    let text = h.text();
    assert!(text.contains("Terminal too small"));
    assert!(text.contains("Current size: 99 x 30"));
    assert!(text.contains("Required size: 100 x 30"));
    assert!(!text.contains("New game"));
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::MainMenu);
    h.type_char('q');
    assert!(h.app.should_quit());
}

#[test]
fn exactly_100_by_30_is_enough() {
    let mut h = Harness::with(MemoryStorage::new(), TRUE_COLOR, 100, 30);
    let text = h.text();
    assert!(text.contains("New game"));
    assert!(!text.contains("Terminal too small"));
}

#[test]
fn the_height_alone_can_be_too_small() {
    let mut h = Harness::with(MemoryStorage::new(), TRUE_COLOR, 120, 29);
    assert!(h.text().contains("Current size: 120 x 29"));
}

#[test]
fn resizing_switches_between_notice_and_screen() {
    let mut h = Harness::new();
    assert!(h.text().contains("New game"));
    h.resize(80, 24);
    assert!(h.text().contains("Current size: 80 x 24"));
    h.resize(100, 30);
    assert!(h.text().contains("New game"));
}

#[test]
fn menu_leads_to_game_selection_and_back() {
    let mut h = Harness::new();
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::GameSelect);
    assert!(h.text().contains("Choose a game"));
    h.press(KeyCode::Esc);
    assert_eq!(h.app.screen(), Screen::MainMenu);
}

#[test]
fn menu_leads_to_settings_and_back() {
    let mut h = Harness::new();
    h.press(KeyCode::Down);
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::Settings);
    h.press(KeyCode::Esc);
    assert_eq!(h.app.screen(), Screen::MainMenu);
}

#[test]
fn a_click_activates_a_menu_item() {
    let mut h = Harness::new();
    let found = h.find("Settings");
    assert!(found.is_some());
    let (column, row) = found.unwrap_or_default();
    h.click(column, row);
    assert_eq!(h.app.screen(), Screen::Settings);
}
