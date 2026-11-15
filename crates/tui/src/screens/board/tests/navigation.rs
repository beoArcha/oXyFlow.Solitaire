use oxysol_core::{DrawCount, PileRef, Variant};
use ratatui::crossterm::event::KeyCode;

use super::*;
use crate::screens::board::cursor::{Dir, nearest};

const KLONDIKE: Variant = Variant::Klondike(DrawCount::One);

#[test]
fn the_cursor_starts_on_the_first_column() {
    let h = start(KLONDIKE, 5);
    assert_eq!(cursor(&h), (PileRef::Tableau(0), 1));
}

#[test]
fn left_and_right_walk_the_columns_and_stop_at_the_edges() {
    let mut h = start(KLONDIKE, 5);
    h.press(KeyCode::Left);
    assert_eq!(cursor(&h).0, PileRef::Tableau(0));
    for column in 1..7 {
        h.press(KeyCode::Right);
        assert_eq!(cursor(&h).0, PileRef::Tableau(column));
    }
    h.press(KeyCode::Right);
    assert_eq!(cursor(&h).0, PileRef::Tableau(6));
    h.type_char('h');
    h.press(KeyCode::Left);
    assert_eq!(cursor(&h).0, PileRef::Tableau(5));
}

#[test]
fn up_and_down_cross_between_the_top_row_and_the_columns() {
    let mut h = start(KLONDIKE, 5);
    h.press(KeyCode::Up);
    assert_eq!(cursor(&h).0, PileRef::Stock);
    h.press(KeyCode::Right);
    assert_eq!(cursor(&h).0, PileRef::Waste);
    h.press(KeyCode::Down);
    assert_eq!(cursor(&h).0, PileRef::Tableau(1));
    put_cursor(&mut h, PileRef::Tableau(6));
    h.press(KeyCode::Up);
    assert!(matches!(cursor(&h).0, PileRef::Foundation(_)));
    h.press(KeyCode::Down);
    assert_eq!(cursor(&h).0, PileRef::Tableau(6));
    h.press(KeyCode::Down);
    assert_eq!(cursor(&h).0, PileRef::Tableau(6));
}

#[test]
fn j_and_k_move_like_the_arrows() {
    let mut h = start(KLONDIKE, 5);
    h.type_char('k');
    assert_eq!(cursor(&h).0, PileRef::Stock);
    h.type_char('j');
    assert_eq!(cursor(&h).0, PileRef::Tableau(0));
}

#[test]
fn up_reaches_deeper_into_a_column_and_down_comes_back() {
    let mut h = start(Variant::FreeCell, 1);
    put_cursor(&mut h, PileRef::Tableau(2));
    h.press(KeyCode::Up);
    assert_eq!(cursor(&h), (PileRef::Tableau(2), 2));
    h.type_char('+');
    assert_eq!(cursor(&h), (PileRef::Tableau(2), 3));
    h.type_char('-');
    assert_eq!(cursor(&h), (PileRef::Tableau(2), 2));
    h.press(KeyCode::Down);
    h.press(KeyCode::Down);
    assert_eq!(cursor(&h), (PileRef::Tableau(2), 1));
}

#[test]
fn the_depth_stops_at_the_face_up_cards() {
    let mut h = start(KLONDIKE, 5);
    h.type_char('+');
    assert_eq!(cursor(&h), (PileRef::Tableau(0), 1));
    h.type_char('-');
    assert_eq!(cursor(&h), (PileRef::Tableau(0), 1));
}

#[test]
fn the_focus_is_marked_on_the_card_at_the_cursor_depth() {
    let mut h = start(Variant::FreeCell, 1);
    put_cursor(&mut h, PileRef::Tableau(2));
    let len = pile_len(&h, PileRef::Tableau(2));
    assert_eq!(board(&h).marks().focus.map(|f| f.index), Some(len - 1));
    h.press(KeyCode::Up);
    assert_eq!(board(&h).marks().focus.map(|f| f.index), Some(len - 2));
}

#[test]
fn moving_to_another_pile_resets_the_depth() {
    let mut h = start(Variant::FreeCell, 1);
    put_cursor(&mut h, PileRef::Tableau(2));
    h.type_char('+');
    h.press(KeyCode::Right);
    assert_eq!(cursor(&h), (PileRef::Tableau(3), 1));
}

#[test]
fn freecell_moves_between_the_top_row_and_the_columns() {
    let mut h = start(Variant::FreeCell, 1);
    put_cursor(&mut h, PileRef::Tableau(0));
    for _ in 0..30 {
        h.press(KeyCode::Up);
        if !matches!(cursor(&h).0, PileRef::Tableau(_)) {
            break;
        }
    }
    assert!(matches!(
        cursor(&h).0,
        PileRef::Cell(_) | PileRef::Foundation(_)
    ));
    h.press(KeyCode::Down);
    assert_eq!(cursor(&h).0, PileRef::Tableau(0));
}

#[test]
fn every_pile_is_reachable_by_arrows_in_every_variant() {
    for variant in Variant::ALL {
        let h = start(variant, 3);
        let piles = session(&h).game().piles();
        let Some(geometry) = board(&h).geometry(&piles) else {
            unreachable!("a game is running")
        };
        let mut reached = vec![board(&h).cursor.pile];
        let mut frontier = reached.clone();
        while let Some(from) = frontier.pop() {
            for dir in [Dir::Up, Dir::Down, Dir::Left, Dir::Right] {
                let next = nearest(&piles, &geometry, from, dir);
                if let Some(next) = next.filter(|next| !reached.contains(next)) {
                    reached.push(next);
                    frontier.push(next);
                }
            }
        }
        let focusable = piles
            .iter()
            .filter(|snapshot| crate::screens::board::cursor::focusable(snapshot))
            .count();
        assert_eq!(reached.len(), focusable, "{variant:?}");
    }
}
