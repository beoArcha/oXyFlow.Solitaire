//! The Close button with its key and the scroll bar on the windows other than Rules: the Keys tab,
//! the Command line tab and the guide window (the Rules window is tested in `how_to_play`).

use oxysol_core::{DrawCount, Variant};
use ratatui::crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

use super::{guide_window, help};
use crate::app::Modal;
use crate::caps::TerminalCaps;
use crate::glyphs::is_wgl4;
use crate::i18n::{self, Locale};
use crate::input::Input;
use crate::screens::Effect;
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, PLAIN, TRUE_COLOR};

const AREA: Rect = Rect::new(0, 0, 100, 30);
const CLOSE: [(Locale, &str); 2] = [
    (Locale::EnUs, "Close [Esc]"),
    (Locale::PlPl, "Zamknij [Esc]"),
];

#[derive(Clone, Copy, Debug)]
enum Window {
    Keys,
    CommandLine,
    Guide,
}

fn harness(caps: TerminalCaps) -> Harness {
    Harness::with(MemoryStorage::new(), caps, AREA.width, AREA.height)
}

fn open(window: Window, locale: Locale, caps: TerminalCaps) -> Harness {
    let mut h = harness(caps);
    i18n::set_locale(locale);
    match window {
        Window::Keys => h.type_char('?'),
        Window::CommandLine => {
            h.type_char('?');
            h.press(KeyCode::Left);
        }
        Window::Guide => {
            h.app.apply(Effect::Start {
                variant: Variant::Klondike(DrawCount::Three),
                seed: Some(5),
                undo_allowed: true,
            });
            h.type_char('g');
        }
    }
    h.lines();
    h
}

fn track(window: Window) -> Rect {
    let body = match window {
        Window::Guide => guide_window::geometry(AREA).body,
        Window::Keys | Window::CommandLine => help::geometry(AREA).body,
    };
    let (y, height) = match window {
        Window::Guide => (body.y, 17),
        Window::CommandLine => (body.y + 1, 21),
        Window::Keys => (body.y + 1, 22),
    };
    Rect::new(body.right() - 1, y, 1, height)
}

