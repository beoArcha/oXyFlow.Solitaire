//! The focus of the Rules tab: its order, what Left and Right do on each part, how each part is
//! drawn without color, what the footer says and the scroll bar of the text.

use ratatui::crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use ratatui::style::Modifier;

use super::state::Focus;
use super::*;
use crate::app::Modal;
use crate::caps::TerminalCaps;
use crate::i18n::{self, Locale};
use crate::input::Input;
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, PLAIN, TRUE_COLOR};

const AREA: Rect = Rect::new(0, 0, 100, 30);

fn opened(locale: Locale, caps: TerminalCaps) -> Harness {
    let mut h = Harness::with(MemoryStorage::new(), caps, AREA.width, AREA.height);
    i18n::set_locale(locale);
    h.type_char('i');
    h.lines();
    h
}

fn state(h: &Harness) -> HowToPlay {
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

fn mouse(h: &mut Harness, kind: MouseEventKind, column: u16, row: u16) {
    h.send(Input::Mouse(MouseEvent {
        kind,
        column,
        row,
        modifiers: KeyModifiers::NONE,
    }));
}

fn down(h: &mut Harness, column: u16, row: u16) {
    mouse(h, MouseEventKind::Down(MouseButton::Left), column, row);
}

fn track() -> Rect {
    let parts = view::parts(geometry(AREA).body);
    crate::widgets::scrollbar::track_of(parts.text)
}

fn bar(h: &mut Harness) -> Vec<String> {
    let track = track();
    let lines = h.lines();
    (track.y..track.bottom())
        .map(|row| {
            lines[usize::from(row)]
                .chars()
                .nth(usize::from(track.x))
                .map_or_else(String::new, |symbol| symbol.to_string())
        })
        .collect()
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
        (Locale::PlPl, "Left/Right: zakładka", "Left/Right: gra"),
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

#[test]
fn the_scroll_bar_shows_the_visible_share_and_follows_the_text() {
    let mut h = opened(Locale::EnUs, TRUE_COLOR);
    let column = bar(&mut h);
    assert_eq!(column.len(), 20);
    let total = state(&h).metrics(20).total;
    let thumb = column.iter().filter(|symbol| *symbol == "█").count();
    assert_eq!(thumb, (20 * 20 + total / 2) / total);
    assert_eq!(column[0], "█");
    assert_eq!(column[19], "░");
    h.press(KeyCode::End);
    let column = bar(&mut h);
    assert_eq!(column[0], "░");
    assert_eq!(column[19], "█");
    assert_eq!(column.iter().filter(|symbol| *symbol == "█").count(), thumb);
}

#[test]
fn the_scroll_bar_is_ascii_in_the_ascii_set() {
    let mut h = opened(Locale::EnUs, PLAIN);
    let column = bar(&mut h);
    assert_eq!(column[0], "#");
    assert_eq!(column[19], ":");
    assert!(column.iter().all(|symbol| symbol == "#" || symbol == ":"));
}

#[test]
fn a_click_on_the_track_pages_and_a_drag_of_the_thumb_scrolls() {
    let mut h = opened(Locale::EnUs, TRUE_COLOR);
    let track = track();
    let max = state(&h).max_scroll();
    down(&mut h, track.x, track.y + 19);
    assert_eq!(state(&h).scroll, max.min(19));
    assert_eq!(state(&h).focus, Focus::Text);
    down(&mut h, track.x, track.y);
    assert_eq!(state(&h).scroll, 0, "a click above the thumb pages up");

    down(&mut h, track.x, track.y);
    mouse(
        &mut h,
        MouseEventKind::Drag(MouseButton::Left),
        track.x,
        track.y + 19,
    );
    assert_eq!(state(&h).scroll, max);
    mouse(
        &mut h,
        MouseEventKind::Up(MouseButton::Left),
        track.x,
        track.y + 19,
    );
    mouse(
        &mut h,
        MouseEventKind::Drag(MouseButton::Left),
        track.x,
        track.y,
    );
    assert_eq!(
        state(&h).scroll,
        max,
        "a drag after the release does nothing"
    );
}

#[test]
fn the_wheel_still_scrolls_and_a_click_on_the_bar_does_not_close_the_window() {
    let mut h = opened(Locale::EnUs, TRUE_COLOR);
    let track = track();
    mouse(&mut h, MouseEventKind::ScrollDown, 50, 15);
    assert_eq!(state(&h).scroll, 3);
    down(&mut h, track.x, track.y + 10);
    assert!(matches!(h.app.modal(), Some(Modal::HowToPlay(_))));
}

#[test]
fn every_state_of_the_window_renders_at_100_by_30_in_both_locales() {
    for locale in Locale::ALL {
        for caps in [TRUE_COLOR, PLAIN] {
            let mut h = opened(locale, caps);
            for step in [KeyCode::Up, KeyCode::Down, KeyCode::Down, KeyCode::End] {
                h.press(step);
                let lines = h.lines();
                assert_eq!(lines.len(), 30);
                for ch in lines.iter().flat_map(|line| line.chars()) {
                    assert!(ch.is_ascii() || crate::glyphs::is_wgl4(ch), "{ch:?}");
                }
                let close = i18n::tf("help.rules.close", &[("key", "Esc")]);
                assert!(lines.join("\n").contains(&close), "{locale:?} {step:?}");
            }
        }
    }
    i18n::set_locale(Locale::EnUs);
}
