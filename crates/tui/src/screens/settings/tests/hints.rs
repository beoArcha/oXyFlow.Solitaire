//! The "Hints and help" group: the aids a new game starts with.

use ratatui::crossterm::event::KeyCode;

use super::super::options::{FIELDS, Field, Group};
use super::{line_of, move_down, open_settings};
use crate::i18n::{Locale, set_locale, t};
use crate::screens::Effect;
use crate::settings::{Aid, Settings};

fn index_of(aid: Aid) -> usize {
    FIELDS
        .iter()
        .position(|field| field.aid() == Some(aid))
        .unwrap_or_default()
}

#[test]
fn the_aid_rows_close_the_list_in_one_group_in_this_order() {
    let aids: Vec<Aid> = FIELDS.iter().filter_map(|field| field.aid()).collect();
    assert_eq!(aids, Aid::ALL);
    assert!(
        FIELDS
            .iter()
            .filter(|f| f.aid().is_some())
            .all(|f| f.group() == Group::Hints)
    );
    assert!(
        FIELDS
            .iter()
            .filter(|f| f.aid().is_none())
            .all(|f| f.group() != Group::Hints)
    );
    assert_eq!(FIELDS.last(), Some(&Field::AutoFinish));
}

#[test]
fn the_group_is_named_in_both_locales_and_the_old_board_heading_is_gone() {
    let mut h = open_settings();
    let text = h.text();
    assert!(text.contains("Hints and help"), "{text}");
    assert!(!text.contains("Board"), "{text}");
    h.app.apply(Effect::Apply(Settings {
        locale: Locale::PlPl,
        ..Settings::default()
    }));
    assert!(h.text().contains("Podpowiedzi i pomoc"), "{}", h.text());
    set_locale(Locale::EnUs);
}

#[test]
fn every_aid_row_switches_by_key_and_by_click_and_is_saved() {
    for aid in Aid::ALL {
        let expected = aid.toggled(&Settings::default().aids());
        let mut h = open_settings();
        h.resize(140, 40);
        move_down(&mut h, index_of(aid));
        h.press(KeyCode::Enter);
        assert_eq!(h.app.settings().aids(), expected, "{aid:?} by key");
        let saved = h.storage.borrow().settings.clone();
        assert!(saved.is_some_and(|s| s.aids() == expected), "{aid:?} saved");
        h.press(KeyCode::Left);
        assert_eq!(
            h.app.settings().aids(),
            Settings::default().aids(),
            "{aid:?}"
        );

        let label = t(aid.label_key()).into_owned();
        let Some((column, row)) = h.find(&format!("{label} .")) else {
            panic!("{label} is not drawn");
        };
        h.click(column + 1, row);
        assert_eq!(h.app.settings().aids(), expected, "{aid:?} by click");
    }
}

#[test]
fn the_focused_aid_row_is_explained_in_both_locales() {
    for locale in Locale::ALL {
        for aid in Aid::ALL {
            let mut h = open_settings();
            h.app.apply(Effect::Apply(Settings {
                locale,
                ..Settings::default()
            }));
            move_down(&mut h, index_of(aid));
            let text = h.text();
            assert!(
                text.contains(t(aid.description_key()).as_ref()),
                "{locale:?} {aid:?}"
            );
        }
    }
    set_locale(Locale::EnUs);
}

#[test]
fn target_marks_and_the_moves_panel_have_their_own_rows() {
    let mut h = open_settings();
    h.resize(140, 40);
    assert!(line_of(&mut h, "Target marks").ends_with("On"));
    assert!(line_of(&mut h, "Possible moves panel").ends_with("On"));
    move_down(&mut h, index_of(Aid::TargetMarks));
    h.press(KeyCode::Enter);
    assert!(!h.app.settings().target_marks);
    assert!(line_of(&mut h, "Target marks").ends_with("Off"));
}
