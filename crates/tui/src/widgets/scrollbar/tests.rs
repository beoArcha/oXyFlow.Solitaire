//! The thumb's size and place, the mouse on the bar and its drawing in both symbol sets.

use std::time::Instant;

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

use super::*;
use crate::audio::AudioStatus;
use crate::glyphs::{Glyphs, is_wgl4};
use crate::screens::ViewCtx;
use crate::settings::{GlyphMode, Settings};
use crate::test_support::TRUE_COLOR;
use crate::theme::{Theme, ThemeId};

const TRACK: Rect = Rect::new(9, 4, 1, 20);

fn metrics(total: usize, offset: usize) -> Metrics {
    Metrics {
        total,
        view: 20,
        offset,
    }
}

fn mouse(kind: MouseEventKind, row: u16) -> MouseEvent {
    MouseEvent {
        kind,
        column: TRACK.x,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

fn left(kind: fn(MouseButton) -> MouseEventKind, row: u16) -> MouseEvent {
    mouse(kind(MouseButton::Left), row)
}

#[test]
fn text_that_fits_has_no_thumb() {
    assert_eq!(metrics(20, 0).thumb(20), None);
    assert_eq!(metrics(5, 0).thumb(20), None);
    assert_eq!(metrics(40, 0).thumb(0), None);
}

#[test]
fn the_thumb_is_the_visible_share_of_the_track() {
    assert_eq!(metrics(40, 0).thumb(20), Some((0, 10)));
    assert_eq!(metrics(37, 0).thumb(20), Some((0, 11)));
    assert_eq!(metrics(2000, 0).thumb(20), Some((0, 1)));
}

#[test]
fn the_thumb_follows_the_position_from_the_top_to_the_bottom() {
    assert_eq!(metrics(40, 0).thumb(20), Some((0, 10)));
    assert_eq!(metrics(40, 10).thumb(20), Some((5, 10)));
    assert_eq!(metrics(40, 20).thumb(20), Some((10, 10)));
    let (start, size) = metrics(37, 17).thumb(20).expect("scrolls");
    assert_eq!(start + size, 20);
}

#[test]
fn the_thumb_stays_inside_the_track_for_every_position() {
    for total in 21..120 {
        for offset in 0..=total - 20 {
            let (start, size) = metrics(total, offset).thumb(20).expect("scrolls");
            assert!(size >= 1 && start + size <= 20, "{total}/{offset}");
        }
    }
}

#[test]
fn a_click_above_the_thumb_pages_up_and_below_it_pages_down() {
    let mut drag = None;
    let at = metrics(60, 20);
    let (start, size) = at.thumb(20).expect("scrolls");
    let above = left(MouseEventKind::Down, TRACK.y + start - 1);
    assert_eq!(handle_mouse(&above, TRACK, at, &mut drag), Some(1));
    let below = left(MouseEventKind::Down, TRACK.y + start + size);
    assert_eq!(handle_mouse(&below, TRACK, at, &mut drag), Some(39));
    assert_eq!(drag, None);
}

#[test]
fn paging_stops_at_both_ends() {
    let mut drag = None;
    let near_end = metrics(30, 8);
    let (start, size) = near_end.thumb(20).expect("scrolls");
    let below = left(MouseEventKind::Down, TRACK.y + start + size);
    assert_eq!(handle_mouse(&below, TRACK, near_end, &mut drag), Some(10));
    let near_top = metrics(60, 3);
    let (start, _) = near_top.thumb(20).expect("scrolls");
    assert!(start > 0);
    let above = left(MouseEventKind::Down, TRACK.y + start - 1);
    assert_eq!(handle_mouse(&above, TRACK, near_top, &mut drag), Some(0));
}

#[test]
fn a_click_beside_the_track_does_nothing() {
    let mut drag = None;
    let beside = MouseEvent {
        column: TRACK.x - 1,
        ..left(MouseEventKind::Down, TRACK.y + 19)
    };
    assert_eq!(
        handle_mouse(&beside, TRACK, metrics(60, 0), &mut drag),
        None
    );
}

#[test]
fn dragging_the_thumb_scrolls_and_the_release_ends_it() {
    let mut drag = None;
    let at = metrics(60, 0);
    let press = left(MouseEventKind::Down, TRACK.y + 1);
    assert_eq!(handle_mouse(&press, TRACK, at, &mut drag), None);
    assert_eq!(drag, Some(1));
    let middle = left(MouseEventKind::Drag, TRACK.y + 1 + 7);
    let moved = handle_mouse(&middle, TRACK, at, &mut drag).expect("drag scrolls");
    assert!(moved > 0 && moved <= 40, "{moved}");
    let bottom = left(MouseEventKind::Drag, TRACK.y + 40);
    assert_eq!(handle_mouse(&bottom, TRACK, at, &mut drag), Some(40));
    let top = left(MouseEventKind::Drag, 0);
    assert_eq!(handle_mouse(&top, TRACK, at, &mut drag), Some(0));
    assert_eq!(
        handle_mouse(&left(MouseEventKind::Up, TRACK.y), TRACK, at, &mut drag),
        None
    );
    assert_eq!(drag, None);
    assert_eq!(handle_mouse(&middle, TRACK, at, &mut drag), None);
}

#[test]
fn the_track_is_the_last_column_of_the_text() {
    assert_eq!(track_of(Rect::new(2, 3, 30, 17)), Rect::new(31, 3, 1, 17));
    assert_eq!(track_of(Rect::new(2, 3, 0, 17)).width, 0);
}

fn drawn(mode: GlyphMode, at: Metrics) -> Vec<String> {
    let glyphs = Glyphs::new(mode);
    let theme = Theme::new(ThemeId::Dark, TRUE_COLOR.color);
    let settings = Settings::default();
    let ctx = ViewCtx {
        theme: &theme,
        glyphs: &glyphs,
        settings: &settings,
        caps: &TRUE_COLOR,
        audio_status: AudioStatus::NoDevice,
        sound_styles: &[],
        saved: None,
        pointer: None,
        now: Instant::now(),
        font: crate::app::FontFacts::default(),
    };
    let mut terminal = Terminal::new(TestBackend::new(12, 24)).expect("terminal");
    terminal
        .draw(|frame| draw(frame, TRACK, at, &ctx))
        .expect("draw");
    let buffer = terminal.backend().buffer();
    (TRACK.y..TRACK.bottom())
        .map(|row| buffer[(TRACK.x, row)].symbol().to_owned())
        .collect()
}

#[test]
fn the_bar_draws_a_thumb_in_a_groove_in_both_symbol_sets() {
    for (mode, thumb, groove) in [(GlyphMode::Unicode, "█", "░"), (GlyphMode::Ascii, "#", ":")]
    {
        let column = drawn(mode, metrics(40, 20));
        assert_eq!(
            column.iter().filter(|c| *c == thumb).count(),
            10,
            "{mode:?}"
        );
        assert_eq!(
            column.iter().filter(|c| *c == groove).count(),
            10,
            "{mode:?}"
        );
        assert_eq!(column[0], groove);
        assert_eq!(column[19], thumb);
        let ascii = mode == GlyphMode::Ascii;
        assert!(column.iter().all(|c| c.chars().all(|ch| if ascii {
            ch.is_ascii()
        } else {
            is_wgl4(ch)
        })));
    }
}

#[test]
fn a_text_that_fits_draws_no_bar() {
    assert!(
        drawn(GlyphMode::Unicode, metrics(20, 0))
            .iter()
            .all(|c| c == " ")
    );
}
