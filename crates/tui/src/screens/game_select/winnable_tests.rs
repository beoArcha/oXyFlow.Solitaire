use oxysol_core::{Variant, YukonRules};
use ratatui::crossterm::event::KeyCode;

use crate::app::Screen;
use crate::i18n::{Locale, set_locale, t, tf};
use crate::settings::Settings;
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, TRUE_COLOR};

fn button(state: &str) -> String {
    tf("menu.winnable.button", &[("key", "w"), ("state", state)])
}

fn off() -> String {
    button(&t("common.select.off"))
}

fn on() -> String {
    button(&t("common.select.on"))
}
const SLOW: &str = "(the search can take long)";

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
    assert!(h.text().contains(&off()));
    assert!(h.text().contains(t("menu.winnable.off_hint").as_ref()));
}

#[test]
fn enter_on_the_focused_toggle_switches_it_and_it_is_remembered() {
    let mut h = select();
    h.press(KeyCode::Tab);
    h.press(KeyCode::Tab);
    h.press(KeyCode::Enter);
    assert!(h.text().contains(&on()));
    assert!(h.app.settings().winnable_only);
    assert!(stored_winnable(&h));
    assert_eq!(h.app.screen(), Screen::GameSelect);
    h.type_char(' ');
    assert!(h.text().contains(&off()));
    assert!(!stored_winnable(&h));
}

#[test]
fn a_click_on_the_toggle_switches_it_without_starting_a_game() {
    let mut h = select();
    let (column, row) = h.find(&off()).unwrap_or_default();
    h.click(column + 4, row);
    assert!(h.text().contains(&on()));
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
    assert!(h.text().contains(&on()));
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
    assert_eq!(noted.len(), 2, "{lines:#?}");
    let names = ["Pyramid, Classic", "Russian Solitaire"];
    for name in names {
        assert!(noted.iter().any(|line| line.contains(name)), "{name}");
    }
}

#[test]
fn every_yukon_variant_searches_for_a_winnable_deal_and_deals_it_marked() {
    let family = [
        YukonRules::Classic,
        YukonRules::Russian,
        YukonRules::Alaska,
        YukonRules::Moosehide,
        YukonRules::Australian,
    ];
    for rules in family {
        let storage = MemoryStorage {
            settings: Some(Settings {
                winnable_only: true,
                ..Settings::default()
            }),
            ..MemoryStorage::default()
        };
        let mut h = Harness::with(storage, TRUE_COLOR, 120, 40);
        h.solver.borrow_mut().accept_nth = Some(2);
        h.press(KeyCode::Enter);
        let variant = Variant::Yukon(rules);
        h.focus_game_variant(variant);
        h.press(KeyCode::Enter);
        let submitted = h.solver.borrow().submitted.clone();
        assert!(!submitted.is_empty(), "{rules:?}");
        assert!(submitted.iter().all(|query| query.variant == variant));
        assert_eq!(h.app.screen(), Screen::Board, "{rules:?}");
        assert_eq!(h.app.board().session().map(|s| s.variant()), Some(variant));
        assert!(h.text().contains("Winnable: yes"), "{rules:?}");
    }
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
    set_locale(Locale::PlPl);
    let mut h = Harness::with(storage, TRUE_COLOR, 120, 40);
    h.press(KeyCode::Enter);
    let text = h.text();
    assert!(text.contains(&on()), "{text}");
    assert!(text.contains("szukanie może długo trwać"), "{text}");
}
