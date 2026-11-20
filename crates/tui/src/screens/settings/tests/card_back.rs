use ratatui::crossterm::event::KeyCode;

use super::super::options::{Field, Step};
use super::{bare, changed, index_of, line_of, move_down, open_settings, row, value};
use crate::i18n::{Locale, set_locale, t};
use crate::settings::{CardBack, Settings};

#[test]
fn card_backs_cycle_both_ways() {
    let base = Settings::default();
    assert_eq!(base.card_back, CardBack::Theme);
    let monokai = changed(Field::CardBack, &base, Step::Next);
    assert_eq!(monokai.card_back, CardBack::Monokai);
    let dracula = changed(Field::CardBack, &monokai, Step::Next);
    assert_eq!(dracula.card_back, CardBack::Dracula);
    assert_eq!(
        changed(Field::CardBack, &dracula, Step::Next).card_back,
        CardBack::Theme
    );
    assert_eq!(
        changed(Field::CardBack, &base, Step::Previous).card_back,
        CardBack::Dracula
    );
    assert!(value(Field::CardBack, &base).starts_with("Theme "));
    assert!(value(Field::CardBack, &monokai).starts_with("Monokai "));
    assert!(value(Field::CardBack, &dracula).starts_with("Dracula "));
    assert!(row(Field::CardBack, &base, true).starts_with("Card back"));
    assert_eq!(
        Field::CardBack.group(),
        super::super::options::Group::Display
    );
}

#[test]
fn card_back_names_in_polish() {
    set_locale(Locale::PlPl);
    let base = Settings::default();
    assert!(value(Field::CardBack, &base).starts_with("Motyw "));
    let monokai = Settings {
        card_back: CardBack::Monokai,
        ..base.clone()
    };
    assert!(value(Field::CardBack, &monokai).starts_with("Monokai "));
    let dracula = Settings {
        card_back: CardBack::Dracula,
        ..base.clone()
    };
    assert!(value(Field::CardBack, &dracula).starts_with("Dracula "));
    assert_eq!(t("settings.row.card_back"), "Rewers kart");
    set_locale(Locale::EnUs);
}

#[test]
fn a_card_back_change_is_shown_at_once() {
    let mut h = open_settings();
    move_down(&mut h, index_of(Field::CardBack));
    h.press(KeyCode::Right);
    assert_eq!(h.app.settings().card_back, CardBack::Monokai);
    assert!(bare(&line_of(&mut h, "Card back")).contains("Monokai "));
}
