//! The Card face row (P3-33): the five tints in order, their localized names and the change shown at once.

use ratatui::crossterm::event::KeyCode;

use super::super::options::{Field, Group, Step};
use super::{bare, changed, index_of, line_of, move_down, open_settings, row, value};
use crate::i18n::{Locale, set_locale, t};
use crate::settings::{CardFaceTint, Settings};

#[test]
fn card_faces_cycle_both_ways_through_every_tint() {
    let base = Settings::default();
    assert_eq!(base.card_face, CardFaceTint::Theme);
    for (index, tint) in CardFaceTint::ALL.into_iter().enumerate() {
        let from = Settings {
            card_face: tint,
            ..base.clone()
        };
        let next = CardFaceTint::ALL[(index + 1) % CardFaceTint::ALL.len()];
        assert_eq!(changed(Field::CardFace, &from, Step::Next).card_face, next);
    }
    assert_eq!(
        changed(Field::CardFace, &base, Step::Previous).card_face,
        CardFaceTint::Dark
    );
    assert_eq!(Field::CardFace.group(), Group::Interface);
}

#[test]
fn each_tint_shows_its_localized_name_and_the_row_its_label() {
    let base = Settings::default();
    for tint in CardFaceTint::ALL {
        let settings = Settings {
            card_face: tint,
            ..base.clone()
        };
        assert_eq!(
            value(Field::CardFace, &settings),
            t(tint.label_key()).into_owned()
        );
    }
    assert!(row(Field::CardFace, &base, true).starts_with(&*t("settings.row.card_face")));
}

#[test]
fn card_face_names_follow_the_locale() {
    let base = Settings::default();
    set_locale(Locale::EnUs);
    let english = value(Field::CardFace, &base);
    set_locale(Locale::PlPl);
    let polish = value(Field::CardFace, &base);
    assert_eq!(polish, t("settings.card_face.theme").into_owned());
    assert_ne!(polish, english);
    set_locale(Locale::EnUs);
}

#[test]
fn a_card_face_change_is_shown_at_once() {
    let mut h = open_settings();
    move_down(&mut h, index_of(Field::CardFace));
    h.press(KeyCode::Right);
    assert_eq!(h.app.settings().card_face, CardFaceTint::White);
    let label = t("settings.row.card_face").into_owned();
    let line = line_of(&mut h, &label);
    assert!(bare(&line).contains(&*t("settings.card_face.white")));
}
