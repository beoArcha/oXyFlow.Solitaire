//! The window through the whole application at the smallest supported size (100 x 30): the entry
//! points, switching, scrolling, closing by key and mouse, and clean drawing in every game, locale
//! and glyph mode.

use oxysol_core::{DrawCount, PyramidRules, SuitCount, Variant};
use ratatui::crossterm::event::{KeyCode, MouseEvent, MouseEventKind};
use ratatui::crossterm::event::{KeyModifiers, MouseButton};
use ratatui::style::Modifier;

use super::content;
use super::state::{HowToPlay, Rules};
use crate::app::{Modal, Screen};
use crate::caps::TerminalCaps;
use crate::glyphs::is_wgl4;
use crate::i18n::{self, Locale};
use crate::input::Input;
use crate::screens::Effect;
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, PLAIN, TRUE_COLOR};

const WIDTH: u16 = 100;
const HEIGHT: u16 = 30;

fn harness(caps: TerminalCaps) -> Harness {
    Harness::with(MemoryStorage::new(), caps, WIDTH, HEIGHT)
}

fn playing(variant: Variant) -> Harness {
    let mut h = harness(TRUE_COLOR);
    h.app.apply(Effect::Start {
        variant,
        seed: Some(7),
        undo_allowed: true,
    });
    h
}

fn shown(h: &Harness) -> Option<HowToPlay> {
    match h.app.modal() {
        Some(Modal::HowToPlay(state)) => Some(*state),
        _ => None,
    }
}

fn open(h: &mut Harness, rules: Rules) {
    h.app
        .apply(Effect::Open(Modal::HowToPlay(HowToPlay::new(rules))));
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
    let Some((column, row)) = h.find(needle) else {
        unreachable!("{needle} is on the screen")
    };
    h.click(column, row);
}

fn is_ascii_or_wgl4(symbol: char) -> bool {
    symbol.is_ascii() || is_wgl4(symbol)
}

#[test]
fn the_menu_item_opens_klondike_by_keyboard() {
    let mut h = harness(TRUE_COLOR);
    for _ in 0..4 {
        h.press(KeyCode::Down);
    }
    h.press(KeyCode::Enter);
    assert_eq!(shown(&h), Some(HowToPlay::new(Rules::Klondike)));
    let text = h.text();
    assert!(text.contains("Help"));
    assert!(text.contains("[Rules]"));
    assert!(text.contains("[Klondike]"));
    assert!(text.contains("Goal"));
}

#[test]
fn the_menu_item_opens_by_mouse() {
    let mut h = harness(TRUE_COLOR);
    click_text(&mut h, "How to play");
    assert_eq!(shown(&h), Some(HowToPlay::new(Rules::Klondike)));
}

#[test]
fn the_menu_item_opens_on_the_last_played_game() {
    let mut h = playing(Variant::Pyramid(PyramidRules::Relaxed));
    h.type_char('m');
    assert_eq!(h.app.screen(), Screen::MainMenu);
    click_text(&mut h, "How to play");
    assert_eq!(shown(&h), Some(HowToPlay::new(Rules::Pyramid)));
}

#[test]
fn the_board_key_opens_the_rules_of_the_current_game() {
    let cases = [
        (Variant::Klondike(DrawCount::Three), Rules::Klondike),
        (Variant::Spider(SuitCount::Four), Rules::Spider),
        (Variant::FreeCell, Rules::FreeCell),
        (Variant::Pyramid(PyramidRules::Classic), Rules::Pyramid),
    ];
    for (variant, rules) in cases {
        let mut h = playing(variant);
        h.type_char('i');
        assert_eq!(shown(&h), Some(HowToPlay::new(rules)), "{variant:?}");
        assert!(!h.app.board().clock_running());
        h.press(KeyCode::Esc);
        assert!(h.app.modal().is_none());
        assert!(h.app.board().clock_running());
        assert_eq!(h.app.screen(), Screen::Board);
    }
}

