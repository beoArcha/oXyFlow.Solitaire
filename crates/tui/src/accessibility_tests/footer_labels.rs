//! Footer labels per screen, in both locales: each key line names the keys that work on its screen
//! in the words of that screen, and no key that does nothing there is drawn.

use oxysol_core::{DrawCount, Variant};
use ratatui::crossterm::event::KeyCode;

use crate::accessibility_tests::footer::{each_locale, harness, observed, sized};
use crate::app::Screen;
use crate::i18n::{Locale, tf};
use crate::screens::Effect;
use crate::test_support::Harness;

/// The drawn key line of a screen, one line per locale, as the player reads it.
fn expect_drawn(h: &mut Harness, labels: &[&str]) {
    let text = h.text();
    for label in labels {
        assert!(text.contains(label), "{label:?} is not drawn in {text}");
    }
}

#[test]
fn the_main_menu_names_only_the_keys_that_work_there() {
    each_locale(|locale| {
        let mut h = harness(locale, Vec::new());
        let open = if locale == Locale::PlPl {
            "Enter Otwórz"
        } else {
            "Enter Open"
        };
        let quit = if locale == Locale::PlPl {
            "q Zakończ"
        } else {
            "q Quit"
        };
        let help = if locale == Locale::PlPl {
            "? Pomoc"
        } else {
            "? Help"
        };
        expect_drawn(&mut h, &[open, quit, help]);
        assert!(
            !h.text().contains("Esc"),
            "{locale:?}: Esc is drawn in the item list"
        );
    });
}

#[test]
fn esc_in_the_main_menu_item_list_does_nothing() {
    let mut h = harness(Locale::EnUs, Vec::new());
    let before = observed(&mut h);
    h.press(KeyCode::Esc);
    assert_eq!(observed(&mut h), before);
}

#[test]
fn the_game_selection_and_settings_name_their_keys_in_their_own_words() {
    each_locale(|locale| {
        let (play, change, back, help) = if locale == Locale::PlPl {
            ("Enter Zagraj", "Enter Zmień", "Esc Wstecz", "? Pomoc")
        } else {
            ("Enter Play", "Enter Change", "Esc Back", "? Help")
        };
        let mut games = harness(locale, Vec::new());
        games.app.apply(Effect::Goto(Screen::GameSelect));
        expect_drawn(&mut games, &[play, back, help]);
        let mut settings = harness(locale, Vec::new());
        settings.app.apply(Effect::Goto(Screen::Settings));
        expect_drawn(&mut settings, &[change, back, help]);
    });
}

#[test]
fn the_board_names_its_keys_by_what_they_do_now() {
    each_locale(|locale| {
        let mut h = sized(locale, Vec::new(), (120, 40));
        h.app.apply(Effect::Start {
            variant: Variant::Klondike(DrawCount::One),
            seed: Some(7),
            undo_allowed: true,
        });
        let text = h.text();
        assert!(
            !text.contains("Enter Select") && !text.contains("Enter Wybierz"),
            "{locale:?}"
        );
        assert!(
            !text.contains("Esc Back") && !text.contains("Esc Wstecz"),
            "{locale:?}"
        );
        assert!(
            text.contains(&tf("board.action.menu", &[("key", "Esc")])),
            "{locale:?}"
        );
    });
}
