use ratatui::crossterm::event::KeyCode;

use super::super::options::FIELDS;
use super::{move_down, open_settings};

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
