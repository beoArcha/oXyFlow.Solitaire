use ratatui::layout::{Margin, Rect};

use super::*;

fn area(width: u16, height: u16) -> Rect {
    Rect::new(0, 0, width, height)
}

#[test]
fn the_class_follows_the_size_at_every_draw() {
    let cases = [
        ((100, 30), Class::Compact),
        ((129, 50), Class::Compact),
        ((115, 35), Class::Compact),
        ((130, 30), Class::Standard),
        ((130, 35), Class::Standard),
        ((200, 30), Class::Wide),
        ((130, 36), Class::Standard),
        ((179, 60), Class::Standard),
        ((180, 36), Class::Wide),
        ((200, 60), Class::Wide),
    ];
    for ((width, height), class) in cases {
        assert_eq!(Class::of(area(width, height)), class, "{width}x{height}");
    }
}

#[test]
fn a_compact_screen_has_a_header_row_and_three_chrome_rows() {
    let parts = chrome_with(area(100, 30), true, false);
    assert_eq!(parts.class, Class::Compact);
    assert_eq!(parts.header, Some(Rect::new(0, 0, 100, 1)));
    assert_eq!((parts.body.y, parts.body.height), (1, 27));
    assert_eq!((parts.now.y, parts.possible.y), (28, 29));
    assert_eq!((parts.guide, parts.body.width), (None, 100));
}

#[test]
fn a_standard_screen_has_two_chrome_rows_and_a_full_height_panel() {
    let parts = chrome_with(area(130, 40), true, false);
    assert_eq!(parts.header, None);
    assert_eq!(parts.guide, Some(Rect::new(100, 0, GUIDE_WIDTH, 40)));
    assert_eq!(parts.body, Rect::new(0, 0, 100, 38));
    assert_eq!(
        (parts.now, parts.possible),
        (Rect::new(0, 38, 100, 1), Rect::new(0, 39, 100, 1))
    );
}

#[test]
fn a_short_wide_screen_keeps_the_panel_and_a_board_of_the_minimum_width() {
    let parts = chrome_with(area(130, 30), true, false);
    assert_eq!(parts.class, Class::Standard);
    assert_eq!(parts.header, None);
    assert_eq!(parts.guide, Some(Rect::new(100, 0, GUIDE_WIDTH, 30)));
    assert_eq!(parts.body, Rect::new(0, 0, 100, 28));
    assert!(parts.body.width >= BOARD_MIN_WIDTH);
}

#[test]
fn a_tall_narrow_screen_has_no_panel_whatever_its_height() {
    let parts = chrome_with(area(129, 60), true, false);
    assert_eq!((parts.guide, parts.body.width), (None, 129));
    assert!(parts.header.is_some());
}

#[test]
fn a_wide_screen_has_a_wider_panel_and_the_board_keeps_its_width() {
    let parts = chrome_with(area(200, 60), true, false);
    assert_eq!(parts.guide, Some(Rect::new(160, 0, GUIDE_WIDTH_WIDE, 60)));
    assert_eq!(parts.body.width, 160);
    let edge = chrome_with(area(WIDE_MIN_WIDTH, 50), true, false);
    assert_eq!(edge.body.width, WIDE_MIN_WIDTH - GUIDE_WIDTH_WIDE);
    assert!(edge.body.width >= BOARD_MIN_WIDTH);
}

#[test]
fn hiding_the_panel_brings_the_header_row_back() {
    for (width, height) in [(130, 40), (200, 60)] {
        let parts = chrome_with(area(width, height), false, false);
        assert_eq!(parts.guide, None);
        assert_eq!(parts.header, Some(Rect::new(0, 0, width, 1)));
        assert_eq!(parts.body.width, width);
    }
}

#[test]
fn the_moves_panel_alone_takes_the_side_column_and_brings_the_header_row_back() {
    let parts = chrome_panels(area(130, 40), false, true, false);
    assert_eq!(parts.guide, None);
    assert_eq!(parts.moves, Some(Rect::new(100, 0, GUIDE_WIDTH, 40)));
    assert_eq!(parts.header, Some(Rect::new(0, 0, 100, 1)));
    assert_eq!(parts.body.width, 100);
}

