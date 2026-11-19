//! Footer buttons: every button drawn in a key line, on every screen that has one and in both
//! locales, does exactly what its key does when it is clicked, and the button under the pointer is
//! drawn reversed. Each button is found by its drawn text, so the test clicks what the player sees.

use oxysol_core::{DrawCount, Variant};
use ratatui::crossterm::event::{KeyCode, KeyModifiers, MouseEvent, MouseEventKind};
use ratatui::style::Modifier;

use crate::action::Action;
use crate::app::Screen;
use crate::i18n::{Locale, set_locale, t, tf};
use crate::input::{Input, primary_key, primary_key_event};
use crate::screens::Effect;
use crate::screens::game_select::FOOTER as SELECT_FOOTER;
use crate::screens::main_menu::FOOTER as MENU_FOOTER;
use crate::screens::settings::FOOTER as SETTINGS_FOOTER;
use crate::settings::Settings;
use crate::storage::Outcome;
use crate::storage::memory::{HistoryRow, MemoryStorage};
use crate::test_support::{Harness, TRUE_COLOR};

const SIZE: (u16, u16) = (100, 30);

fn harness(locale: Locale, history: Vec<HistoryRow>) -> Harness {
    sized(locale, history, SIZE)
}

/// A harness of the given size. The board is drawn wider, so that every button of its `Do:` row
/// fits in both locales.
fn sized(locale: Locale, history: Vec<HistoryRow>, size: (u16, u16)) -> Harness {
    let storage = MemoryStorage {
        settings: Some(Settings {
            locale,
            ..Settings::for_caps(&TRUE_COLOR)
        }),
        history,
        ..MemoryStorage::new()
    };
    Harness::with(storage, TRUE_COLOR, size.0, size.1)
}

fn played() -> HistoryRow {
    HistoryRow::finished(
        1,
        Variant::Klondike(DrawCount::One),
        7,
        Outcome::Won,
        60_000,
    )
}

/// What the player can observe: the screen, the modal, quitting and the drawn text.
fn observed(h: &mut Harness) -> String {
    let text = h.text();
    format!(
        "{:?}|{:?}|{}|{text}",
        h.app.screen(),
        h.app.modal(),
        h.app.should_quit()
    )
}

/// The drawn label of an action in a key line: its primary key and its label.
fn key_label(action: Action) -> String {
    let key = primary_key(action).unwrap_or_default();
    format!("{key} {}", t(action.label_key()))
}

/// Clicks each `(action, drawn text)` on a fresh harness made by `setup` and compares the result
/// with the same fresh harness after the primary key of the action.
fn assert_clicks_match_keys(
    locale: Locale,
    setup: &dyn Fn(Locale) -> Harness,
    buttons: &[(Action, String)],
    context: &str,
) {
    for (action, text) in buttons {
        let mut by_key = setup(locale);
        let mut by_click = setup(locale);
        let Some((column, row)) = by_click.find(text) else {
            panic!("{context} ({locale:?}): {text:?} is not drawn");
        };
        let Some(key) = primary_key_event(*action) else {
            panic!("{context}: {action:?} has no key");
        };
        by_key.send(Input::Key(key));
        by_click.click(column + 1, row);
        assert_eq!(
            observed(&mut by_click),
            observed(&mut by_key),
            "{context} ({locale:?}): clicking {text:?} differs from its key"
        );
    }
}

fn each_locale(mut check: impl FnMut(Locale)) {
    for locale in Locale::ALL {
        set_locale(locale);
        check(locale);
    }
    set_locale(Locale::EnUs);
}

fn buttons_of(actions: &[Action]) -> Vec<(Action, String)> {
    actions
        .iter()
        .map(|action| (*action, key_label(*action)))
        .collect()
}

#[test]
fn every_button_of_the_main_menu_does_what_its_key_does() {
    each_locale(|locale| {
        let setup = |locale| harness(locale, Vec::new());
        assert_clicks_match_keys(locale, &setup, &buttons_of(&MENU_FOOTER), "main menu");
    });
}