fn bar(h: &mut Harness, track: Rect) -> Vec<String> {
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

fn scroll_of(h: &Harness) -> u16 {
    match h.app.modal() {
        Some(Modal::CommandLine(state)) => state.scroll,
        Some(Modal::Guide(state)) => state.scroll,
        other => panic!("not a scrolling window: {other:?}"),
    }
}

fn mouse(h: &mut Harness, kind: MouseEventKind, track: Rect, row: u16) {
    h.send(Input::Mouse(MouseEvent {
        kind,
        column: track.x,
        row: track.y + row,
        modifiers: KeyModifiers::NONE,
    }));
}

fn left(kind: fn(MouseButton) -> MouseEventKind) -> MouseEventKind {
    kind(MouseButton::Left)
}

#[test]
fn every_close_button_shows_its_key_in_both_locales() {
    for (locale, label) in CLOSE {
        for window in [Window::Keys, Window::CommandLine, Window::Guide] {
            let mut h = open(window, locale, TRUE_COLOR);
            assert!(h.text().contains(label), "{window:?} {locale:?}");
            assert!(h.text().contains(&format!("► {label} ◄")), "{window:?}");
        }
    }
    i18n::set_locale(Locale::EnUs);
}

#[test]
fn the_close_button_with_its_key_is_still_clickable() {
    for (locale, label) in CLOSE {
        for window in [Window::Keys, Window::CommandLine, Window::Guide] {
            let mut h = open(window, locale, TRUE_COLOR);
            let (column, row) = h.find(label).expect("button");
            h.click(column + 2, row);
            assert!(h.app.modal().is_none(), "{window:?} {locale:?}");
        }
    }
    i18n::set_locale(Locale::EnUs);
}

#[test]
fn the_close_button_names_its_key_in_ascii_too() {
    for window in [Window::Keys, Window::CommandLine, Window::Guide] {
        let mut h = open(window, Locale::EnUs, PLAIN);
        assert!(h.text().contains("> Close [Esc] <"), "{window:?}");
    }
}

#[test]
fn the_keys_command_line_and_guide_windows_draw_a_scroll_bar_that_follows_the_text() {
    for window in [Window::Keys, Window::CommandLine, Window::Guide] {
        let mut h = open(window, Locale::EnUs, TRUE_COLOR);
        let column = bar(&mut h, track(window));
        assert_eq!(column[0], "█", "{window:?}");
        assert_eq!(column.last().map(String::as_str), Some("░"), "{window:?}");
        h.press(KeyCode::End);
        let column = bar(&mut h, track(window));
        assert_eq!(column[0], "░", "{window:?}");
        assert_eq!(column.last().map(String::as_str), Some("█"), "{window:?}");
    }
}

#[test]
fn the_scroll_bar_uses_ascii_symbols_in_the_ascii_set() {
    for window in [Window::CommandLine, Window::Guide] {
        let mut h = open(window, Locale::EnUs, PLAIN);
        let column = bar(&mut h, track(window));
        assert!(column.iter().all(|symbol| symbol == "#" || symbol == ":"));
        assert_eq!(column[0], "#");
    }
}

#[test]
fn a_click_on_the_track_pages_the_command_line_and_the_guide_windows() {
    for window in [Window::CommandLine, Window::Guide] {
        let mut h = open(window, Locale::EnUs, TRUE_COLOR);
        let track = track(window);
        mouse(&mut h, left(MouseEventKind::Down), track, track.height - 1);
        let paged = scroll_of(&h);
        assert!(paged > 0, "{window:?} did not page down");
        mouse(&mut h, left(MouseEventKind::Down), track, 0);
        assert!(scroll_of(&h) < paged, "{window:?} did not page up");
        assert!(h.app.modal().is_some());
    }
}

#[test]
fn dragging_the_thumb_scrolls_the_command_line_and_the_guide_windows_to_the_end() {
    for window in [Window::CommandLine, Window::Guide] {
        let mut h = open(window, Locale::EnUs, TRUE_COLOR);
        let track = track(window);
        mouse(&mut h, left(MouseEventKind::Down), track, 0);
        assert_eq!(scroll_of(&h), 0);
        mouse(&mut h, left(MouseEventKind::Drag), track, track.height - 1);
        let end = scroll_of(&h);
        assert!(end > 0, "{window:?}");
        h.press(KeyCode::End);
        assert_eq!(scroll_of(&h), end, "{window:?} drag reached the end");
        mouse(&mut h, left(MouseEventKind::Up), track, track.height - 1);
        mouse(&mut h, left(MouseEventKind::Drag), track, 0);
        assert_eq!(scroll_of(&h), end, "{window:?} drag after the release");
    }
}

#[test]
fn the_wheel_keeps_working_on_the_command_line_and_the_guide_windows() {
    for window in [Window::CommandLine, Window::Guide] {
        let mut h = open(window, Locale::EnUs, TRUE_COLOR);
        let track = track(window);
        mouse(&mut h, MouseEventKind::ScrollDown, track, 5);
        assert_eq!(scroll_of(&h), 3, "{window:?}");
    }
}

fn assert_renders_cleanly(window: Window, locale: Locale, caps: TerminalCaps) {
    let mut h = open(window, locale, caps);
    for step in [None, Some(KeyCode::PageDown), Some(KeyCode::End)] {
        if let Some(code) = step {
            h.press(code);
        }
        let lines = h.lines();
        assert_eq!(lines.len(), 30);
        let drawn = lines.iter().flat_map(|line| line.chars());
        for ch in drawn {
            assert!(ch.is_ascii() || is_wgl4(ch), "{window:?} {locale:?} {ch:?}");
        }
    }
}

#[test]
fn every_window_renders_at_100_by_30_in_both_locales_with_only_wgl4_or_ascii() {
    for locale in Locale::offered() {
        for caps in [TRUE_COLOR, PLAIN] {
            for window in [Window::Keys, Window::CommandLine, Window::Guide] {
                assert_renders_cleanly(window, locale, caps);
            }
        }
    }
    i18n::set_locale(Locale::EnUs);
}
