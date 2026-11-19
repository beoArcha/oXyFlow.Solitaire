use super::*;
use crate::game::{Game, SuitCount, Variant};
use crate::pile::PileSnapshot;
use crate::solver::search::{Outcome, Strategy, run};
use crate::solver::tests::{check_steps, no_finish, pile, run_of, same_moves, walk};

const SUITS: [SuitCount; 3] = [SuitCount::One, SuitCount::Two, SuitCount::Four];

fn make(game: &dyn Game) -> Option<Position> {
    Position::from_piles(&game.piles())
}

fn normalized(position: &Position, mv: Move) -> Move {
    let Move::Transfer { from, to, count } = mv else {
        return mv;
    };
    let to = match to {
        PileRef::Tableau(i) if position.len[usize::from(i)] == 0 => {
            tableau(position.first_empty_column().unwrap_or(0))
        }
        other => other,
    };
    Move::Transfer { from, to, count }
}

#[test]
fn direct_steps_are_the_engine_moves() {
    for suits in SUITS {
        walk(Variant::Spider(suits), |game, seed, history| {
            let position = make(game).unwrap();
            let mut steps = Vec::new();
            position.direct(&mut steps);
            let model = steps
                .iter()
                .filter_map(|&step| position.engine_move(step))
                .map(|mv| normalized(&position, mv))
                .collect();
            let engine = game
                .legal_moves()
                .into_iter()
                .map(|mv| normalized(&position, mv))
                .collect();
            same_moves(
                model,
                engine,
                &format!("{suits:?} seed {seed} after {}", history.len()),
            );
        });
    }
}

#[test]
fn steps_play_legally_and_reach_the_engine_position() {
    for suits in SUITS {
        check_steps(Variant::Spider(suits), make);
    }
}

fn seven_runs_home(extra: Vec<PileSnapshot>) -> Vec<PileSnapshot> {
    let mut piles: Vec<PileSnapshot> = (0..7)
        .map(|i| pile(PileRef::Foundation(i), "", &run_of('S', 1)))
        .collect();
    piles.extend(extra);
    piles
}

#[test]
fn the_last_run_without_its_ace_is_a_dead_end() {
    let piles = seven_runs_home(vec![
        pile(PileRef::Tableau(0), "", "KS"),
        pile(PileRef::Tableau(1), "", "KS QS"),
        pile(PileRef::Tableau(2), "", "JS 10S"),
        pile(PileRef::Tableau(3), "", "9S"),
    ]);
    let position = Position::from_piles(&piles).unwrap();
    let searched = run(Strategy::DepthFirst, &position, 10_000, &no_finish);
    assert!(matches!(searched.outcome, Outcome::Exhausted));
}

#[test]
fn the_last_run_with_its_ace_is_won_and_collected() {
    let piles = seven_runs_home(vec![
        pile(PileRef::Tableau(0), "", "KS QS JS 10S"),
        pile(PileRef::Tableau(1), "", "9S 8S 7S"),
        pile(PileRef::Tableau(2), "", "6S 5S 4S 3S 2S"),
        pile(PileRef::Tableau(3), "", "AS"),
    ]);
    let position = Position::from_piles(&piles).unwrap();
    assert_eq!(position.removed, 7);
    let searched = run(Strategy::DepthFirst, &position, 10_000, &no_finish);
    assert!(matches!(searched.outcome, Outcome::Won(_)));
}

#[test]
fn a_complete_run_is_collected_and_uncovers_a_face_down_card() {
    let long = "KS QS JS 10S 9S 8S 7S 6S 5S 4S 3S 2S";
    let piles = vec![
        pile(PileRef::Tableau(0), "4H", long),
        pile(PileRef::Tableau(1), "", "AS"),
    ];
    let mut position = Position::from_piles(&piles).unwrap();
    position.move_run(1, 0, 1);
    assert_eq!(position.removed, 1);
    assert_eq!((position.len[0], position.hidden[0]), (1, 0));
    assert_eq!(position.len[1], 0);
}

#[test]
fn only_the_same_suit_tail_moves_together() {
    let piles = vec![pile(PileRef::Tableau(0), "", "9H 8S 7S 6H 5H")];
    let position = Position::from_piles(&piles).unwrap();
    assert_eq!(position.run_len(0), 2);
}