#[test]
fn every_button_of_the_game_selection_does_what_its_key_does() {
    each_locale(|locale| {
        let setup = |locale| {
            let mut h = harness(locale, Vec::new());
            h.app.apply(Effect::Goto(Screen::GameSelect));
            h
        };
        let buttons = buttons_of(&SELECT_FOOTER);
        assert_clicks_match_keys(locale, &setup, &buttons, "game selection");
    });
}

#[test]
fn every_button_of_the_games_of_a_seed_does_what_its_key_does() {
    each_locale(|locale| {
        let setup = |locale| {
            let mut h = harness(locale, Vec::new());
            h.press(KeyCode::Down);
            h.press(KeyCode::Enter);
            for c in "42".chars() {
                h.type_char(c);
            }
            h.press(KeyCode::Enter);
            h
        };
        let buttons = buttons_of(&SELECT_FOOTER);
        assert_clicks_match_keys(locale, &setup, &buttons, "games of a seed");
    });
}

#[test]
fn every_button_of_the_settings_does_what_its_key_does() {
    each_locale(|locale| {
        let setup = |locale| {
            let mut h = harness(locale, Vec::new());
            h.app.apply(Effect::Goto(Screen::Settings));
            h
        };
        let buttons = buttons_of(&SETTINGS_FOOTER);
        assert_clicks_match_keys(locale, &setup, &buttons, "settings");
    });
}

#[test]
fn every_button_of_the_statistics_does_what_its_key_does() {
    each_locale(|locale| {
        let setup = |locale| {
            let mut h = harness(locale, vec![played()]);
            h.app.apply(Effect::Goto(Screen::Stats));
            h
        };
        let buttons = [
            (Action::Select, t("stats.footer_replay").into_owned()),
            (Action::CopySeed, t("stats.footer_copy").into_owned()),
            (Action::Back, t("stats.footer_back").into_owned()),
        ];
        assert_clicks_match_keys(locale, &setup, &buttons, "statistics");
    });
}

#[test]
fn every_button_of_the_board_does_what_its_key_does() {
    each_locale(|locale| {
        let setup = |locale| {
            let mut h = sized(locale, Vec::new(), (160, 40));
            h.app.apply(Effect::Start {
                variant: Variant::Klondike(DrawCount::One),
                seed: Some(7),
                undo_allowed: true,
            });
            h
        };
        let board = [
            (Action::Help, "board.action.help"),
            (Action::Rules, "board.action.rules"),
            (Action::Guide, "board.action.guide"),
            (Action::Back, "board.action.menu"),
        ];
        let buttons: Vec<(Action, String)> = board
            .iter()
            .map(|(action, key)| {
                let name = primary_key(*action).unwrap_or_default();
                (*action, tf(key, &[("key", &name)]))
            })
            .collect();
        assert_clicks_match_keys(locale, &setup, &buttons, "board");
    });
}

#[test]
fn a_button_under_the_pointer_is_drawn_reversed_and_only_that_one() {
    let mut h = harness(Locale::EnUs, Vec::new());
    let text = key_label(Action::Quit);
    let Some((column, row)) = h.find(&text) else {
        panic!("{text:?} is not drawn");
    };
    let moved = |column: u16| {
        Input::Mouse(MouseEvent {
            kind: MouseEventKind::Moved,
            column,
            row,
            modifiers: KeyModifiers::NONE,
        })
    };
    let reversed = |h: &mut Harness, column: u16| {
        h.lines();
        h.terminal.backend().buffer()[(column, row)]
            .modifier
            .contains(Modifier::REVERSED)
    };
    assert!(!reversed(&mut h, column + 1));
    h.send(moved(column + 1));
    assert!(reversed(&mut h, column + 1));
    let Some((help_column, _)) = h.find(&key_label(Action::Help)) else {
        panic!("help is not drawn");
    };
    assert!(!reversed(&mut h, help_column + 1));
}
