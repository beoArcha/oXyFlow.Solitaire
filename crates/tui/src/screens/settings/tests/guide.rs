use ratatui::crossterm::event::KeyCode;

use super::super::options::FIELDS;
use super::{move_down, open_settings};
use crate::i18n::{Locale, set_locale};
use crate::screens::Effect;
use crate::settings::Settings;

#[test]
fn the_guide_panel_row_toggles_and_reaches_storage() {
    let mut h = open_settings();
    move_down(&mut h, FIELDS.len() - 1);
    assert!(h.text().contains("Guide panel: On"));
    h.press(KeyCode::Enter);
    assert!(!h.app.settings().guide_panel);
    assert!(h.text().contains("Guide panel: Off"));
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
        h.text()
            .contains("Guide panel: On (needs 130 columns, now 100)"),
        "{}",
        h.text()
    );
    h.resize(129, 40);
    assert!(
        h.text()
            .contains("Guide panel: On (needs 130 columns, now 129)")
    );
    h.resize(130, 30);
    assert!(h.text().contains("Guide panel: On"));
    assert!(!h.text().contains("needs"));
}

#[test]
fn the_guide_row_adds_no_note_when_the_panel_is_off() {
    let mut h = open_settings();
    h.resize(100, 30);
    move_down(&mut h, FIELDS.len() - 1);
    h.press(KeyCode::Enter);
    assert!(!h.app.settings().guide_panel);
    assert!(h.text().contains("Guide panel: Off"));
    assert!(!h.text().contains("needs"));
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
        text.contains("Panel przewodnika: Tak (potrzeba 130 kolumn, jest 100)"),
        "{text}"
    );
    set_locale(Locale::EnUs);
}
