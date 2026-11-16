//! Tests of the "Command line" tab: reaching it, switching away, scrolling and rendering.

use ratatui::crossterm::event::{KeyCode, KeyModifiers, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

use super::command_line::{self, CommandLine, VIEW_ROWS};
use super::help;
use crate::app::Modal;
use crate::caps::TerminalCaps;
use crate::glyphs::is_wgl4;
use crate::i18n::{self, Locale};
use crate::input::Input;
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, PLAIN, TRUE_COLOR};

const WIDTH: u16 = 100;
const HEIGHT: u16 = 30;

fn harness(caps: TerminalCaps) -> Harness {
    Harness::with(MemoryStorage::new(), caps, WIDTH, HEIGHT)
}

fn harness_in(locale: Locale, caps: TerminalCaps) -> Harness {
    let h = harness(caps);
    i18n::set_locale(locale);
    h
}

fn on_command_line() -> Harness {
    let mut h = harness(TRUE_COLOR);
    h.type_char('?');
    h.press(KeyCode::Tab);
    h.press(KeyCode::Tab);
    h
}

fn scroll_of(h: &Harness) -> Option<u16> {
    match h.app.modal() {
        Some(Modal::CommandLine(state)) => Some(state.scroll),
        _ => None,
    }
}

fn wheel(h: &mut Harness, kind: MouseEventKind) {
    h.send(Input::Mouse(MouseEvent {
        kind,
        column: 50,
        row: 15,
        modifiers: KeyModifiers::NONE,
    }));
}

fn click_text(h: &mut Harness, needle: &str) {
    let (col, row) = h
        .find(needle)
        .unwrap_or_else(|| panic!("{needle} not found"));
    h.click(col, row);
}

fn text_columns() -> usize {
    let area = Rect::new(0, 0, WIDTH, HEIGHT);
    usize::from(help::geometry(area).body.width.saturating_sub(2))
}

#[test]
fn the_third_tab_shows_the_help_text_with_the_new_window_option() {
    let mut h = on_command_line();
    assert_eq!(scroll_of(&h), Some(0));
    let text = h.text();
    assert!(text.contains("[Command line]"));
    assert!(text.contains("Usage: oxysol [options]"));
    assert!(text.contains("--version"));
    assert!(text.contains("1-21 / "));
}

#[test]
fn the_help_text_is_the_cli_text_fits_the_window_and_names_the_new_option_in_both_locales() {
    for locale in Locale::ALL {
        i18n::set_locale(locale);
        let lines = command_line::lines();
        assert!(lines.len() > VIEW_ROWS, "{locale:?} should scroll");
        for line in &lines {
            assert!(line.chars().count() <= text_columns(), "{locale:?}: {line}");
        }
        let text = i18n::t("cli.help");
        for needle in [
            "--new-window",
            "--seed",
            "--ascii",
            "--colors",
            "OXYSOL_HOME",
            "oxysol.portable",
        ] {
            assert!(text.contains(needle), "{locale:?} lacks {needle}");
        }
        assert_ne!(i18n::t("help.tab.command_line"), "help.tab.command_line");
        assert_ne!(i18n::t("help.command_line.footer"), "help.command_line.footer");
    }
    i18n::set_locale(Locale::EnUs);
}

#[test]
fn polish_names_the_tab_in_polish() {
    let mut h = harness_in(Locale::PlPl, TRUE_COLOR);
    h.type_char('?');
    h.press(KeyCode::BackTab);
    assert!(h.text().contains("[Wiersz poleceń]"));
    assert!(h.text().contains("Użycie: oxysol [opcje]"));
    i18n::set_locale(Locale::EnUs);
}

#[test]
fn keys_step_through_the_tabs_in_both_directions() {
    let mut h = harness(TRUE_COLOR);
    h.type_char('?');
    h.press(KeyCode::Left);
    assert!(matches!(h.app.modal(), Some(Modal::CommandLine(_))));
    h.press(KeyCode::Right);
    assert_eq!(h.app.modal(), Some(&Modal::Help));
    h.press(KeyCode::BackTab);
    assert!(matches!(h.app.modal(), Some(Modal::CommandLine(_))));
    h.press(KeyCode::Left);
    assert!(matches!(h.app.modal(), Some(Modal::HowToPlay(_))));
    h.press(KeyCode::Tab);
    assert!(matches!(h.app.modal(), Some(Modal::CommandLine(_))));
    h.type_char('?');
    assert_eq!(h.app.modal(), Some(&Modal::Help));
    h.press(KeyCode::Tab);
    h.press(KeyCode::Tab);
    h.type_char('i');
    assert!(matches!(h.app.modal(), Some(Modal::HowToPlay(_))));
}

#[test]
fn clicking_the_tabs_switches_between_all_three() {
    let mut h = harness(TRUE_COLOR);
    h.type_char('?');
    click_text(&mut h, "Command line");
    assert!(matches!(h.app.modal(), Some(Modal::CommandLine(_))));
    assert!(h.text().contains("[Command line]"));
    click_text(&mut h, "Rules");
    assert!(matches!(h.app.modal(), Some(Modal::HowToPlay(_))));
    click_text(&mut h, "Command line");
    click_text(&mut h, "Keys");
    assert_eq!(h.app.modal(), Some(&Modal::Help));
}

#[test]
fn the_text_scrolls_by_key_and_by_wheel_and_stops_at_both_ends() {
    let mut h = on_command_line();
    let max = CommandLine::new().max_scroll();
    assert!(max > 0);
    h.press(KeyCode::Up);
    assert_eq!(scroll_of(&h), Some(0));
    h.press(KeyCode::Down);
    assert_eq!(scroll_of(&h), Some(1));
    assert!(h.text().contains("2-22 / "));
    h.press(KeyCode::PageDown);
    assert!(scroll_of(&h) >= Some(1));
    h.press(KeyCode::End);
    assert_eq!(scroll_of(&h), Some(max));
    h.press(KeyCode::Down);
    assert_eq!(scroll_of(&h), Some(max));
    assert!(h.text().contains("OXYSOL_LOG"));
    h.press(KeyCode::Home);
    assert_eq!(scroll_of(&h), Some(0));
    wheel(&mut h, MouseEventKind::ScrollDown);
    assert_eq!(scroll_of(&h), Some(3));
    wheel(&mut h, MouseEventKind::ScrollUp);
    assert_eq!(scroll_of(&h), Some(0));
}

#[test]
fn escape_and_the_close_button_close_the_window() {
    let mut h = on_command_line();
    h.press(KeyCode::Esc);
    assert!(h.app.modal().is_none());
    let mut h = on_command_line();
    click_text(&mut h, "Close");
    assert!(h.app.modal().is_none());
}

#[test]
fn renders_cleanly_at_100x30_in_both_locales_and_glyph_modes_at_both_ends() {
    for locale in Locale::ALL {
        for caps in [TRUE_COLOR, PLAIN] {
            let mut h = harness_in(locale, caps);
            h.type_char('?');
            h.press(KeyCode::BackTab);
            for end in [KeyCode::Home, KeyCode::End] {
                h.press(end);
                let text = h.text();
                for ch in text.chars() {
                    assert!(
                        ch.is_ascii() || is_wgl4(ch),
                        "{locale:?} unicode={}: {ch:?}",
                        caps.unicode
                    );
                }
                let tab = i18n::t("help.tab.command_line");
                assert!(text.contains(&format!("[{tab}]")), "{locale:?}");
            }
        }
    }
    i18n::set_locale(Locale::EnUs);
}
