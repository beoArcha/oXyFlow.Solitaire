//! The groups of the settings screen (O-96): Process priority sits under its own Performance heading,
//! after Sound and before Hints and help.

use ratatui::crossterm::event::KeyCode;

use super::super::options::{FIELDS, Field, Group};
use super::{index_of, move_down, open_settings};
use crate::i18n::{Locale, set_locale, t};

#[test]
fn process_priority_is_the_only_row_of_the_performance_group_after_sound() {
    let position = |group| Group::ALL.iter().position(|g| *g == group);
    assert_eq!(Field::Priority.group(), Group::Performance);
    assert!(position(Group::Sound) < position(Group::Performance));
    assert!(position(Group::Performance) < position(Group::Hints));
    let rows: Vec<Field> = FIELDS
        .iter()
        .copied()
        .filter(|field| field.group() == Group::Performance)
        .collect();
    assert_eq!(rows, [Field::Priority]);
}

#[test]
fn the_performance_heading_sits_directly_above_the_priority_row_in_every_locale() {
    for locale in Locale::offered() {
        set_locale(locale);
        let mut h = open_settings();
        h.resize(100, 60);
        let lines = h.lines_without_pane();
        let heading = t("settings.group.performance").into_owned();
        let row = t(Field::Priority.label_key());
        let at = lines.iter().position(|line| line.trim() == heading);
        let priority = lines.iter().position(|line| line.contains(&*row));
        assert!(at.is_some(), "{locale:?}: no heading {heading:?}");
        assert_eq!(priority, at.map(|at| at + 1), "{locale:?}");
    }
    set_locale(Locale::EnUs);
}

#[test]
fn process_priority_still_previews_and_waits_for_confirmation() {
    let mut h = open_settings();
    move_down(&mut h, index_of(Field::Priority));
    let before = h.app.settings().process_priority;
    h.press(KeyCode::Right);
    assert_ne!(h.app.settings().process_priority, before);
    assert_eq!(h.storage.borrow().settings, None);
}
