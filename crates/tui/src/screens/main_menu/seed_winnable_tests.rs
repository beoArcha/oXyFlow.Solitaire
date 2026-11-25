//! The winnable-deals toggle is not offered in the game list of "New game from seed" or in the games
//! of a command-line seed, where a seed is dealt as chosen; the normal New game list keeps it.

use ratatui::crossterm::event::KeyCode;

use crate::i18n::t;
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, PLAIN};

fn offers_toggle(text: &str) -> bool {
    text.contains(&t("menu.footer.winnable").into_owned())
        || text.contains(&t("menu.winnable.off_hint").into_owned())
        || text.contains(&t("menu.winnable.on_hint").into_owned())
}

fn winnable_on(winnable_only: bool) -> MemoryStorage {
    MemoryStorage {
        settings: Some(crate::settings::Settings {
            winnable_only,
            ..crate::settings::Settings::for_caps(&PLAIN)
        }),
        ..MemoryStorage::new()
    }
}

#[test]
fn the_game_list_of_new_game_from_seed_has_no_winnable_toggle() {
    let mut h = Harness::with(winnable_on(true), PLAIN, 100, 40);
    h.press(KeyCode::Down);
    h.press(KeyCode::Enter);
    assert!(!offers_toggle(&h.text()), "{}", h.text());
}

#[test]
fn the_key_w_does_not_change_the_setting_in_the_seed_flow() {
    let mut h = Harness::with(winnable_on(false), PLAIN, 100, 40);
    h.press(KeyCode::Down);
    h.press(KeyCode::Enter);
    h.type_char('w');
    assert!(!h.app.settings().winnable_only);
}

#[test]
fn the_games_of_a_command_line_seed_keep_the_winnable_toggle() {
    let mut h = Harness::starting_at_seed(77);
    assert!(offers_toggle(&h.text()), "{}", h.text());
}

#[test]
fn the_normal_new_game_list_keeps_the_winnable_toggle() {
    let mut h = Harness::with(winnable_on(false), PLAIN, 100, 40);
    h.press(KeyCode::Enter);
    let text = h.text();
    assert!(
        text.contains(&t("menu.footer.winnable").into_owned()),
        "{text}"
    );
    assert!(
        text.contains(&t("menu.winnable.off_hint").into_owned()),
        "{text}"
    );
}

#[test]
fn the_normal_new_game_list_toggles_with_w_as_before() {
    let mut h = Harness::with(winnable_on(false), PLAIN, 100, 40);
    h.press(KeyCode::Enter);
    h.type_char('w');
    assert!(h.app.settings().winnable_only);
}
