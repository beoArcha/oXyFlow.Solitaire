use super::*;
use crate::game::{Game, PyramidRules, Variant};
use crate::pile::PileSnapshot;
use crate::solver::search::{Outcome, Strategy, run};
use crate::solver::tests::{check_steps, no_finish, pile, same_moves, walk};

const RULES: [PyramidRules; 2] = [PyramidRules::Relaxed, PyramidRules::Classic];

fn make(rules: PyramidRules) -> impl Fn(&dyn Game) -> Option<Position> {
    move |game| {
        let recycles = game.stock_status().and_then(|status| status.recycles_left);
        Position::from_piles(rules, &game.piles(), recycles.unwrap_or(0))
    }
}

#[test]
fn direct_steps_are_the_engine_moves() {
    for rules in RULES {
        walk(Variant::Pyramid(rules), |game, seed, history| {
            let position = make(rules)(game).unwrap();
            let mut steps = Vec::new();
            position.direct(&mut steps);
            let model = steps
                .iter()
                .filter_map(|&step| position.engine_move(step))
                .collect();
            same_moves(
                model,
                game.legal_moves(),
                &format!("{rules:?} seed {seed} after {}", history.len()),
            );
        });
    }
}

#[test]
fn steps_play_legally_and_reach_the_engine_position() {
    for rules in RULES {
        check_steps(Variant::Pyramid(rules), make(rules));
    }
}

#[test]
fn cover_table_matches_the_pyramid_shape() {
    assert_eq!(position::COVERS[0], 0b110);
    assert!(position::COVERS[21..].iter().all(|&mask| mask == 0));
    assert_eq!(position::PLACES[27], (6, 6));
    assert_eq!(position::PLACES[3], (2, 0));
}

fn small_pyramid(extra: Vec<PileSnapshot>) -> Vec<PileSnapshot> {
    let slot = |row, index| PileRef::Pyramid { row, index };
    let mut piles = vec![
        pile(slot(0, 0), "", "6S"),
        pile(slot(1, 0), "", "7C"),
        pile(slot(1, 1), "", "KD"),
        pile(PileRef::Waste, "", "6H"),
    ];
    piles.extend(extra);
    piles
}

fn search(piles: &[PileSnapshot]) -> Outcome {
    let position = Position::from_piles(PyramidRules::Relaxed, piles, 2).unwrap();
    run(Strategy::DepthFirst, &position, 10_000, &no_finish).outcome
}

#[test]
fn a_card_without_a_second_partner_is_unwinnable() {
    assert!(matches!(search(&small_pyramid(vec![])), Outcome::Exhausted));
}

#[test]
fn the_missing_partner_in_the_stock_makes_it_winnable() {
    let piles = small_pyramid(vec![pile(PileRef::Stock, "", "7D")]);
    assert!(matches!(search(&piles), Outcome::Won(_)));
}

#[test]
fn a_pyramid_card_nothing_can_pair_with_is_hopeless() {
    let slot = PileRef::Pyramid { row: 0, index: 0 };
    let piles = vec![pile(slot, "", "5S")];
    let position = Position::from_piles(PyramidRules::Relaxed, &piles, 2).unwrap();
    let mut steps = Vec::new();
    position.steps(&mut steps);
    assert!(steps.is_empty());
}

#[test]
fn kings_are_removed_at_once() {
    let piles = small_pyramid(vec![]);
    let mut position = Position::from_piles(PyramidRules::Relaxed, &piles, 2).unwrap();
    let mut moves = Vec::new();
    position.settle(&mut moves);
    assert_eq!(moves.len(), 1);
    assert_eq!(position.mask.count_ones(), 2);
}
