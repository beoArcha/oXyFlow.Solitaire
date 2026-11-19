use ratatui::crossterm::event::KeyCode;

use super::super::options::{FIELDS, Field};
use super::{bare, line_of, move_down, open_settings};
use crate::i18n::{Locale, set_locale};
use crate::screens::Effect;
use crate::settings::Settings;
use crate::test_support::Harness;

fn move_to_guide(h: &mut Harness) {
    let index = FIELDS
        .iter()
        .position(|field| *field == Field::GuidePanel)
        .unwrap_or_default();
    move_down(h, index);
}

#[test]
fn the_guide_panel_row_toggles_and_reaches_storage() {
    let mut h = open_settings();
    h.resize(130, 40);
    move_to_guide(&mut h);
    assert!(bare(&line_of(&mut h, "Guide panel")).ends_with("On"));
    h.press(KeyCode::Enter);
    assert!(!h.app.settings().guide_panel);
    assert!(bare(&line_of(&mut h, "Guide panel")).ends_with("Off"));
    let saved = h.storage.borrow().settings.clone();
    assert!(saved.is_some_and(|settings| !settings.guide_panel));
    h.press(KeyCode::Right);
    assert!(h.app.settings().guide_panel);
}

#[test]
fn the_guide_row_names_the_columns_the_panel_needs_on_a_narrow_screen() {
    let mut h = open_settings();
    h.resize(100, 30);
    assert!(
        bare(&line_of(&mut h, "Guide panel")).ends_with("On (needs 130 columns, now 100)"),
        "{}",
        h.text()
    );
    h.resize(129, 40);
    assert!(bare(&line_of(&mut h, "Guide panel")).ends_with("On (needs 130 columns, now 129)"));
    h.resize(130, 30);
    assert!(bare(&line_of(&mut h, "Guide panel")).ends_with("On"));
    assert!(!h.text().contains("needs"));
}

#[test]
fn the_guide_row_adds_no_note_when_the_panel_is_off() {
    let mut h = open_settings();
    h.resize(100, 30);
    move_to_guide(&mut h);
    h.press(KeyCode::Enter);
    assert!(!h.app.settings().guide_panel);
    assert!(bare(&line_of(&mut h, "Guide panel")).ends_with("Off"));
    assert!(!line_of(&mut h, "Guide panel").contains("needs"));
}

#[test]
fn the_guide_row_note_is_polish_in_polish() {
    let mut h = open_settings();
    h.resize(100, 30);
    h.app.apply(Effect::Apply(Settings {
        locale: Locale::PlPl,
        ..Settings::default()
    }));
    let text = h.text();
    assert!(
        text.contains("Tak (potrzeba 130 kolumn, jest 100)"),
        "{text}"
    );
    set_locale(Locale::EnUs);
}