#[test]
fn left_and_right_switch_between_the_games() {
    let mut h = playing(Variant::Klondike(DrawCount::One));
    h.type_char('i');
    h.press(KeyCode::Right);
    assert!(h.text().contains("[Spider]"));
    assert!(h.text().contains("eight complete runs"));
    h.press(KeyCode::Right);
    assert!(h.text().contains("[FreeCell]"));
    h.press(KeyCode::Right);
    assert!(h.text().contains("[Pyramid]"));
    h.press(KeyCode::Right);
    assert!(h.text().contains("[Yukon]"));
    assert!(h.text().contains("no redeal"));
    h.press(KeyCode::Right);
    assert!(h.text().contains("[Golf]"));
    assert!(h.text().contains("no foundation"));
    h.press(KeyCode::Right);
    assert!(h.text().contains("[TriPeaks]"));
    assert!(h.text().contains("three overlapping peaks"));
    h.press(KeyCode::Right);
    assert!(h.text().contains("[Klondike]"));
    h.press(KeyCode::Left);
    assert!(h.text().contains("[TriPeaks]"));
}

#[test]
fn the_arrows_and_tabs_switch_the_game_by_mouse() {
    let mut h = playing(Variant::Klondike(DrawCount::One));
    h.type_char('i');
    click_text(&mut h, "FreeCell");
    assert_eq!(shown(&h).map(|s| s.rules), Some(Rules::FreeCell));
    click_text(&mut h, "►");
    assert_eq!(shown(&h).map(|s| s.rules), Some(Rules::Pyramid));
    let (column, row) = h.find("◄").unwrap_or_default();
    h.click(column, row);
    assert_eq!(shown(&h).map(|s| s.rules), Some(Rules::FreeCell));
    click_text(&mut h, "Spider");
    assert_eq!(shown(&h).map(|s| s.rules), Some(Rules::Spider));
}

#[test]
fn keys_and_the_wheel_scroll_the_text() {
    let mut h = playing(Variant::Klondike(DrawCount::One));
    h.type_char('i');
    assert!(h.text().contains("1-20 / "));
    assert!(!h.text().contains("Stuck?"));
    h.press(KeyCode::Down);
    assert!(h.text().contains("2-21 / "));
    h.press(KeyCode::PageDown);
    assert_eq!(shown(&h).map(|s| s.scroll), Some(20));
    wheel(&mut h, MouseEventKind::ScrollUp);
    assert_eq!(shown(&h).map(|s| s.scroll), Some(17));
    wheel(&mut h, MouseEventKind::ScrollDown);
    wheel(&mut h, MouseEventKind::ScrollDown);
    wheel(&mut h, MouseEventKind::ScrollDown);
    let state = shown(&h).unwrap_or(HowToPlay::new(Rules::Klondike));
    assert_eq!(state.scroll, state.max_scroll());
    assert!(h.text().contains("Stuck?"));
    h.press(KeyCode::Home);
    assert!(h.text().contains("1-20 / "));
}

#[test]
fn escape_enter_and_the_close_button_close_the_window() {
    for code in [KeyCode::Esc, KeyCode::Enter] {
        let mut h = playing(Variant::FreeCell);
        h.type_char('i');
        h.press(code);
        assert!(h.app.modal().is_none(), "{code:?}");
    }
    let mut h = playing(Variant::FreeCell);
    h.type_char('i');
    assert!(h.text().contains("► Close [Esc] ◄"));
    click_text(&mut h, "Close");
    assert!(h.app.modal().is_none());
    assert_eq!(h.app.screen(), Screen::Board);
}

#[test]
fn a_click_on_the_text_keeps_the_window_open() {
    let mut h = playing(Variant::FreeCell);
    h.type_char('i');
    h.click(50, 10);
    h.click(0, 0);
    assert!(shown(&h).is_some());
}

