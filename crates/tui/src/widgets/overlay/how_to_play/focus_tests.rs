//! The focus of the Rules tab: its order, what Left and Right do on each part, how each part is
//! drawn without color, what the footer says and the scroll bar of the text.

use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::style::Modifier;

use super::state::Focus;
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
fn the_window_opens_on_the_games_and_the_focus_walks_down_and_up_in_reading_order() {
    let mut state = HowToPlay::new(Rules::Klondike);
    assert_eq!(state.focus, Focus::Games);
    state.up();
    assert_eq!(state.focus, Focus::Tabs);
    state.up();
    assert_eq!(state.focus, Focus::Tabs);
    state.down();
    assert_eq!(state.focus, Focus::Games);
    state.down();
    assert_eq!(state.focus, Focus::Text);
    assert_eq!(state.scroll, 0);
}

#[test]
fn in_the_text_up_and_down_scroll_and_up_at_the_top_returns_to_the_games() {
    let mut state = HowToPlay::new(Rules::Klondike).focused(Focus::Text);
    state.down();
    state.down();
    assert_eq!(state.scroll, 2);
    state.up();
    assert_eq!((state.scroll, state.focus), (1, Focus::Text));
    state.up();
    assert_eq!((state.scroll, state.focus), (0, Focus::Text));
    state.up();
    assert_eq!((state.scroll, state.focus), (0, Focus::Games));
}

#[test]
fn tab_and_shift_tab_follow_the_same_order_and_leave_at_the_ends() {
    let mut state = HowToPlay::new(Rules::Klondike).focused(Focus::Tabs);
    assert!(state.step_focus(true));
    assert_eq!(state.focus, Focus::Games);
    assert!(state.step_focus(true));
    assert_eq!(state.focus, Focus::Text);
    assert!(!state.step_focus(true));
    assert_eq!(state.focus, Focus::Text);
    assert!(state.step_focus(false));
    assert!(state.step_focus(false));
    assert_eq!(state.focus, Focus::Tabs);
    assert!(!state.step_focus(false));
}

#[test]
fn left_and_right_switch_the_game_only_while_the_games_row_has_the_focus() {
    let mut h = opened(Locale::EnUs, TRUE_COLOR);
    assert_eq!(state(&h).focus, Focus::Games);
    h.press(KeyCode::Right);
    assert_eq!(state(&h).rules, Rules::Spider);
    h.press(KeyCode::Left);
    assert_eq!(state(&h).rules, Rules::Klondike);

    h.press(KeyCode::Down);
    assert_eq!(state(&h).focus, Focus::Text);
    h.press(KeyCode::Right);
    h.press(KeyCode::Left);
    assert_eq!(state(&h).rules, Rules::Klondike);
    assert!(matches!(h.app.modal(), Some(Modal::HowToPlay(_))));
}

#[test]
fn left_and_right_switch_the_top_tab_while_the_tabs_row_has_the_focus() {
    let mut h = opened(Locale::EnUs, TRUE_COLOR);
    h.press(KeyCode::Up);
    assert_eq!(state(&h).focus, Focus::Tabs);
    h.press(KeyCode::Right);
    assert!(matches!(h.app.modal(), Some(Modal::CommandLine(_))));
    h.press(KeyCode::Left);
    assert_eq!(state(&h).focus, Focus::Tabs);
    h.press(KeyCode::Left);
    assert_eq!(h.app.modal(), Some(&Modal::Help));
}

