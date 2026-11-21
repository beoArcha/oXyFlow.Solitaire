//! Tab and Shift+Tab on the settings screen: the rows, wrapping at both ends, and Space on a row.

use ratatui::crossterm::event::KeyCode;

use super::{index_of, settings_menu_index};
use crate::app::Screen;
use crate::i18n::t;
use crate::screens::focus::marked_text;
use crate::screens::settings::options::{FIELDS, Field};
use crate::test_support::Harness;

fn open() -> Harness {
    let mut h = Harness::new();
    for _ in 0..settings_menu_index() {
        h.press(KeyCode::Tab);
    }
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::Settings);
    h
}

fn focused(h: &mut Harness) -> Option<String> {
    marked_text(&h.lines_without_pane(), "►")
}

fn is_row(text: Option<String>, index: usize) -> bool {
    let label = t(FIELDS[index].label_key()).into_owned();
    text.is_some_and(|text| text.starts_with(label.as_str()))
}

#[test]
fn tab_moves_the_focus_down_the_rows_one_at_a_time() {
    let mut h = open();
    assert!(is_row(focused(&mut h), 0));
    h.press(KeyCode::Tab);
    assert!(is_row(focused(&mut h), 1));
    h.press(KeyCode::Tab);
    assert!(is_row(focused(&mut h), 2));
}

#[test]
fn tab_from_the_last_row_wraps_to_the_first() {
    let mut h = open();
    for index in 0..FIELDS.len() {
        assert!(is_row(focused(&mut h), index), "row {index}");
        h.press(KeyCode::Tab);
    }
    assert!(is_row(focused(&mut h), 0));
}

#[test]
fn shift_tab_from_the_first_row_wraps_to_the_last() {
    let mut h = open();
    h.press(KeyCode::BackTab);
    assert!(is_row(focused(&mut h), FIELDS.len() - 1));
    h.press(KeyCode::BackTab);
    assert!(is_row(focused(&mut h), FIELDS.len() - 2));
}

#[test]
fn space_changes_the_focused_row_forward_like_enter() {
    let mut h = open();
    let theme = index_of(Field::Theme);
    for _ in 0..theme {
        h.press(KeyCode::Tab);
    }
    let before = h.app.settings().theme;
    h.type_char(' ');
    assert_ne!(h.app.settings().theme, before);
    assert!(is_row(focused(&mut h), theme));
}

#[test]
fn the_marker_is_on_the_focused_row_only() {
    let mut h = open();
    h.press(KeyCode::Tab);
    assert_eq!(
        h.lines_without_pane()
            .iter()
            .filter(|line| line.contains('►'))
            .count(),
        1
    );
}
