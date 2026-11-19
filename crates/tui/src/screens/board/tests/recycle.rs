use oxysol_core::{DrawCount, PyramidRules, Variant};
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;

use super::*;
use crate::storage::memory::MemoryStorage;
use crate::test_support::PLAIN;

const KLONDIKE: Variant = Variant::Klondike(DrawCount::One);
const CLASSIC: Variant = Variant::Pyramid(PyramidRules::Classic);
const RECYCLE_UNICODE: char = '↑';
const RECYCLE_ASCII: char = '^';
const SPENT: char = 'x';
const SEED: u64 = 5;

fn start_ascii(variant: Variant, seed: u64) -> Harness {
    let mut h = Harness::with(MemoryStorage::default(), PLAIN, 120, 40);
    h.app.apply(crate::screens::Effect::Start {
        variant,
        seed: Some(seed),
        undo_allowed: true,
    });
    h
}

fn empty_stock(h: &mut Harness) {
    for _ in 0..pile_len(h, PileRef::Stock) {
        h.type_char('s');
    }
    assert_eq!(pile_len(h, PileRef::Stock), 0);
}

fn stock_slot(h: &mut Harness) -> Rect {
    h.lines();
    let board = board(h);
    let piles = board.piles();
    board
        .geometry(&piles)
        .and_then(|geometry| geometry.pile(PileRef::Stock).map(|placed| placed.slot))
        .unwrap_or_default()
}

fn slot_symbols(h: &mut Harness) -> String {
    let slot = stock_slot(h);
    let buffer = h.terminal.backend().buffer();
    (slot.top()..slot.bottom())
        .flat_map(|y| (slot.left()..slot.right()).map(move |x| buffer[(x, y)].symbol().to_owned()))
        .collect()
}

fn slot_centre(h: &mut Harness) -> (u16, u16) {
    let slot = stock_slot(h);
    (slot.x + slot.width / 2, slot.y + slot.height / 2)
}

#[test]
fn an_empty_stock_with_a_pass_left_shows_the_recycle_sign() {
    let mut h = start(KLONDIKE, SEED);
    assert!(!slot_symbols(&mut h).contains(RECYCLE_UNICODE));
    empty_stock(&mut h);
    let symbols = slot_symbols(&mut h);
    assert!(symbols.contains(RECYCLE_UNICODE), "{symbols:?}");
    assert!(!symbols.contains(SPENT));
}

#[test]
fn the_recycle_sign_has_an_ascii_form() {
    let mut h = start_ascii(KLONDIKE, SEED);
    empty_stock(&mut h);
    let symbols = slot_symbols(&mut h);
    assert!(symbols.contains(RECYCLE_ASCII), "{symbols:?}");
    assert!(!symbols.contains(RECYCLE_UNICODE));
}

#[test]
fn an_empty_stock_without_a_pass_shows_the_spent_sign_not_the_recycle_sign() {
    let mut h = start(CLASSIC, SEED);
    empty_stock(&mut h);
    let symbols = slot_symbols(&mut h);
    assert!(symbols.contains(SPENT), "{symbols:?}");
    assert!(!symbols.contains(RECYCLE_UNICODE));
}

#[test]
fn a_click_on_the_empty_stock_turns_the_waste_back() {
    let mut h = start(KLONDIKE, SEED);
    empty_stock(&mut h);
    let waste = pile_len(&h, PileRef::Waste);
    let moves = moves_made(&h);
    let (column, row) = slot_centre(&mut h);
    h.click(column, row);
    assert_eq!(pile_len(&h, PileRef::Stock), waste);
    assert_eq!(pile_len(&h, PileRef::Waste), 0);
    assert_eq!(moves_made(&h), moves + 1);
}

#[test]
fn enter_on_the_empty_stock_turns_the_waste_back() {
    let mut h = start(KLONDIKE, SEED);
    empty_stock(&mut h);
    let waste = pile_len(&h, PileRef::Waste);
    put_cursor(&mut h, PileRef::Stock);
    h.press(KeyCode::Enter);
    assert_eq!(pile_len(&h, PileRef::Stock), waste);
    assert_eq!(pile_len(&h, PileRef::Waste), 0);
}

#[test]
fn without_a_pass_enter_on_the_empty_stock_changes_nothing() {
    let mut h = start(CLASSIC, SEED);
    empty_stock(&mut h);
    let waste = pile_len(&h, PileRef::Waste);
    let moves = moves_made(&h);
    put_cursor(&mut h, PileRef::Stock);
    h.press(KeyCode::Enter);
    assert_eq!(pile_len(&h, PileRef::Waste), waste);
    assert_eq!(moves_made(&h), moves);
}