#[test]
fn the_focused_row_has_arrows_and_a_reversed_selection_the_other_row_has_neither() {
    let mut h = opened(Locale::EnUs, TRUE_COLOR);
    let tabs_row = row_of(&mut h, "[Rules]");
    let games_row = row_of(&mut h, "[Klondike]");
    let lines = h.lines();
    assert!(lines[games_row].contains('◄') && lines[games_row].contains('►'));
    assert!(!lines[tabs_row].contains('◄') && !lines[tabs_row].contains('►'));
    let (column, row) = h.find("[Klondike]").expect("games selection");
    let focused = &h.terminal.backend().buffer()[(column, row)];
    assert!(
        focused
            .modifier
            .contains(Modifier::REVERSED | Modifier::BOLD)
    );
    let (column, row) = h.find("[Rules]").expect("tab selection");
    let plain = &h.terminal.backend().buffer()[(column, row)];
    assert!(!plain.modifier.contains(Modifier::REVERSED));
    assert!(!plain.modifier.contains(Modifier::BOLD));

    h.press(KeyCode::Up);
    let lines = h.lines();
    assert!(lines[tabs_row].contains('◄') && lines[tabs_row].contains('►'));
    assert!(!lines[games_row].contains('◄') && !lines[games_row].contains('►'));
    assert!(lines[games_row].contains("[Klondike]"));
    let (column, row) = h.find("[Rules]").expect("tab selection");
    let now_focused = &h.terminal.backend().buffer()[(column, row)];
    assert!(now_focused.modifier.contains(Modifier::REVERSED));
}

#[test]
fn the_focus_shows_without_color_and_in_ascii() {
    let mut h = opened(Locale::EnUs, PLAIN);
    let tabs_row = row_of(&mut h, "[Rules]");
    let games_row = row_of(&mut h, "[Klondike]");
    let lines = h.lines();
    assert!(lines[games_row].contains(" < ") && lines[games_row].contains(" > "));
    assert!(!lines[tabs_row].contains(" < ") && !lines[tabs_row].contains(" > "));
    assert!(lines[games_row].contains("[Klondike]"));
    h.press(KeyCode::Down);
    let lines = h.lines();
    assert!(!lines[games_row].contains(" < ") && !lines[games_row].contains(" > "));
    assert!(lines[games_row].contains("[Klondike]"));
}

#[test]
fn the_text_shows_its_focus_with_a_marker_in_the_margin() {
    let mut h = opened(Locale::EnUs, TRUE_COLOR);
    let parts = view::parts(geometry(AREA).body);
    let marker_at = |h: &mut Harness| {
        let lines = h.lines();
        lines[usize::from(parts.text.y)]
            .chars()
            .nth(usize::from(parts.text.x))
    };
    assert_ne!(marker_at(&mut h), Some('►'));
    h.press(KeyCode::Down);
    assert_eq!(marker_at(&mut h), Some('►'));
}

#[test]
fn the_footer_names_what_left_and_right_do_for_the_focused_row() {
    for (locale, tabs, games) in [
        (Locale::EnUs, "Left/Right: tab", "Left/Right: game"),
        (Locale::PlPl, "Lewo/Prawo: zakładka", "Lewo/Prawo: gra"),
    ] {
        let mut h = opened(locale, TRUE_COLOR);
        assert!(h.text().contains(games), "{locale:?} games");
        h.press(KeyCode::Up);
        assert!(h.text().contains(tabs), "{locale:?} tabs");
        h.press(KeyCode::Down);
        h.press(KeyCode::Down);
        let text = h.text();
        assert!(!text.contains("Left/Right"), "{locale:?} text");
        assert!(text.contains("PgUp/PgDn"), "{locale:?} text");
    }
    i18n::set_locale(Locale::EnUs);
}

#[test]
fn the_footer_keeps_its_close_button_clickable_on_every_row() {
    for presses in [0, 1, 2] {
        let mut h = opened(Locale::EnUs, TRUE_COLOR);
        for _ in 0..presses {
            h.press(KeyCode::Down);
        }
        let (column, row) = h.find("Esc: close").expect("footer button");
        h.click(column + 1, row);
        assert!(h.app.modal().is_none(), "after {presses} presses");
    }
}

#[test]
fn clicking_the_games_or_the_text_moves_the_focus_there() {
    let mut h = opened(Locale::EnUs, TRUE_COLOR);
    h.press(KeyCode::Up);
    let (column, row) = h.find("Spider").expect("game tab");
    h.click(column, row);
    assert_eq!(
        (state(&h).rules, state(&h).focus),
        (Rules::Spider, Focus::Games)
    );
    let parts = view::parts(geometry(AREA).body);
    h.click(parts.text.x + 10, parts.text.y + 3);
    assert_eq!(state(&h).focus, Focus::Text);
}