#[test]
fn both_panels_share_the_column_with_the_guide_above() {
    let parts = chrome_panels(area(130, 40), true, true, false);
    assert_eq!(parts.header, None);
    assert_eq!(parts.guide, Some(Rect::new(100, 0, GUIDE_WIDTH, 28)));
    assert_eq!(
        parts.moves,
        Some(Rect::new(100, 28, GUIDE_WIDTH, MOVES_ROWS))
    );
    let tall = chrome_panels(area(200, 60), true, true, false);
    assert_eq!(
        tall.moves,
        Some(Rect::new(160, 38, GUIDE_WIDTH_WIDE, MOVES_ROWS_TALL))
    );
    assert_eq!(tall.body.width, 160);
}

#[test]
fn a_short_column_gives_the_moves_panel_at_most_half_of_it() {
    let parts = chrome_panels(area(130, 20), true, true, false);
    assert_eq!(parts.moves.map(|rect| rect.height), Some(10));
}

#[test]
fn a_compact_screen_has_no_moves_panel() {
    let parts = chrome_panels(area(100, 30), true, true, false);
    assert_eq!((parts.guide, parts.moves), (None, None));
    assert_eq!(parts.body.width, 100);
}

#[test]
fn the_moves_panel_does_not_change_the_board_width() {
    for (width, height) in [(130, 40), (179, 40), (200, 60)] {
        let with = chrome_panels(area(width, height), true, true, false);
        let without = chrome_with(area(width, height), true, false);
        assert_eq!(with.body.width, without.body.width, "{width}x{height}");
    }
}

#[test]
fn without_a_frame_the_body_is_the_field_in_every_size() {
    for (width, height) in [(100, 30), (129, 30), (130, 30), (180, 30), (200, 60)] {
        for guide in [false, true] {
            let parts = chrome_with(area(width, height), guide, false);
            assert_eq!(parts.body, parts.field, "{width}x{height} guide {guide}");
        }
    }
}

#[test]
fn the_frame_leaves_the_field_alone_and_insets_the_body_by_one_cell() {
    let plain = chrome_with(area(130, 40), true, false);
    let framed = chrome_with(area(130, 40), true, true);
    assert_eq!(framed.field, plain.field);
    assert_eq!(framed.body, Rect::new(1, 1, 98, 36));
    assert_eq!(
        (framed.header, framed.guide, framed.now, framed.possible),
        (plain.header, plain.guide, plain.now, plain.possible)
    );
}

#[test]
fn the_framed_body_is_the_field_inside_a_one_cell_frame_in_every_size() {
    for (width, height) in [
        (100, 30),
        (129, 30),
        (130, 30),
        (179, 40),
        (180, 30),
        (200, 60),
    ] {
        for guide in [false, true] {
            let parts = chrome_with(area(width, height), guide, true);
            let inside = Rect::new(
                parts.field.x + FRAME_INSET,
                parts.field.y + FRAME_INSET,
                parts.field.width - 2 * FRAME_INSET,
                parts.field.height - 2 * FRAME_INSET,
            );
            assert_eq!(parts.body, inside, "{width}x{height} guide {guide}");
            assert_eq!(parts.now.width, parts.field.width);
        }
    }
}

#[test]
fn the_frame_and_the_moves_panel_combine_without_touching_each_other() {
    for (width, height) in [(130, 40), (179, 40), (200, 60)] {
        let plain = chrome_panels(area(width, height), true, true, false);
        let framed = chrome_panels(area(width, height), true, true, true);
        assert_eq!(framed.field, plain.field, "{width}x{height}");
        assert_eq!(framed.body, plain.field.inner(Margin::new(1, 1)));
        assert_eq!(
            (framed.guide, framed.moves, framed.now, framed.possible),
            (plain.guide, plain.moves, plain.now, plain.possible)
        );
    }
}
