use oxysol_core::{PileRef, Session, Variant};
use ratatui::crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use super::*;
use crate::announce::Look;
use crate::input::Input;
use crate::layout::Hit;
use crate::screens::board::status::look_at;

const KEYS: [KeyCode; 4] = [KeyCode::Up, KeyCode::Down, KeyCode::Left, KeyCode::Right];
const DEPTH_KEYS: [KeyCode; 6] = [
    KeyCode::Up,
    KeyCode::Down,
    KeyCode::Left,
    KeyCode::Right,
    KeyCode::Char('+'),
    KeyCode::Char('-'),
];

fn next_random(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

fn focused_tableau_look(h: &Harness) -> Option<Look> {
    let (pile, depth) = cursor(h);
    if !matches!(pile, PileRef::Tableau(_)) {
        return None;
    }
    session(h)
        .game()
        .piles()
        .iter()
        .find(|snapshot| snapshot.pile == pile)
        .map(|snapshot| look_at(snapshot, depth))
}

fn column_with_a_face_down_card(h: &Harness) -> Option<PileRef> {
    session(h)
        .game()
        .piles()
        .iter()
        .filter(|snapshot| matches!(snapshot.pile, PileRef::Tableau(_)))
        .max_by_key(|snapshot| snapshot.cards.iter().filter(|card| !card.face_up).count())
        .filter(|snapshot| snapshot.cards.iter().any(|card| !card.face_up))
        .map(|snapshot| snapshot.pile)
}

fn assert_focus_is_face_up(h: &Harness, context: &str) {
    let (pile, depth) = cursor(h);
    assert_ne!(
        focused_tableau_look(h),
        Some(Look::FaceDown),
        "{context}: focus on {pile:?} at depth {depth}"
    );
}

fn mouse(kind: MouseEventKind, cell: (u16, u16)) -> Input {
    Input::Mouse(MouseEvent {
        kind,
        column: cell.0,
        row: cell.1,
        modifiers: KeyModifiers::NONE,
    })
}

fn click_card(h: &mut Harness, pile: PileRef, index: usize) {
    let piles = session(h).game().piles();
    let Some(geometry) = board(h).geometry(&piles) else {
        unreachable!("a game is running")
    };
    let Some(rect) = geometry
        .pile(pile)
        .and_then(|placed| placed.cards.get(index).copied())
    else {
        unreachable!("card {index} of {pile:?} is drawn")
    };
    let wanted = Hit {
        pile,
        card: Some(index),
    };
    let cell = (rect.y..rect.bottom())
        .flat_map(|row| (rect.x..rect.right()).map(move |column| (column, row)))
        .find(|&(column, row)| geometry.hit(column, row) == Some(wanted));
    let Some(cell) = cell else {
        unreachable!("a cell hits card {index} of {pile:?}")
    };
    h.send(mouse(MouseEventKind::Down(MouseButton::Left), cell));
    h.send(mouse(MouseEventKind::Up(MouseButton::Left), cell));
}

#[test]
fn arrows_never_focus_a_face_down_card_in_any_position() {
    for variant in Variant::ALL {
        for seed in 1..=3 {
            let mut session = Session::new(variant, seed);
            let mut moves = Vec::new();
            let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
            for _ in 0..40 {
                let mut h = resumed(variant, seed, &moves);
                for _ in 0..120 {
                    let key = KEYS[usize::try_from(next_random(&mut state) % 4).unwrap_or(0)];
                    let before = cursor(&h).0;
                    h.press(key);
                    let (pile, depth) = cursor(&h);
                    assert_focus_is_face_up(&h, &format!("{variant:?} seed {seed}, {key:?}"));
                    if pile != before && matches!(pile, PileRef::Tableau(_)) {
                        assert_eq!(depth, 1, "{variant:?} seed {seed}: entering {pile:?}");
                    }
                }
                let legal = session.game().legal_moves();
                if legal.is_empty() {
                    break;
                }
                let index = usize::try_from(next_random(&mut state)).unwrap_or(0) % legal.len();
                let mv = legal[index];
                assert_eq!(session.apply(mv), Ok(()));
                moves.push(mv);
            }
        }
    }
}

#[test]
fn up_and_down_inside_a_column_skip_the_face_down_cards() {
    for variant in Variant::ALL {
        let h0 = start(variant, 1);
        let Some(pile) = column_with_a_face_down_card(&h0) else {
            continue;
        };
        let mut h = start(variant, 1);
        put_cursor(&mut h, pile);
        for key in [KeyCode::Up, KeyCode::Down] {
            for _ in 0..20 {
                h.press(key);
                assert_focus_is_face_up(&h, &format!("{variant:?} {key:?} in {pile:?}"));
            }
        }
    }
}

#[test]
fn a_click_on_a_face_down_card_does_not_leave_the_arrows_on_it() {
    for variant in Variant::ALL {
        let h0 = start(variant, 1);
        let Some(pile) = column_with_a_face_down_card(&h0) else {
            continue;
        };
        for key in DEPTH_KEYS {
            let mut h = start(variant, 1);
            click_card(&mut h, pile, 0);
            h.press(key);
            assert_focus_is_face_up(&h, &format!("{variant:?} after a click, {key:?}"));
        }
    }
}
