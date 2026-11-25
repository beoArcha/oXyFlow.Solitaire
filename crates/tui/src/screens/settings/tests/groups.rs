//! Process priority sits in Audio & Visual without a Performance heading and keeps preview-and-confirm.

use ratatui::crossterm::event::KeyCode;

use super::super::options::Field;
use super::super::tabs::Tab;
use super::{focus_in_tab, open_settings};
use crate::i18n::{Locale, set_locale};

#[test]
fn process_priority_is_the_last_row_of_audio_and_visual() {
    assert_eq!(Tab::AudioVisual.fields().last(), Some(&Field::Priority));
    assert!(!Tab::Gameplay.fields().contains(&Field::Priority));
}

#[test]
fn the_performance_heading_is_gone_in_every_locale() {
    for locale in Locale::offered() {
        set_locale(locale);
        let mut h = open_settings();
        h.resize(100, 60);
        focus_in_tab(&mut h, Tab::AudioVisual, Field::Priority);
        let lines = h.lines_without_pane();
        let row = crate::i18n::t(Field::Priority.label_key());
        let priority = lines.iter().position(|line| line.contains(&*row));
        assert!(priority.is_some(), "{locale:?}");
        let prev_row = crate::i18n::t(Field::Volume.label_key());
        assert_eq!(
            priority,
            lines
                .iter()
                .position(|line| line.contains(&*prev_row))
                .map(|at| at + 1),
            "{locale:?}"
        );
    }
    set_locale(Locale::EnUs);
}

#[test]
fn process_priority_still_previews_and_waits_for_confirmation() {
    let mut h = open_settings();
    focus_in_tab(&mut h, Tab::AudioVisual, Field::Priority);
    let before = h.app.settings().process_priority;
    h.press(KeyCode::Right);
    assert_ne!(h.app.settings().process_priority, before);
    assert_eq!(h.storage.borrow().settings, None);
}
