//! The card size the player chose: the limit on the largest size and its fallback.

use oxysol_core::{DrawCount, PyramidRules, Session, Variant};

use super::*;

const KLONDIKE: Variant = Variant::Klondike(DrawCount::One);
const PYRAMID: Variant = Variant::Pyramid(PyramidRules::Relaxed);

fn card(variant: Variant, area: Rect, limit: Option<CardSize>) -> CardSize {
    let piles = Session::new(variant, 3).game().piles();
    compute_within(variant, &piles, area, limit).card
}

#[test]
fn no_limit_is_the_largest_size_that_fits() {
    let area = Rect::new(0, 0, 160, 47);
    let piles = Session::new(KLONDIKE, 3).game().piles();
    assert_eq!(compute(KLONDIKE, &piles, area).card, SIZES[0]);
    assert_eq!(card(KLONDIKE, area, None), SIZES[0]);
}

#[test]
fn a_limit_that_fits_is_used_and_a_larger_window_does_not_exceed_it() {
    let area = Rect::new(0, 0, 160, 47);
    for limit in SIZES {
        assert_eq!(card(KLONDIKE, area, Some(limit)), limit);
    }
    for limit in &SIZES[1..] {
        assert_eq!(card(PYRAMID, area, Some(*limit)), *limit);
    }
}

#[test]
fn a_limit_that_does_not_fit_falls_back_to_the_largest_that_does() {
    let area = Rect::new(0, 0, 100, 27);
    let largest = card(KLONDIKE, area, None);
    assert_ne!(largest, SIZES[0]);
    assert_eq!(card(KLONDIKE, area, Some(SIZES[0])), largest);
    assert_eq!(card(KLONDIKE, area, Some(largest)), largest);
    assert_eq!(
        card(KLONDIKE, area, Some(CardSize::smallest())),
        CardSize::smallest()
    );
}

#[test]
fn nothing_fits_gives_the_smallest_size_whatever_the_limit() {
    let area = Rect::new(0, 0, 40, 10);
    for limit in SIZES {
        assert_eq!(card(KLONDIKE, area, Some(limit)), CardSize::smallest());
    }
}

#[test]
fn the_field_frame_takes_a_cell_from_each_side_before_the_size_is_decided() {
    let screen = Rect::new(0, 0, 120, 38);
    let plain = chrome_with(screen, false, false).body;
    let framed = chrome_with(screen, false, true).body;
    assert_eq!(framed.height + 2 * FRAME_INSET, plain.height);
    assert_eq!(card(KLONDIKE, plain, None), SIZES[1]);
    assert_eq!(card(KLONDIKE, framed, None), SIZES[2]);
}
