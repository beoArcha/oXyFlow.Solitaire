//! The whole path menu -> game -> settings -> quit, for every game, by keyboard and by mouse (P3-09).

use super::*;
use crate::i18n::t;
use crate::screens::variant_label_key;
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, TRUE_COLOR};
use oxysol_core::Variant;
use ratatui::crossterm::event::KeyCode;

const MENU_ITEMS_AT_MOST: usize = 12;

fn focus_menu_item(h: &mut Harness, label: &str) {
    let marked = format!("► {label}");
    for _ in 0..MENU_ITEMS_AT_MOST {
        if h.text().contains(&marked) {
            return;
        }
        h.press(KeyCode::Down);
    }
}

fn click_text(h: &mut Harness, needle: &str) {
    let found = h.find(needle);
    assert!(found.is_some(), "{needle} is not drawn:\n{}", h.text());
    let (column, row) = found.unwrap_or_default();
    h.click(column, row);
}

/// Clicks the name of a variant in the game list. The name is found after the marks of its row
/// (the favourite mark, then the star or a blank), because a family heading can contain the same
/// word.
fn click_variant(h: &mut Harness, variant: Variant) {
    let label = t(variant_label_key(variant));
    let found = [format!("· ★ {label}"), format!("·   {label}")]
        .iter()
        .find_map(|needle| h.find(needle));
    assert!(found.is_some(), "{label} is not drawn: {}", h.text());
    let (column, row) = found.unwrap_or_default();
    h.click(column + 4, row);
}

fn board_variant(h: &Harness) -> Option<Variant> {
    h.app.board().session().map(|session| session.variant())
}

#[test]
fn every_game_goes_from_the_menu_through_the_settings_to_quit_by_keyboard() {
    for variant in Variant::ALL {
        let mut h = Harness::new();
        assert_eq!(h.app.screen(), Screen::MainMenu);

        h.press(KeyCode::Enter);
        assert_eq!(h.app.screen(), Screen::GameSelect, "{variant:?}");
        h.focus_game_variant(variant);
        h.press(KeyCode::Enter);
        assert_eq!(h.app.screen(), Screen::Board, "{variant:?}");
        assert_eq!(board_variant(&h), Some(variant));
        assert!(h.app.modal().is_none(), "{variant:?}");

        h.press(KeyCode::Esc);
        assert_eq!(h.app.screen(), Screen::MainMenu, "{variant:?}");
        assert!(h.text().contains(t("menu.resume").as_ref()), "{variant:?}");

        focus_menu_item(&mut h, &t("common.menu.settings"));
        h.press(KeyCode::Enter);
        assert_eq!(h.app.screen(), Screen::Settings, "{variant:?}");
        let before = h.app.settings().color_mode;
        h.press(KeyCode::BackTab);
        h.press(KeyCode::Right);
        assert_ne!(h.app.settings().color_mode, before, "{variant:?}");
        assert!(h.storage.borrow().settings.is_some(), "{variant:?}");
        h.press(KeyCode::Esc);
        assert_eq!(h.app.screen(), Screen::MainMenu, "{variant:?}");

        focus_menu_item(&mut h, &t("common.menu.quit"));
        h.press(KeyCode::Enter);
        assert!(h.app.should_quit(), "{variant:?}");
        assert!(
            h.storage
                .borrow()
                .game
                .as_ref()
                .is_some_and(|saved| saved.record.variant == variant),
            "{variant:?}: quitting keeps the game"
        );
    }
}

#[test]
fn every_game_goes_from_the_menu_through_the_settings_to_quit_by_mouse() {
    for variant in Variant::ALL {
        let mut h = Harness::with(MemoryStorage::new(), TRUE_COLOR, 160, 40);

        click_text(&mut h, &t("common.menu.new_game"));
        assert_eq!(h.app.screen(), Screen::GameSelect, "{variant:?}");
        click_variant(&mut h, variant);
        assert_eq!(h.app.screen(), Screen::Board, "{variant:?}");
        assert_eq!(board_variant(&h), Some(variant));

        click_text(&mut h, &format!("{}: ", t("help.key.esc")));
        assert_eq!(h.app.screen(), Screen::MainMenu, "{variant:?}");

        click_text(&mut h, &t("common.menu.settings"));
        assert_eq!(h.app.screen(), Screen::Settings, "{variant:?}");
        let before = h.app.settings().color_mode;
        click_text(&mut h, &t("settings.tab.audio_visual"));
        click_text(&mut h, &t("settings.row.colors"));
        assert_ne!(h.app.settings().color_mode, before, "{variant:?}");
        h.press(KeyCode::Esc);
        assert_eq!(h.app.screen(), Screen::MainMenu, "{variant:?}");

        click_text(&mut h, &t("common.menu.quit"));
        assert!(h.app.should_quit(), "{variant:?}");
    }
}
