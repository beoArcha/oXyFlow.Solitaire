use ratatui::crossterm::event::KeyCode;

use super::super::options::{Field, Step};
use super::super::tabs::Tab;
use super::{bare, changed, focus_in_tab, line_of, open_settings, row, value};
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
    assert!(value(Field::CardBack, &base) == "Theme");
    assert!(value(Field::CardBack, &monokai) == "Monokai");
    assert!(value(Field::CardBack, &dracula) == "Dracula");
    assert!(row(Field::CardBack, &base, true).starts_with("Card back"));
    assert!(Tab::AudioVisual.fields().contains(&Field::CardBack));
}

#[test]
fn card_back_names_in_polish() {
    set_locale(Locale::PlPl);
    let base = Settings::default();
    assert!(value(Field::CardBack, &base) == "Motyw");
    let monokai = Settings {
        card_back: CardBack::Monokai,
        ..base.clone()
    };
    assert!(value(Field::CardBack, &monokai) == "Monokai");
    let dracula = Settings {
        card_back: CardBack::Dracula,
        ..base.clone()
    };
    assert!(value(Field::CardBack, &dracula) == "Dracula");
    assert_eq!(t("settings.row.card_back"), "Rewers kart");
    set_locale(Locale::EnUs);
}

#[test]
fn a_card_back_change_is_shown_at_once() {
    let mut h = open_settings();
    focus_in_tab(&mut h, Tab::Popular, Field::CardBack);
    h.press(KeyCode::Right);
    assert_eq!(h.app.settings().card_back, CardBack::Monokai);
    assert!(bare(&line_of(&mut h, "Card back")).contains("Monokai"));
}
