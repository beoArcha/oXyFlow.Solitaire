//! The scroll bar of the Rules tab: its size and place, its ASCII form, the click on the track, the
//! drag of the thumb, the wheel, and the renders at 100 by 30.

use ratatui::crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

use super::focus_tests::{AREA, opened, state};
use super::state::Focus;
use super::*;
use crate::app::Modal;
use crate::i18n::{self, Locale};
use crate::input::Input;
use crate::test_support::{Harness, PLAIN, TRUE_COLOR};

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
