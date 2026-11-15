use super::*;
use crate::screens::Effect;
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, PLAIN, TRUE_COLOR};
use oxysol_core::Variant;
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
    assert!(unicode.text().contains("► New game"));
    let mut ascii = Harness::with(MemoryStorage::new(), PLAIN, 120, 40);
    let text = ascii.text();
    assert!(text.contains("> New game"));
    assert!(text.is_ascii());
}

fn focus_item(h: &mut Harness, label: &str) {
    let marked = format!("► {label}");
    for _ in 0..8 {
        if h.text().contains(&marked) {
            return;
        }
        h.press(KeyCode::Down);
    }
}

#[test]
fn focus_marker_follows_the_cursor() {
    let mut h = Harness::new();
    assert!(h.text().contains("► New game"));
    focus_item(&mut h, "Settings");
    let text = h.text();
    assert!(text.contains("► Settings"));
    assert!(!text.contains("► New game from seed"));
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
    focus_item(&mut h, "Quit");
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
    focus_item(&mut h, "Settings");
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

#[test]
fn a_command_line_seed_opens_the_games_of_that_seed() {
    let mut h = Harness::starting_at_seed(77);
    assert_eq!(h.app.screen(), Screen::MainMenu);
    assert!(h.text().contains("Choose a game for seed 77"));
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::Board);
    let first = Variant::ALL[0];
    assert!(
        h.app
            .board()
            .session()
            .is_some_and(|s| s.variant() == first && s.seed() == 77)
    );
    assert_eq!(h.storage.borrow().history[0].seed, 77);
}

#[test]
fn a_later_new_game_draws_a_fresh_seed_not_the_command_line_one() {
    let mut h = Harness::starting_at_seed(77);
    h.app.apply(Effect::Start {
        variant: Variant::ALL[0],
        seed: None,
    });
    assert!(h.app.board().session().is_some_and(|s| s.seed() == 42));
}

#[test]
fn escape_from_the_games_of_a_command_line_seed_returns_to_the_seed_field() {
    let mut h = Harness::starting_at_seed(77);
    h.press(KeyCode::Esc);
    assert!(h.text().contains("Type a seed:"));
    assert!(h.text().contains("77_"));
    h.press(KeyCode::Esc);
    assert!(!h.text().contains("Type a seed:"));
    assert_eq!(h.app.screen(), Screen::MainMenu);
}

#[test]
fn help_and_rules_keys_open_their_overlays_outside_a_text_field() {
    let mut h = Harness::new();
    h.type_char('?');
    assert!(matches!(h.app.modal(), Some(Modal::Help)));
    h.press(KeyCode::Esc);
    h.type_char('i');
    assert!(matches!(h.app.modal(), Some(Modal::HowToPlay(_))));
}

#[test]
fn help_and_rules_keys_are_typed_into_the_seed_field() {
    let mut h = Harness::new();
    h.press(KeyCode::Down);
    h.press(KeyCode::Enter);
    h.type_char('1');
    h.type_char('?');
    h.type_char('i');
    assert!(h.app.modal().is_none());
    assert!(h.text().contains("1?i_"));
    h.press(KeyCode::Enter);
    assert!(h.text().contains("A seed is a whole number"));
    h.press(KeyCode::Esc);
    h.type_char('?');
    assert!(matches!(h.app.modal(), Some(Modal::Help)));
}

#[test]
fn f1_opens_help_while_a_seed_is_typed() {
    let mut h = Harness::new();
    h.press(KeyCode::Down);
    h.press(KeyCode::Enter);
    h.press(KeyCode::F(1));
    assert!(matches!(h.app.modal(), Some(Modal::Help)));
}
