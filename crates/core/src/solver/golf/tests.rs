use super::*;
use crate::game::{Game, Variant};
use crate::pile::PileSnapshot;
use crate::solver::search::{Outcome, Strategy, run};
use crate::solver::tests::{check_steps, no_finish, pile, same_moves, walk};

fn make(game: &dyn Game) -> Option<Position> {
    Position::from_piles(&game.piles())
}

fn tab(index: u8) -> PileRef {
    PileRef::Tableau(index)
}

/// A table with the given columns (bottom to top), stock (top last) and waste top.
fn table(columns: &[&str], stock: &str, waste: &str) -> Vec<PileSnapshot> {
    let mut piles = vec![
        pile(PileRef::Stock, stock, ""),
        pile(PileRef::Waste, "", waste),
    ];
    for (index, cards) in columns.iter().enumerate() {
        piles.push(pile(tab(u8::try_from(index).unwrap()), "", cards));
    }
    piles
}

fn search(piles: &[PileSnapshot]) -> Outcome {
    let position = Position::from_piles(piles).unwrap();
    run(Strategy::DepthFirst, &position, 100_000, &no_finish).outcome
}

#[test]
fn direct_steps_are_the_engine_moves() {
    walk(Variant::Golf, |game, seed, history| {
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
    check_steps(Variant::Golf, make);
}

#[test]
fn the_model_reads_every_dealt_game() {
    for seed in 0..50 {
        let game = crate::game::deal(Variant::Golf, seed);
        assert!(make(game.as_ref()).is_some(), "seed {seed}");
    }
}

#[test]
fn a_column_that_goes_down_with_the_waste_is_won() {
    let piles = table(&["3C 4H 5S"], "", "6D");
    assert!(matches!(search(&piles), Outcome::Won(moves) if moves.len() == 3));
}

#[test]
fn a_king_does_not_take_an_ace() {
    let piles = table(&["KS"], "", "AD");
    assert!(matches!(search(&piles), Outcome::Exhausted));
}

#[test]
fn the_stock_brings_the_card_that_starts_the_play() {
    let piles = table(&["7S"], "", "KD");
    assert!(matches!(search(&piles), Outcome::Exhausted));
    let piles = table(&["7S"], "6C", "KD");
    assert!(matches!(search(&piles), Outcome::Won(moves) if moves.len() == 2));
}

#[test]
fn a_card_with_no_neighbour_left_is_unwinnable() {
    let piles = table(&["9S", "3H"], "", "4D");
    assert!(matches!(search(&piles), Outcome::Exhausted));
}

#[test]
fn plays_come_before_the_stock_turn() {
    let piles = table(&["5S", "9H"], "2C", "6D");
    let position = Position::from_piles(&piles).unwrap();
    let mut steps = Vec::new();
    position.steps(&mut steps);
    assert_eq!(steps, vec![Step::Play(0), Step::Draw]);
}

#[test]
fn the_play_that_opens_the_most_next_plays_comes_first() {
    let piles = table(&["5D", "3S 7H", "8C"], "", "6S");
    let position = Position::from_piles(&piles).unwrap();
    let mut steps = Vec::new();
    position.steps(&mut steps);
    assert_eq!(steps, vec![Step::Play(1), Step::Play(0)]);
}

#[test]
fn there_is_no_automatic_move() {
    let piles = table(&["5S"], "", "6D");
    let mut position = Position::from_piles(&piles).unwrap();
    let mut moves = Vec::new();
    position.settle(&mut moves);
    assert!(moves.is_empty());
}
