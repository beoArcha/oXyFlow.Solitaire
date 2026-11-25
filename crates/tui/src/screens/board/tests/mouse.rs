//! Mouse gestures on the board: synthetic events on the geometry of a 120 × 40 terminal, driven
//! through `App`.

use std::time::Duration;

use oxysol_core::{DrawCount, PileRef, PyramidRules, Variant};
use ratatui::crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

use super::*;
use crate::app::Screen;
use crate::input::Input;
use crate::layout::{BoardGeometry, Hit};
use crate::settings::{Aids, Settings};

const KLONDIKE: Variant = Variant::Klondike(DrawCount::One);
const RELAXED: Variant = Variant::Pyramid(PyramidRules::Relaxed);
const PAIR_SEED: u64 = 13;
const HEADER: (u16, u16) = (0, 0);

fn send(h: &mut Harness, kind: MouseEventKind, cell: (u16, u16)) {
    h.send(Input::Mouse(MouseEvent {
        kind,
        column: cell.0,
        row: cell.1,
        modifiers: KeyModifiers::NONE,
    }));
}

fn press(h: &mut Harness, cell: (u16, u16)) {
    send(h, MouseEventKind::Down(MouseButton::Left), cell);
}

fn release(h: &mut Harness, cell: (u16, u16)) {
    send(h, MouseEventKind::Up(MouseButton::Left), cell);
}

fn click(h: &mut Harness, cell: (u16, u16)) {
    press(h, cell);
    release(h, cell);
}

fn drag_to(h: &mut Harness, cell: (u16, u16)) {
    send(h, MouseEventKind::Drag(MouseButton::Left), cell);
}

fn right_click(h: &mut Harness, cell: (u16, u16)) {
    send(h, MouseEventKind::Down(MouseButton::Right), cell);
}

fn wait(h: &mut Harness, millis: u64) {
    h.now += Duration::from_millis(millis);
}

fn geometry(h: &Harness) -> BoardGeometry {
    let piles = session(h).game().piles();
    match board(h).geometry(&piles) {
        Some(geometry) => geometry,
        None => unreachable!("every test starts a game"),
    }
}

fn cells(area: Rect) -> impl Iterator<Item = (u16, u16)> {
    (area.y..area.bottom())
        .flat_map(move |row| (area.x..area.right()).map(move |column| (column, row)))
}

/// The cell that hits card `index` of `pile`, if one is visible.
fn card_cell(h: &Harness, pile: PileRef, index: usize) -> Option<(u16, u16)> {
    let geometry = geometry(h);
    let rect = *geometry.pile(pile)?.cards.get(index)?;
    let wanted = Hit {
        pile,
        card: Some(index),
    };
    cells(rect).find(|&(column, row)| geometry.hit(column, row) == Some(wanted))
}

/// A cell of `pile`, on a card or on its empty area.
fn pile_cell(h: &Harness, pile: PileRef) -> Option<(u16, u16)> {
    let geometry = geometry(h);
    let area = geometry.pile(pile)?.area;
    cells(area).find(|&(column, row)| {
        geometry
            .hit(column, row)
            .is_some_and(|hit| hit.pile == pile)
    })
}

fn must(cell: Option<(u16, u16)>) -> (u16, u16) {
    match cell {
        Some(cell) => cell,
        None => unreachable!("the target is visible on the board"),
    }
}

fn first_move(h: &Harness) -> (PileRef, PileRef, u8) {
    match first_transfer(h) {
        Some(found) => found,
        None => unreachable!("the seed offers a transfer"),
    }
}

fn targets(h: &Harness) -> Vec<PileRef> {
    board(h).marks().targets
}

