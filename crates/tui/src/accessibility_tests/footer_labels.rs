//! Footer labels per screen, in both locales: each key line names the keys that work on its screen
//! in the words of that screen, and no key that does nothing there is drawn.

use oxysol_core::{DrawCount, Variant};
use ratatui::crossterm::event::KeyCode;

use crate::accessibility_tests::footer::{each_locale, harness, observed, sized};
use crate::action::Action;
use crate::app::Screen;
use crate::i18n::{Locale, t, tf};
use crate::input::primary_key;
use crate::screens::Effect;
use crate::test_support::Harness;

/// A key and its label as a key line draws them, in the current locale.
fn hint(action: Action, label: &str) -> String {
    format!("{} {}", primary_key(action).unwrap_or_default(), t(label))
}

/// The drawn key line of a screen, one line per locale, as the player reads it.
fn expect_drawn(h: &mut Harness, labels: &[&str]) {
    let text = h.text();
    for label in labels {
        assert!(text.contains(label), "{label:?} is not drawn in {text}");
    }
}

/// Whether `key` is drawn as a key name of its own, not as the start of a word such as "Esci".
fn names_key(text: &str, key: &str) -> bool {
    text.match_indices(key).any(|(at, _)| {
        let before = text[..at].chars().next_back();
        let after = text[at + key.len()..].chars().next();
        !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
    })
}

#[test]
fn the_main_menu_names_only_the_keys_that_work_there() {
    each_locale(|locale| {
        let mut h = harness(locale, Vec::new());
        let open = hint(Action::Select, "menu.footer.open");
        let quit = hint(Action::Quit, "help.action.quit");
        let help = hint(Action::Help, "help.action.help");
        expect_drawn(&mut h, &[&open, &quit, &help]);
        assert!(
            !names_key(&h.text(), "Esc"),
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
        let play = hint(Action::Select, "menu.footer.play");
        let change = hint(Action::Select, "settings.footer.change");
        let back = hint(Action::Back, "help.action.back");
        let help = hint(Action::Help, "help.action.help");
        let mut games = harness(locale, Vec::new());
        games.app.apply(Effect::Goto(Screen::GameSelect));
        expect_drawn(&mut games, &[&play, &back, &help]);
        let mut settings = harness(locale, Vec::new());
        settings.app.apply(Effect::Goto(Screen::Settings));
        expect_drawn(&mut settings, &[&change, &back, &help]);
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
