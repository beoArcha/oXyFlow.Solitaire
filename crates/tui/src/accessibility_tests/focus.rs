//! Focus on the screens and the dialogs at 100 × 30, in both locales and both glyph sets: the
//! focused element carries its marker, and every character drawn is in the Windows Glyph List 4
//! set (ASCII is part of it).

use oxysol_core::{DrawCount, Variant};
use ratatui::crossterm::event::KeyCode;

use crate::caps::TerminalCaps;
use crate::glyphs::is_wgl4;
use crate::i18n::{Locale, set_locale, t};
use crate::settings::Settings;
use crate::storage::Outcome;
use crate::storage::memory::{HistoryRow, MemoryStorage};
use crate::test_support::{Harness, PLAIN, TRUE_COLOR};

const SIZE: (u16, u16) = (100, 30);

fn harness(locale: Locale, caps: &TerminalCaps, history: Vec<HistoryRow>) -> Harness {
    let storage = MemoryStorage {
        settings: Some(Settings {
            locale,
            ..Settings::for_caps(caps)
        }),
        history,
        ..MemoryStorage::new()
    };
    Harness::with(storage, *caps, SIZE.0, SIZE.1)
}

fn marker(caps: &TerminalCaps) -> &'static str {
    if caps.unicode { "►" } else { ">" }
}

fn closer(caps: &TerminalCaps) -> &'static str {
    if caps.unicode { "◄" } else { "<" }
}

fn each_setup(mut check: impl FnMut(Locale, &TerminalCaps)) {
    for locale in Locale::offered() {
        for caps in [TRUE_COLOR, PLAIN] {
            set_locale(locale);
            check(locale, &caps);
        }
    }
    set_locale(Locale::EnUs);
}

fn assert_wgl4(h: &mut Harness, context: &str) {
    for line in h.lines() {
        for c in line.chars() {
            assert!(is_wgl4(c), "{context}: {c:?} in {line:?}");
        }
    }
}

fn played_game() -> HistoryRow {
    HistoryRow::finished(
        1,
        Variant::Klondike(DrawCount::One),
        0,
        Outcome::Won,
        60_000,
    )
}

#[test]
fn the_main_menu_marks_the_focused_item_in_both_glyph_sets() {
    each_setup(|locale, caps| {
        let mut h = harness(locale, caps, Vec::new());
        h.press(KeyCode::Tab);
        let text = h.text();
        assert!(text.contains(&format!("{} ", marker(caps))), "{locale:?}");
        assert_wgl4(&mut h, "main menu");
    });
}

#[test]
fn the_seed_entry_marks_the_field_and_the_focused_button() {
    each_setup(|locale, caps| {
        let mut h = harness(locale, caps, Vec::new());
        h.press(KeyCode::Tab);
        h.press(KeyCode::Enter);
        h.press(KeyCode::Enter);
        h.press(KeyCode::Enter);
        assert!(h.text().contains(&format!("{} [ _", marker(caps))));
        h.press(KeyCode::Tab);
        let continue_label = t("menu.seed.continue").into_owned();
        let text = h.text();
        assert!(text.contains(&format!("{} [ {continue_label} ]", marker(caps))));
        assert_wgl4(&mut h, "seed entry");
    });
}

#[test]
fn the_game_selection_marks_the_undo_toggle_when_it_has_the_focus() {
    each_setup(|locale, caps| {
        let mut h = harness(locale, caps, Vec::new());
        h.press(KeyCode::Enter);
        h.press(KeyCode::Tab);
        assert!(h.text().contains(&format!("{} [ ", marker(caps))));
        assert_wgl4(&mut h, "game selection");
    });
}

#[test]
fn the_settings_mark_one_row_at_a_time() {
    each_setup(|locale, caps| {
        let mut h = harness(locale, caps, Vec::new());
        for _ in 0..3 {
            h.press(KeyCode::Tab);
        }
        h.press(KeyCode::Enter);
        h.press(KeyCode::Tab);
        let marked = h
            .lines()
            .iter()
            .filter(|line| line.trim_start().starts_with(marker(caps)))
            .count();
        assert_eq!(marked, 1, "{locale:?}");
        assert_wgl4(&mut h, "settings");
    });
}

#[test]
fn the_statistics_mark_the_list_and_then_the_buttons() {
    each_setup(|locale, caps| {
        let mut h = harness(locale, caps, vec![played_game()]);
        for _ in 0..2 {
            h.press(KeyCode::Tab);
        }
        h.press(KeyCode::Enter);
        h.press(KeyCode::Tab);
        assert!(h.text().contains(&format!("{} [ ", marker(caps))));
        assert_wgl4(&mut h, "statistics");
    });
}

#[test]
fn the_confirm_dialog_marks_the_focused_button_with_its_markers() {
    each_setup(|locale, caps| {
        let mut h = harness(locale, caps, Vec::new());
        h.press(KeyCode::Enter);
        h.press(KeyCode::Enter);
        h.type_char('q');
        h.press(KeyCode::Tab);
        let yes = t("common.dialog.yes").into_owned();
        let text = h.text();
        let expected = format!("{} {yes} {}", marker(caps), closer(caps));
        assert!(text.contains(&expected), "{locale:?}: {expected}");
        assert_wgl4(&mut h, "confirm");
    });
}

#[test]
fn the_help_window_lists_tab_and_shift_tab_at_100_by_30() {
    each_setup(|locale, caps| {
        let mut h = harness(locale, caps, Vec::new());
        h.type_char('?');
        let text = h.text();
        assert!(text.contains("Shift+Tab"), "{locale:?}");
        let previous = t("help.action.focus_previous").into_owned();
        assert!(text.contains(&previous), "{locale:?}: {previous}");
        assert_wgl4(&mut h, "help");
    });
}