#[test]
fn the_key_opens_the_window_on_the_menu_too() {
    let mut h = harness(TRUE_COLOR);
    h.type_char('i');
    assert_eq!(shown(&h), Some(HowToPlay::new(Rules::Klondike)));
    h.press(KeyCode::Esc);
    assert!(h.app.modal().is_none());
}

#[test]
fn the_chosen_tab_and_the_close_button_show_focus_without_color() {
    let mut h = playing(Variant::FreeCell);
    h.type_char('i');
    let (column, row) = h.find("[FreeCell]").unwrap_or_default();
    let cell = &h.terminal.backend().buffer()[(column, row)];
    assert!(cell.modifier.contains(Modifier::UNDERLINED));
    assert!(cell.modifier.contains(Modifier::BOLD));
    assert!(!cell.modifier.contains(Modifier::REVERSED));
    let (btn_col, btn_row) = h.find("► Close [Esc] ◄").unwrap_or_default();
    let btn_cell = &h.terminal.backend().buffer()[(btn_col, btn_row)];
    assert!(btn_cell.modifier.contains(Modifier::REVERSED));
    assert!(btn_cell.modifier.contains(Modifier::BOLD));
    assert!(!btn_cell.modifier.contains(Modifier::UNDERLINED));
    let text = h.text();
    assert!(text.contains("[FreeCell]"));
    assert!(!text.contains("[Klondike]"));
}

#[test]
fn every_game_draws_inside_100_by_30_with_only_wgl4_or_ascii_glyphs() {
    for locale in Locale::offered() {
        for caps in [TRUE_COLOR, PLAIN] {
            for rules in Rules::ALL {
                let mut h = harness(caps);
                i18n::set_locale(locale);
                open(&mut h, rules);
                let lines = h.lines();
                let context = format!("{locale:?} unicode={} {rules:?}", caps.unicode);
                assert_eq!(lines.len(), usize::from(HEIGHT));
                for ch in lines.iter().flat_map(|line| line.chars()) {
                    assert!(is_ascii_or_wgl4(ch), "{context}: {ch:?}");
                }
                let text = lines.join("\n");
                assert!(
                    text.contains(&i18n::tf("help.rules.close", &[("key", "Esc")])),
                    "{context}"
                );
                let up_down = format!("{}/{}", i18n::t("help.key.up"), i18n::t("help.key.down"));
                assert!(text.contains(&up_down), "{context}");
                let frame = if caps.unicode { '╔' } else { '+' };
                assert!(text.contains(frame), "{context}");
            }
        }
    }
    i18n::set_locale(Locale::EnUs);
}

#[test]
fn the_first_and_the_last_line_of_every_page_can_be_read() {
    for locale in Locale::offered() {
        for rules in Rules::ALL {
            let mut h = harness(TRUE_COLOR);
            i18n::set_locale(locale);
            open(&mut h, rules);
            let rows = content::rows(rules);
            let (first, last) = (rows.first(), rows.last());
            assert!(h.text().contains(first.map_or("", |row| row.text.as_str())));
            h.press(KeyCode::End);
            let text = h.text();
            assert!(
                text.contains(last.map_or("", |row| row.text.as_str())),
                "{locale:?} {rules:?}"
            );
            assert!(text.contains(&format!("/ {}", rows.len())));
        }
    }
    i18n::set_locale(Locale::EnUs);
}

#[test]
fn the_window_does_not_cover_the_terminal_edges_in_ascii_mode() {
    let mut h = harness(PLAIN);
    open(&mut h, Rules::Spider);
    let text = h.text();
    assert!(text.is_ascii());
    assert!(text.contains("> Close [Esc] <"));
    assert!(text.contains("[Spider]"));
}

#[test]
fn a_right_click_keeps_the_window_open() {
    let mut h = playing(Variant::FreeCell);
    h.type_char('i');
    h.send(Input::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Right),
        column: 50,
        row: 15,
        modifiers: KeyModifiers::NONE,
    }));
    assert!(shown(&h).is_some());
}
