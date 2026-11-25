//! The footer of the settings screen names Left/Right (value change) and Shift+Tab / Tab (tabs).

use ratatui::crossterm::event::KeyCode;

use super::super::options::Field;
use super::super::tabs::Tab;
use super::{focus_in_tab, open_settings};
use crate::action::Action;
use crate::i18n::{Locale, set_locale, t, tf};
use crate::input::primary_key;

fn footer_line(lines: &[String]) -> String {
    lines
        .iter()
        .rev()
        .find(|line| !line.trim().is_empty())
        .cloned()
        .unwrap_or_default()
}

#[test]
fn the_footer_names_the_arrows_and_tab_keys_in_every_locale() {
    for locale in Locale::offered() {
        set_locale(locale);
        let mut h = open_settings();
        h.resize(100, 30);
        let arrows = tf("settings.footer.arrows", &[("arrows", "◄ ►")]);
        let prev = primary_key(Action::FocusPrevious).unwrap_or_default();
        let next = primary_key(Action::FocusNext).unwrap_or_default();
        let tabs = tf("settings.footer.tabs", &[("prev", &prev), ("next", &next)]);
        let footer = footer_line(&h.lines());
        assert!(footer.contains(&arrows), "{locale:?}: {footer:?}");
        assert!(footer.contains(&tabs), "{locale:?}: {footer:?}");
        assert!(
            footer.contains(&*t("settings.footer.change")),
            "{locale:?}: {footer:?}"
        );
    }
    set_locale(Locale::EnUs);
}

#[test]
fn the_arrows_in_the_footer_really_change_the_focused_row() {
    let mut h = open_settings();
    focus_in_tab(&mut h, Tab::Popular, Field::Theme);
    let before = h.app.settings().theme;
    h.press(KeyCode::Right);
    let forward = h.app.settings().theme;
    h.press(KeyCode::Left);
    assert_ne!(forward, before);
    assert_eq!(h.app.settings().theme, before);
}