#[test]
fn a_click_picks_a_card_and_a_second_click_drops_it() {
    let mut h = start(KLONDIKE, 5);
    let (from, to, count) = first_move(&h);
    let before = pile_len(&h, from);
    let index = before - usize::from(count);
    let cell = must(card_cell(&h, from, index));
    click(&mut h, cell);
    assert_eq!(cursor(&h), (from, usize::from(count)));
    assert!(targets(&h).contains(&to));
    assert_eq!(moves_made(&h), 0);
    let cell = must(pile_cell(&h, to));
    click(&mut h, cell);
    assert_eq!(moves_made(&h), 1);
    assert_eq!(pile_len(&h, from), before - usize::from(count));
    assert!(targets(&h).is_empty());
}

#[test]
fn a_drag_from_a_card_onto_its_target_moves_it() {
    let mut h = start(KLONDIKE, 5);
    let (from, to, count) = first_move(&h);
    let index = pile_len(&h, from) - usize::from(count);
    let cell = must(card_cell(&h, from, index));
    press(&mut h, cell);
    assert!(targets(&h).contains(&to));
    drag_to(&mut h, HEADER);
    let cell = must(pile_cell(&h, to));
    drag_to(&mut h, cell);
    let cell = must(pile_cell(&h, to));
    release(&mut h, cell);
    assert_eq!(moves_made(&h), 1);
    assert!(targets(&h).is_empty());
}

#[test]
fn a_release_on_the_source_pile_keeps_the_pick() {
    let mut h = start(KLONDIKE, 5);
    let (from, to, _) = first_move(&h);
    let source = must(card_cell(&h, from, pile_len(&h, from) - 1));
    press(&mut h, source);
    release(&mut h, source);
    assert!(targets(&h).contains(&to));
    assert_eq!(moves_made(&h), 0);
}

#[test]
fn a_click_on_the_stock_draws_a_card() {
    let mut h = start(KLONDIKE, 5);
    let waste = pile_len(&h, PileRef::Waste);
    let cell = must(pile_cell(&h, PileRef::Stock));
    click(&mut h, cell);
    assert_eq!(pile_len(&h, PileRef::Waste), waste + 1);
    assert_eq!(moves_made(&h), 1);
}

#[test]
fn a_double_click_on_a_card_quick_moves_it() {
    let mut h = start(KLONDIKE, 5);
    let (from, _, count) = first_move(&h);
    let before = pile_len(&h, from);
    let source = must(card_cell(&h, from, before - usize::from(count)));
    click(&mut h, source);
    wait(&mut h, 150);
    click(&mut h, source);
    assert_eq!(moves_made(&h), 1);
    assert_eq!(pile_len(&h, from), before - usize::from(count));
}

#[test]
fn a_double_click_does_nothing_when_quick_move_is_off() {
    let mut h = start(KLONDIKE, 5);
    h.app.apply(Effect::Quick(Aids {
        quick_move: false,
        ..Settings::default().aids()
    }));
    let (from, _, count) = first_move(&h);
    let before = pile_len(&h, from);
    let source = must(card_cell(&h, from, before - usize::from(count)));
    click(&mut h, source);
    wait(&mut h, 150);
    click(&mut h, source);
    assert_eq!(moves_made(&h), 0);
    assert_eq!(pile_len(&h, from), before);
    assert_eq!(status(&h), Some(crate::screens::board::status::QUICK_OFF));
}

#[test]
fn the_double_click_window_is_400_milliseconds() {
    let mut h = start(KLONDIKE, 5);
    let (from, _, count) = first_move(&h);
    let source = must(card_cell(&h, from, pile_len(&h, from) - usize::from(count)));
    click(&mut h, source);
    wait(&mut h, 401);
    click(&mut h, source);
    assert_eq!(moves_made(&h), 0);
    assert!(targets(&h).is_empty());
}

#[test]
fn a_second_click_on_the_source_within_the_window_is_still_a_double_click() {
    let mut h = start(KLONDIKE, 5);
    let (from, _, count) = first_move(&h);
    let before = pile_len(&h, from);
    let source = must(card_cell(&h, from, before - usize::from(count)));
    click(&mut h, source);
    wait(&mut h, 400);
    click(&mut h, source);
    assert_eq!(moves_made(&h), 1);
}

