//! Tab and Shift+Tab on the board: the cursor visits the piles in reading order and wraps.

use oxysol_core::{DrawCount, PileRef, Variant};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::*;
use crate::input::Input;
use crate::screens::board::cursor::focusable;

const KLONDIKE: Variant = Variant::Klondike(DrawCount::One);

fn shift_tab(h: &mut Harness) {
    h.send(Input::Key(KeyEvent::new(
        KeyCode::BackTab,
        KeyModifiers::SHIFT,
    )));
}

fn tableau(index: u8) -> PileRef {
    PileRef::Tableau(index)
}

fn klondike_reading_order() -> Vec<PileRef> {
    let mut order = vec![PileRef::Stock, PileRef::Waste];
    order.extend((0..4).map(PileRef::Foundation));
    order.extend((0..7).map(tableau));
    order
}

#[test]
fn tab_visits_every_pile_once_and_wraps_to_the_start() {
    let mut h = start(KLONDIKE, 5);
    let mut visited = Vec::new();
    for _ in 0..klondike_reading_order().len() {
        h.press(KeyCode::Tab);
        visited.push(cursor(&h).0);
    }
    let order = klondike_reading_order();
    let start_at = order.iter().position(|pile| *pile == tableau(0));
    let expected: Vec<PileRef> = (1..=order.len())
        .map(|step| order[(start_at.unwrap_or(0) + step) % order.len()])
        .collect();
    assert_eq!(visited, expected);
    assert_eq!(cursor(&h).0, tableau(0));
}

#[test]
fn tab_visits_every_focusable_pile_of_every_game_once_and_wraps() {
    for variant in Variant::ALL {
        let mut h = start(variant, 9);
        let first = cursor(&h).0;
        let count = board(&h)
            .piles()
            .iter()
            .filter(|snapshot| focusable(snapshot))
            .count();
        let mut visited = Vec::new();
        for _ in 0..count {
            h.press(KeyCode::Tab);
            visited.push(cursor(&h).0);
        }
        for (index, pile) in visited.iter().enumerate() {
            assert!(
                !visited[..index].contains(pile),
                "{variant:?}: {pile:?} twice"
            );
        }
        assert_eq!(visited.last(), Some(&first), "{variant:?}");
    }
}

#[test]
fn shift_tab_visits_the_piles_backwards() {
    let mut h = start(KLONDIKE, 5);
    assert_eq!(cursor(&h), (tableau(0), 1));
    shift_tab(&mut h);
    assert_eq!(cursor(&h).0, PileRef::Foundation(3));
    shift_tab(&mut h);
    assert_eq!(cursor(&h).0, PileRef::Foundation(2));
}

#[test]
fn tab_resets_the_depth_of_the_cursor() {
    let mut h = start(KLONDIKE, 5);
    h.type_char('h');
    h.press(KeyCode::Tab);
    assert_eq!(cursor(&h).1, 1);
}

#[test]
fn space_on_a_pile_reached_by_tab_acts_on_that_pile() {
    let mut h = start(KLONDIKE, 5);
    let before = moves_made(&h);
    for _ in 0..6 {
        shift_tab(&mut h);
    }
    assert_eq!(cursor(&h).0, PileRef::Stock);
    h.type_char(' ');
    assert!(moves_made(&h) > before);
}

#[test]
fn enter_on_a_column_reached_by_tab_acts_as_on_the_same_column_reached_by_right() {
    let mut by_tab = start(KLONDIKE, 5);
    by_tab.press(KeyCode::Tab);
    by_tab.press(KeyCode::Enter);
    let mut by_arrow = start(KLONDIKE, 5);
    by_arrow.press(KeyCode::Right);
    by_arrow.press(KeyCode::Enter);
    assert_eq!(cursor(&by_tab), cursor(&by_arrow));
    assert_eq!(board(&by_tab).pick, board(&by_arrow).pick);
    assert_eq!(board(&by_tab).status_key(), board(&by_arrow).status_key());
}
