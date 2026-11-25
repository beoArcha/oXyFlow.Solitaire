//! The footer of the settings screen (O-96) names Left and Right, which change the focused row.

use ratatui::crossterm::event::KeyCode;

use super::{index_of, move_down, open_settings};
use crate::i18n::{Locale, set_locale, t, tf};
use crate::screens::settings::options::Field;

fn footer_line(lines: &[String]) -> String {
    lines
        .iter()
        .rev()
        .find(|line| !line.trim().is_empty())
        .cloned()
        .unwrap_or_default()
}

#[test]
fn the_footer_names_the_arrows_that_change_the_focused_row_in_every_locale() {
    for locale in Locale::offered() {
        set_locale(locale);
        let mut h = open_settings();
        h.resize(100, 30);
        let hint = tf("settings.footer.arrows", &[("arrows", "◄ ►")]);
        let footer = footer_line(&h.lines());
        assert!(footer.contains(&hint), "{locale:?}: {footer:?}");
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
    move_down(&mut h, index_of(Field::Theme));
    let before = h.app.settings().theme;
    h.press(KeyCode::Right);
    let forward = h.app.settings().theme;
    h.press(KeyCode::Left);
    assert_ne!(forward, before);
    assert_eq!(h.app.settings().theme, before);
}
