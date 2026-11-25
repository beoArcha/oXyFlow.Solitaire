//! Tests for the unified Help window footer across Keys, Rules, and Command line in all
//! offered locales: same entries in the same order, line counter on overflow only, and fit at
//! 100 x 30 and 100 x 60.

use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;

use super::help;
use super::tabs::HelpTab;
use crate::action::Action;
use crate::i18n::{self, Locale, t, tf};
use crate::input::primary_key;
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, TRUE_COLOR};

fn open(tab: HelpTab, locale: Locale, width: u16, height: u16) -> Harness {
    let mut h = Harness::with(MemoryStorage::new(), TRUE_COLOR, width, height);
    i18n::set_locale(locale);
    match tab {
        HelpTab::Keys => h.type_char('?'),
        HelpTab::Rules => h.type_char('i'),
        HelpTab::CommandLine => {
            h.type_char('?');
            h.press(KeyCode::Left);
        }
    }
    h.lines();
    h
}

fn key_name(action: Action) -> String {
    primary_key(action).unwrap_or_default()
}

fn footer_line(h: &mut Harness, width: u16, height: u16) -> String {
    let geometry = help::geometry(Rect::new(0, 0, width, height));
    let footer_row = usize::from(geometry.body.bottom().saturating_sub(1));
    h.lines()[footer_row].clone()
}

#[test]
fn every_tab_shows_the_same_footer_entries_in_the_same_order_when_overflowing() {
    for locale in Locale::offered() {
        for tab in HelpTab::ALL {
            let mut h = open(tab, locale, 100, 30);
            let row = footer_line(&mut h, 100, 30);
            let tab_hint = tf(
                "help.command_line.footer_tab",
                &[("key", &key_name(Action::FocusNext))],
            );
            let move_hint = tf(
                "help.command_line.footer_move",
                &[
                    ("up", &key_name(Action::Up)),
                    ("down", &key_name(Action::Down)),
                ],
            );
            let close_hint = tf(
                "help.command_line.footer_close",
                &[("back", &key_name(Action::Back))],
            );
            let mut expected = vec![tab_hint];
            if tab == HelpTab::Rules {
                expected.push(tf(
                    "help.rules.footer_game",
                    &[
                        ("left", &key_name(Action::Left)),
                        ("right", &key_name(Action::Right)),
                    ],
                ));
            }
            expected.push(move_hint);
            expected.push(close_hint);

            let mut cursor = 0;
            for entry in &expected {
                let found = row[cursor..].find(entry.as_str()).unwrap_or_else(|| {
                    panic!("{locale:?} {tab:?}: missing {entry:?} after {cursor} in {row:?}")
                });
                cursor += found + entry.len();
            }
            assert!(
                row[cursor..].contains(" / "),
                "{locale:?} {tab:?}: missing line counter after {cursor} in {row:?}"
            );
        }
    }
    i18n::set_locale(Locale::EnUs);
}

#[test]
fn every_tab_fits_at_100_by_30_and_100_by_60_in_every_locale() {
    let mut failures = Vec::new();
    for locale in Locale::offered() {
        for (width, height) in [(100, 30), (100, 60)] {
            for tab in HelpTab::ALL {
                let mut h = open(tab, locale, width, height);
                let row = footer_line(&mut h, width, height);
                let close_hint = tf(
                    "help.command_line.footer_close",
                    &[("back", &key_name(Action::Back))],
                );
                if !row.contains(close_hint.as_str()) {
                    failures.push(format!(
                        "{locale:?} {tab:?} {width}x{height}: {close_hint:?} truncated in {row:?}"
                    ));
                }
            }
        }
    }
    i18n::set_locale(Locale::EnUs);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn keys_tab_shows_line_counter_only_when_overflowing_and_updates_on_scroll() {
    let mut h = open(HelpTab::Keys, Locale::EnUs, 100, 30);
    let row = footer_line(&mut h, 100, 30);
    assert!(row.contains("1-21 / "), "{row:?}");
    h.press(KeyCode::Down);
    let row = footer_line(&mut h, 100, 30);
    assert!(row.contains("2-22 / "), "{row:?}");

    let mut tall = open(HelpTab::Keys, Locale::EnUs, 100, 60);
    let tall_row = footer_line(&mut tall, 100, 60);
    assert!(
        !tall_row.contains(" / "),
        "non-overflowing Keys tab should not show line counter: {tall_row:?}"
    );
    let tab_hint = tf(
        "help.command_line.footer_tab",
        &[("key", &key_name(Action::FocusNext))],
    );
    assert!(tall_row.contains(tab_hint.as_str()));
    assert!(
        tall_row.contains(
            &t("help.command_line.footer_close").replace("{back}", &key_name(Action::Back))
        )
    );
}