#[test]
fn a_right_click_cancels_the_pick() {
    let mut h = start(KLONDIKE, 5);
    let (from, _, count) = first_move(&h);
    let source = must(card_cell(&h, from, pile_len(&h, from) - usize::from(count)));
    click(&mut h, source);
    assert!(!targets(&h).is_empty());
    right_click(&mut h, source);
    assert!(targets(&h).is_empty());
    assert_eq!(moves_made(&h), 0);
    assert_eq!(h.app.screen(), Screen::Board);
}

#[test]
fn a_right_click_without_a_pick_leaves_the_board_alone() {
    let mut h = start(KLONDIKE, 5);
    let before = cursor(&h);
    let cell = must(pile_cell(&h, PileRef::Waste));
    right_click(&mut h, cell);
    assert_eq!(h.app.screen(), Screen::Board);
    assert_eq!(cursor(&h), before);
    assert!(cues(&h).is_empty());
}

#[test]
fn a_click_on_empty_space_does_nothing() {
    let mut h = start(KLONDIKE, 5);
    let before = (cursor(&h), moves_made(&h), cues(&h), status(&h));
    click(&mut h, HEADER);
    assert_eq!((cursor(&h), moves_made(&h), cues(&h), status(&h)), before);
}

#[test]
fn a_click_on_empty_space_keeps_a_pick() {
    let mut h = start(KLONDIKE, 5);
    let (from, to, count) = first_move(&h);
    let source = must(card_cell(&h, from, pile_len(&h, from) - usize::from(count)));
    click(&mut h, source);
    click(&mut h, HEADER);
    assert!(targets(&h).contains(&to));
    assert_eq!(moves_made(&h), 0);
}

#[test]
fn a_release_outside_the_board_moves_nothing() {
    let mut h = start(KLONDIKE, 5);
    let (from, to, count) = first_move(&h);
    let source = must(card_cell(&h, from, pile_len(&h, from) - usize::from(count)));
    press(&mut h, source);
    drag_to(&mut h, HEADER);
    release(&mut h, HEADER);
    assert_eq!(moves_made(&h), 0);
    assert!(targets(&h).contains(&to));
}

#[test]
fn a_click_on_a_face_down_card_is_refused() {
    let mut h = start(KLONDIKE, 5);
    let column = PileRef::Tableau(6);
    let face_down = session(&h)
        .game()
        .piles()
        .iter()
        .find(|snapshot| snapshot.pile == column)
        .map(|snapshot| {
            snapshot
                .cards
                .iter()
                .enumerate()
                .filter(|(_, card)| !card.face_up)
                .map(|(index, _)| index)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let cell = must(
        face_down
            .into_iter()
            .find_map(|index| card_cell(&h, column, index)),
    );
    click(&mut h, cell);
    assert_eq!(status(&h), Some("board.error.face_down"));
    assert!(cues(&h).contains(&SoundCue::Illegal));
    assert!(targets(&h).is_empty());
    assert_eq!(moves_made(&h), 0);
}

#[test]
fn a_click_on_a_pyramid_pair_removes_it() {
    let mut h = start(RELAXED, PAIR_SEED);
    let first = PileRef::Pyramid { row: 6, index: 4 };
    let second = PileRef::Pyramid { row: 6, index: 5 };
    let cell = must(card_cell(&h, first, 0));
    click(&mut h, cell);
    let cell = must(card_cell(&h, second, 0));
    click(&mut h, cell);
    assert_eq!(pile_len(&h, first), 0);
    assert_eq!(pile_len(&h, second), 0);
    assert_eq!(moves_made(&h), 1);
}

#[test]
fn a_click_moves_the_cursor_to_the_clicked_pile() {
    let mut h = start(KLONDIKE, 5);
    let cell = must(pile_cell(&h, PileRef::Stock));
    click(&mut h, cell);
    let cell = must(pile_cell(&h, PileRef::Tableau(3)));
    click(&mut h, cell);
    assert_eq!(cursor(&h).0, PileRef::Tableau(3));
}
