//! Key groups, accessibility without color, and footer behavior on the Rules tab.

use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::style::Modifier;

use super::*;
use crate::app::Modal;
use crate::caps::TerminalCaps;
use crate::i18n::{self, Locale};
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, PLAIN, TRUE_COLOR};

pub(super) const AREA: Rect = Rect::new(0, 0, 100, 30);

pub(super) fn opened(locale: Locale, caps: TerminalCaps) -> Harness {
    let mut h = Harness::with(MemoryStorage::new(), caps, AREA.width, AREA.height);
    i18n::set_locale(locale);
    h.type_char('i');
    h.lines();
    h
}

pub(super) fn state(h: &Harness) -> HowToPlay {
    match h.app.modal() {
        Some(Modal::HowToPlay(state)) => *state,
        other => panic!("rules are not open: {other:?}"),
    }
}

fn row_of(h: &mut Harness, needle: &str) -> usize {
    h.lines()
        .iter()
        .position(|line| line.contains(needle))
        .unwrap_or_else(|| panic!("{needle} is not drawn"))
}

#[test]
fn the_selected_game_is_marked_without_color_in_unicode_and_ascii() {
    let mut h = opened(Locale::EnUs, TRUE_COLOR);
    let games_row = row_of(&mut h, "[Klondike]");
    let lines = h.lines();
    assert!(lines[games_row].contains('◄') && lines[games_row].contains('►'));
    let (column, row) = h.find("[Klondike]").expect("games selection");
    let selected = &h.terminal.backend().buffer()[(column, row)];
    assert!(
        selected
            .modifier
            .contains(Modifier::REVERSED | Modifier::BOLD)
    );

    let mut ascii = opened(Locale::EnUs, PLAIN);
    let ascii_row = row_of(&mut ascii, "[Klondike]");
    let ascii_lines = ascii.lines();
    assert!(ascii_lines[ascii_row].contains(" < ") && ascii_lines[ascii_row].contains(" > "));
    assert!(ascii_lines[ascii_row].contains("[Klondike]"));
}

#[test]
fn tab_and_shift_tab_switch_the_main_tab_from_rules_in_every_locale() {
    for locale in Locale::offered() {
        let mut h = opened(locale, TRUE_COLOR);
        h.press(KeyCode::Tab);
        assert!(matches!(h.app.modal(), Some(Modal::CommandLine(_))));
        let command_line = i18n::t("help.tab.command_line");
        assert!(
            h.text().contains(&format!("[{command_line}]")),
            "{locale:?}: missing [{command_line}]"
        );

        h.press(KeyCode::BackTab);
        assert!(matches!(h.app.modal(), Some(Modal::HowToPlay(_))));
        let rules = i18n::t("help.tab.rules");
        assert!(
            h.text().contains(&format!("[{rules}]")),
            "{locale:?}: missing [{rules}]"
        );

        h.press(KeyCode::BackTab);
        assert!(matches!(h.app.modal(), Some(Modal::Help(_))));
        let keys = i18n::t("help.tab.keys");
        assert!(
            h.text().contains(&format!("[{keys}]")),
            "{locale:?}: missing [{keys}]"
        );
    }
    i18n::set_locale(Locale::EnUs);
}

#[test]
fn left_and_right_switch_the_game_from_the_rules_text_in_every_locale() {
    for locale in Locale::offered() {
        let mut h = opened(locale, TRUE_COLOR);
        h.press(KeyCode::Down);
        assert_eq!(state(&h).scroll, 1, "{locale:?}");

        h.press(KeyCode::Right);
        assert_eq!(state(&h).rules, Rules::Spider, "{locale:?}");
        assert_eq!(state(&h).scroll, 0, "{locale:?}");
        let spider = i18n::t(Rules::Spider.name_key());
        let spider_text = content::rows(Rules::Spider)[0].text.clone();
        let rendered = h.text();
        assert!(
            rendered.contains(&format!("[{spider}]")),
            "{locale:?}: missing [{spider}]"
        );
        assert!(
            rendered.contains(&spider_text),
            "{locale:?}: missing {spider_text}"
        );

        h.press(KeyCode::Left);
        assert_eq!(state(&h).rules, Rules::Klondike, "{locale:?}");
        let klondike = i18n::t(Rules::Klondike.name_key());
        assert!(
            h.text().contains(&format!("[{klondike}]")),
            "{locale:?}: missing [{klondike}]"
        );
    }
    i18n::set_locale(Locale::EnUs);
}

#[test]
fn up_and_down_scroll_from_the_start_in_every_locale() {
    for locale in Locale::offered() {
        let mut h = opened(locale, TRUE_COLOR);
        assert_eq!(state(&h).scroll, 0, "{locale:?}");
        assert!(h.text().contains("1-20 / "), "{locale:?}");

        h.press(KeyCode::Down);
        assert_eq!(state(&h).scroll, 1, "{locale:?}");
        assert!(h.text().contains("2-21 / "), "{locale:?}");

        h.press(KeyCode::Up);
        assert_eq!(state(&h).scroll, 0, "{locale:?}");
        assert!(h.text().contains("1-20 / "), "{locale:?}");

        h.press(KeyCode::PageDown);
        assert!(state(&h).scroll >= 19, "{locale:?}");
        h.press(KeyCode::Home);
        assert_eq!(state(&h).scroll, 0, "{locale:?}");
        h.press(KeyCode::End);
        assert_eq!(state(&h).scroll, state(&h).max_scroll(), "{locale:?}");
    }
    i18n::set_locale(Locale::EnUs);
}

#[test]
fn the_footer_keeps_its_entries_across_every_locale() {
    for locale in Locale::offered() {
        let mut h = opened(locale, TRUE_COLOR);
        let tab = i18n::tf("help.command_line.footer_tab", &[("key", "Tab")]);
        let game = i18n::tf(
            "help.rules.footer_game",
            &[
                ("left", &i18n::t("help.key.left")),
                ("right", &i18n::t("help.key.right")),
            ],
        );
        let move_keys = i18n::tf(
            "help.command_line.footer_move",
            &[
                ("up", &i18n::t("help.key.up")),
                ("down", &i18n::t("help.key.down")),
            ],
        );
        let close = i18n::tf("help.command_line.footer_close", &[("back", "Esc")]);
        let text = h.text();
        for entry in [&tab, &game, &move_keys, &close] {
            assert!(text.contains(entry.as_str()), "{locale:?}: missing {entry}");
        }
    }
    i18n::set_locale(Locale::EnUs);
}

#[test]
fn the_footer_keeps_its_close_button_clickable() {
    let mut h = opened(Locale::EnUs, TRUE_COLOR);
    let (column, row) = h.find("Esc: close").expect("footer button");
    h.click(column + 1, row);
    assert!(h.app.modal().is_none());
}
