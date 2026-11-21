//! The `Do:` row of the Yukon family: a movable face-up card offers `pick up` and a quick move, a
//! blocked or face-down card offers neither.

use super::*;
use crate::action::Action;
use crate::i18n::tf;
use crate::input::primary_key;
use crate::screens::Effect;
use crate::storage::memory::MemoryStorage;

const WIDTH: u16 = 120;
const HEIGHT: u16 = 40;
const SEEDS: u64 = 40;

fn yukon_variants() -> impl Iterator<Item = Variant> {
    Variant::ALL
        .into_iter()
        .filter(|variant| matches!(variant, Variant::Yukon(_)))
}

fn yukon(variant: Variant, seed: u64) -> Harness {
    let mut h = Harness::with(MemoryStorage::default(), TRUE_COLOR, WIDTH, HEIGHT);
    h.app.apply(Effect::Start {
        variant,
        seed: Some(seed),
        undo_allowed: true,
    });
    h
}

fn action_text(action: Action, key: &str) -> String {
    let name = primary_key(action).unwrap_or_default();
    tf(key, &[("key", &name)])
}

fn quick_texts() -> [String; 2] {
    [
        action_text(Action::QuickMove, "board.action.quick"),
        action_text(Action::QuickMove, "board.action.quick_foundation"),
    ]
}

fn do_row(h: &mut Harness) -> String {
    h.lines()
        .into_iter()
        .filter(|line| line.contains("Do:"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn focus(h: &mut Harness, pile: PileRef, depth: usize) {
    h.app.board_mut().cursor = Cursor { pile, depth };
}

fn transfer_of_count(h: &Harness, pile: PileRef, count: usize) -> bool {
    legal(h).iter().any(|mv| {
        matches!(mv, Move::Transfer { from, count: n, .. } if *from == pile && usize::from(*n) == count)
    })
}

fn depth_limit(h: &Harness, pile: PileRef) -> (usize, usize) {
    session(h)
        .game()
        .piles()
        .iter()
        .find(|snapshot| snapshot.pile == pile)
        .map_or((0, 0), |snapshot| {
            (max_depth(snapshot), snapshot.cards.len())
        })
}

#[test]
fn a_movable_face_up_card_offers_pick_up_and_quick_move_in_every_yukon_variant() {
    let pick_up = action_text(Action::Select, "board.action.pick_up");
    let quick = quick_texts();
    for variant in yukon_variants() {
        let mut checked = 0;
        for seed in 1..=SEEDS {
            let mut h = yukon(variant, seed);
            for column in 0..7u8 {
                let pile = PileRef::Tableau(column);
                let (face_up, _) = depth_limit(&h, pile);
                for depth in 1..=face_up {
                    if !transfer_of_count(&h, pile, depth) {
                        continue;
                    }
                    focus(&mut h, pile, depth);
                    let row = do_row(&mut h);
                    assert!(
                        row.contains(&pick_up),
                        "{variant:?} seed {seed} {pile:?} {depth}: {row}"
                    );
                    assert!(
                        quick.iter().any(|text| row.contains(text.as_str())),
                        "{variant:?} seed {seed} {pile:?} {depth}: {row}"
                    );
                    checked += 1;
                }
            }
        }
        assert!(checked > 0, "{variant:?}: no movable card was found");
    }
}

#[test]
fn a_blocked_or_face_down_card_offers_neither_pick_up_nor_quick_move_in_every_yukon_variant() {
    let pick_up = action_text(Action::Select, "board.action.pick_up");
    let quick = quick_texts();
    for variant in yukon_variants() {
        let mut checked = 0;
        for seed in 1..=SEEDS {
            let mut h = yukon(variant, seed);
            for column in 0..7u8 {
                let pile = PileRef::Tableau(column);
                let (face_up, len) = depth_limit(&h, pile);
                for depth in 1..=len {
                    if depth <= face_up && transfer_of_count(&h, pile, depth) {
                        continue;
                    }
                    focus(&mut h, pile, depth);
                    let row = do_row(&mut h);
                    assert!(
                        !row.contains(&pick_up),
                        "{variant:?} seed {seed} {pile:?} {depth}: {row}"
                    );
                    assert!(
                        quick.iter().all(|text| !row.contains(text.as_str())),
                        "{variant:?} seed {seed} {pile:?} {depth}: {row}"
                    );
                    checked += 1;
                }
            }
        }
        assert!(checked > 0, "{variant:?}: no blocked card was found");
    }
}
