use super::*;
use crate::game::{Game, Variant};
use crate::pile::PileSnapshot;
use crate::solver::search::{Outcome, Strategy, run};
use crate::solver::tests::{check_steps, no_finish, pile, same_moves, walk};

fn make(game: &dyn Game) -> Option<Position> {
    Position::from_piles(&game.piles())
}

fn peak_ref(row: u8, index: u8) -> PileRef {
    PileRef::Pyramid { row, index }
}

fn table(slots: &[((u8, u8), &str)], stock: &str, waste: &str) -> Vec<PileSnapshot> {
    let mut piles = vec![
        pile(PileRef::Stock, stock, ""),
        pile(PileRef::Waste, "", waste),
    ];
    for &(row, index) in &PLACES {
        let code = slots
            .iter()
            .find(|&&((r, i), _)| r == row && i == index)
            .map_or("", |&(_, code)| code);
        piles.push(pile(peak_ref(row, index), "", code));
    }
    piles
}

fn search(piles: &[PileSnapshot]) -> Outcome {
    let position = Position::from_piles(piles).unwrap();
    run(Strategy::DepthFirst, &position, 100_000, &no_finish).outcome
}

#[test]
fn direct_steps_are_the_engine_moves() {
    walk(Variant::TriPeaks, |game, seed, history| {
        let position = make(game).unwrap();
        let mut steps = Vec::new();
        position.direct(&mut steps);
        let model = steps
            .iter()
            .filter_map(|&step| position.engine_move(step))
            .collect();
        same_moves(
            model,
            game.legal_moves(),
            &format!("seed {seed} after {}", history.len()),
        );
    });
}

#[test]
fn steps_play_legally_and_reach_the_engine_position() {
    check_steps(Variant::TriPeaks, make);
}

#[test]
fn the_model_reads_every_dealt_game() {
    for seed in 0..50 {
        let game = crate::game::deal(Variant::TriPeaks, seed);
        assert!(make(game.as_ref()).is_some(), "seed {seed}");
    }
}

#[test]
fn a_king_and_an_ace_wrap_around() {
    let piles = table(&[((3, 0), "KS"), ((3, 1), "AH")], "", "AD");
    assert!(matches!(search(&piles), Outcome::Won(moves) if moves.len() == 2));
}

#[test]
fn a_covered_card_cannot_be_played_until_both_children_leave() {
    let piles = table(&[((2, 0), "7H"), ((3, 0), "8C"), ((3, 1), "9D")], "", "6S");
    assert!(matches!(search(&piles), Outcome::Exhausted));

    let piles = table(&[((2, 0), "6H"), ((3, 0), "7C"), ((3, 1), "5D")], "", "6S");
    assert!(matches!(search(&piles), Outcome::Exhausted));
}

#[test]
fn a_dead_rank_group_position_is_exhausted_immediately() {
    let piles = table(&[((3, 0), "9S"), ((3, 1), "3H")], "", "4D");
    let pos = Position::from_piles(&piles).unwrap();
    assert!(pos.is_dead());
    assert!(matches!(search(&piles), Outcome::Exhausted));
}

#[test]
fn plays_come_before_the_stock_turn() {
    let piles = table(&[((3, 0), "5S"), ((3, 1), "9H")], "2C", "6D");
    let position = Position::from_piles(&piles).unwrap();
    let mut steps = Vec::new();
    position.steps(&mut steps);
    assert_eq!(steps, vec![Step::Play(18), Step::Draw]);
}
