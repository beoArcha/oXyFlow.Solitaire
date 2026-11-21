use oxysol_core::{Move, PileRef, PyramidRules, Variant};
use ratatui::crossterm::event::KeyCode;

use super::*;
use crate::screens::board::cursor::{Dir, focusable, nearest};

const RELAXED: Variant = Variant::Pyramid(PyramidRules::Relaxed);
const PAIR_SEED: u64 = 13;
const KING_SEED: u64 = 1;
const STOCK_PAIR_SEED: u64 = 2;

fn position(row: u8, index: u8) -> PileRef {
    PileRef::Pyramid { row, index }
}

#[test]
fn the_cursor_starts_at_the_bottom_left_of_the_pyramid() {
    let h = start(RELAXED, PAIR_SEED);
    assert_eq!(cursor(&h), (position(6, 0), 1));
}

#[test]
fn a_pair_is_picked_and_removed() {
    let mut h = start(RELAXED, PAIR_SEED);
    let (first, second) = (position(6, 4), position(6, 5));
    assert!(legal(&h).contains(&Move::Remove {
        first,
        second: Some(second)
    }));
    put_cursor(&mut h, first);
    h.press(KeyCode::Enter);
    assert!(board(&h).marks().targets.contains(&second));
    put_cursor(&mut h, second);
    h.press(KeyCode::Enter);
    assert_eq!(pile_len(&h, first), 0);
    assert_eq!(pile_len(&h, second), 0);
    assert_eq!(cues(&h), vec![SoundCue::CardPick, SoundCue::CardPlace]);
    assert_eq!(moves_made(&h), 1);
}

#[test]
fn a_pair_can_be_picked_from_its_second_card() {
    let mut h = start(RELAXED, PAIR_SEED);
    drag(&mut h, position(6, 5), position(6, 4));
    assert_eq!(pile_len(&h, position(6, 4)), 0);
    assert_eq!(pile_len(&h, position(6, 5)), 0);
}

#[test]
fn a_pair_with_the_stock_card_is_dropped_on_the_stock() {
    let mut h = start(RELAXED, STOCK_PAIR_SEED);
    let Some(Move::Remove {
        first: PileRef::Stock,
        second: Some(partner),
    }) = legal(&h).into_iter().find(|mv| {
        matches!(
            mv,
            Move::Remove {
                first: PileRef::Stock,
                second: Some(_)
            }
        )
    })
    else {
        unreachable!("seed 2 pairs the stock card")
    };
    let stock = pile_len(&h, PileRef::Stock);
    drag(&mut h, partner, PileRef::Stock);
    assert_eq!(pile_len(&h, PileRef::Stock), stock - 1);
    assert_eq!(pile_len(&h, partner), 0);
}

#[test]
fn a_quick_move_on_the_stock_card_removes_a_pair() {
    let mut h = start(RELAXED, STOCK_PAIR_SEED);
    put_cursor(&mut h, PileRef::Stock);
    h.type_char('a');
    assert_eq!(moves_made(&h), 1);
    assert_eq!(cues(&h), vec![SoundCue::CardPlace]);
}

#[test]
fn a_king_is_removed_at_once() {
    let mut h = start(RELAXED, KING_SEED);
    let Some(Move::Remove {
        first: king,
        second: None,
    }) = legal(&h)
        .into_iter()
        .find(|mv| matches!(mv, Move::Remove { second: None, .. }))
    else {
        unreachable!("seed 1 has a king")
    };
    put_cursor(&mut h, king);
    h.press(KeyCode::Enter);
    assert_eq!(pile_len(&h, king), 0);
    assert_eq!(moves_made(&h), 1);
    assert_eq!(cues(&h), vec![SoundCue::CardPlace]);
    assert!(board(&h).marks().selected.is_none());
}

#[test]
fn a_covered_card_or_a_card_without_a_partner_is_refused_by_the_engine() {
    let mut h = start(RELAXED, PAIR_SEED);
    put_cursor(&mut h, position(0, 0));
    h.press(KeyCode::Enter);
    assert_eq!(status(&h), Some("board.error.blocked"));
    assert_eq!(moves_made(&h), 0);
    assert_eq!(cues(&h), vec![SoundCue::Illegal]);
}

#[test]
fn a_wrong_partner_is_refused_with_the_pair_message() {
    let mut h = start(RELAXED, PAIR_SEED);
    let (first, second) = (position(6, 4), position(6, 5));
    let wrong = (0..7).map(|index| position(6, index)).find(|pile| {
        *pile != first
            && *pile != second
            && !legal(&h).contains(&Move::Remove {
                first,
                second: Some(*pile),
            })
    });
    put_cursor(&mut h, first);
    h.press(KeyCode::Enter);
    put_cursor(&mut h, wrong.unwrap_or(position(6, 0)));
    h.press(KeyCode::Enter);
    assert_eq!(status(&h), Some("board.error.not_a_pair"));
    assert!(board(&h).marks().selected.is_some());
}

#[test]
fn removed_positions_are_skipped_and_the_cursor_leaves_them() {
    let mut h = start(RELAXED, PAIR_SEED);
    drag(&mut h, position(6, 4), position(6, 5));
    assert!(focusable_cursor(&h));
    assert_ne!(cursor(&h).0, position(6, 5));
    let piles = session(&h).game().piles();
    let Some(geometry) = board(&h).geometry(&piles) else {
        unreachable!("a game is running")
    };
    for snapshot in piles.iter().filter(|s| focusable(s)) {
        for dir in [Dir::Up, Dir::Down, Dir::Left, Dir::Right] {
            let next = nearest(&piles, &geometry, snapshot.pile, dir);
            assert_ne!(next, Some(position(6, 4)));
            assert_ne!(next, Some(position(6, 5)));
        }
    }
}

fn focusable_cursor(h: &Harness) -> bool {
    let piles = session(h).game().piles();
    piles
        .iter()
        .find(|s| s.pile == board(h).cursor.pile)
        .is_some_and(focusable)
}

#[test]
fn arrows_walk_a_row_of_the_pyramid_and_between_rows() {
    let mut h = start(RELAXED, PAIR_SEED);
    for index in 1..7 {
        h.press(KeyCode::Right);
        assert_eq!(cursor(&h).0, position(6, index));
    }
    h.press(KeyCode::Up);
    assert!(matches!(cursor(&h).0, PileRef::Pyramid { row: 5, .. }));
    h.press(KeyCode::Down);
    assert!(matches!(cursor(&h).0, PileRef::Pyramid { row: 6, .. }));
}

#[test]
fn a_fresh_deal_marks_only_the_covered_cards_as_blocked() {
    let h = start(RELAXED, PAIR_SEED);
    let blocked = board(&h).marks().blocked;
    assert_eq!(blocked.len(), 21);
    assert!((0..7).all(|index| !blocked.contains(&position(6, index))));
    assert!(
        blocked
            .iter()
            .all(|pile| matches!(pile, PileRef::Pyramid { row, .. } if *row < 6))
    );
}
