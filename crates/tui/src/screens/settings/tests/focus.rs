//! Row navigation with Down and Up (wrapping at both ends of the active tab), Space on a row,
//! and the focus marker.

use ratatui::crossterm::event::KeyCode;

use super::settings_menu_index;
use crate::app::Screen;
use crate::i18n::t;
use crate::screens::settings::options::Field;
use crate::screens::settings::tabs::Tab;
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
    h.lines_without_pane().into_iter().find_map(|line| {
        line.trim_start()
            .strip_prefix('►')
            .map(|after| after.trim().to_owned())
    })
}

fn is_row(text: Option<String>, field: Field) -> bool {
    let label = t(field.label_key()).into_owned();
    text.is_some_and(|text| text.starts_with(label.as_str()))
}

#[test]
fn down_moves_the_focus_through_the_active_tab_one_row_at_a_time() {
    let mut h = open();
    let popular = Tab::Popular.fields();
    assert!(is_row(focused(&mut h), popular[0]));
    h.press(KeyCode::Down);
    assert!(is_row(focused(&mut h), popular[1]));
    h.press(KeyCode::Down);
    assert!(is_row(focused(&mut h), popular[2]));
}

#[test]
fn down_from_the_last_row_wraps_to_the_first() {
    let mut h = open();
    let popular = Tab::Popular.fields();
    for field in popular {
        assert!(is_row(focused(&mut h), *field), "{field:?}");
        h.press(KeyCode::Down);
    }
    assert!(is_row(focused(&mut h), popular[0]));
}

#[test]
fn up_from_the_first_row_wraps_to_the_last() {
    let mut h = open();
    let popular = Tab::Popular.fields();
    h.press(KeyCode::Up);
    assert!(is_row(focused(&mut h), popular[popular.len() - 1]));
    h.press(KeyCode::Up);
    assert!(is_row(focused(&mut h), popular[popular.len() - 2]));
}

#[test]
fn space_changes_the_focused_row_forward_like_enter() {
    let mut h = open();
    let theme_index = Tab::Popular
        .fields()
        .iter()
        .position(|field| *field == Field::Theme)
        .unwrap_or_default();
    for _ in 0..theme_index {
        h.press(KeyCode::Down);
    }
    let before = h.app.settings().theme;
    h.type_char(' ');
    assert_ne!(h.app.settings().theme, before);
    assert!(is_row(focused(&mut h), Field::Theme));
}

#[test]
fn the_marker_is_on_the_focused_row_only() {
    let mut h = open();
    h.press(KeyCode::Down);
    assert_eq!(
        h.lines_without_pane()
            .iter()
            .filter(|line| line.trim_start().starts_with('►'))
            .count(),
        1
    );
}
