use super::*;
use crate::game::{DrawCount, Variant};
use crate::pile::PileSnapshot;
use crate::solver::search::{Outcome, Strategy, run};
use crate::solver::tests::{check_steps, no_finish, pile, run_of, same_moves, walk};

fn make(draw: DrawCount) -> impl Fn(&dyn crate::game::Game) -> Option<Position> {
    move |game| Position::from_piles(draw, &game.piles())
}

fn normalized(position: &Position, mv: Move) -> Move {
    let Move::Transfer { from, to, count } = mv else {
        return mv;
    };
    let first_empty = position.len.iter().position(|&len| len == 0);
    let to = match to {
        PileRef::Foundation(_) => PileRef::Foundation(0),
        PileRef::Tableau(i) if position.len[usize::from(i)] == 0 => {
            tableau(first_empty.unwrap_or(0))
        }
        other => other,
    };
    Move::Transfer { from, to, count }
}

#[test]
fn direct_steps_are_the_engine_moves() {
    for draw in [DrawCount::One, DrawCount::Three] {
        walk(Variant::Klondike(draw), |game, seed, history| {
            let position = Position::from_piles(draw, &game.piles()).unwrap();
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
                .filter(|&mv| mv != Move::Stock)
                .map(|mv| normalized(&position, mv))
                .collect();
            same_moves(model, engine, &format!("{draw:?} seed {seed} after {}", history.len()));
        });
    }
}

#[test]
fn steps_play_legally_and_reach_the_engine_position() {
    for draw in [DrawCount::One, DrawCount::Three] {
        check_steps(Variant::Klondike(draw), make(draw));
    }
}

fn blocked_spades() -> Vec<PileSnapshot> {
    let mut piles = vec![
        pile(PileRef::Foundation(0), "", &run_of('C', 1)),
        pile(PileRef::Foundation(1), "", &run_of('D', 1)),
        pile(PileRef::Foundation(2), "", &run_of('H', 1)),
        pile(PileRef::Tableau(0), "AS", "2S"),
        pile(PileRef::Stock, "3S 4S 5S 6S 7S", ""),
    ];
    for (column, card) in ["KS", "QS", "JS", "10S", "9S", "8S"].iter().enumerate() {
        piles.push(pile(PileRef::Tableau(u8::try_from(column + 1).unwrap()), "", card));
    }
    piles
}

#[test]
fn a_position_with_no_move_that_helps_is_exhausted() {
    for draw in [DrawCount::One, DrawCount::Three] {
        let position = Position::from_piles(draw, &blocked_spades()).unwrap();
        let searched = run(Strategy::DepthFirst, &position, 1_000, &no_finish);
        assert!(matches!(searched.outcome, Outcome::Exhausted), "{draw:?}");
    }
}

#[test]
fn a_free_column_turns_the_blocked_position_into_a_win() {
    let mut piles = blocked_spades();
    piles.retain(|snapshot| snapshot.pile != PileRef::Tableau(6));
    piles.push(pile(PileRef::Stock, "3S 4S 5S 6S 7S 8S", ""));
    piles.retain(|snapshot| snapshot.pile != PileRef::Stock || snapshot.cards.len() == 6);
    let position = Position::from_piles(DrawCount::One, &piles).unwrap();
    let searched = run(Strategy::DepthFirst, &position, 10_000, &no_finish);
    assert!(matches!(searched.outcome, Outcome::Won(_)));
}

#[test]
fn aces_and_twos_are_safe_and_a_lone_high_card_is_not() {
    let piles = vec![pile(PileRef::Tableau(0), "", "AS"), pile(PileRef::Tableau(1), "", "AD")];
    let mut position = Position::from_piles(DrawCount::One, &piles).unwrap();
    let mut moves = Vec::new();
    position.settle(&mut moves);
    assert_eq!(moves.len(), 2);
    assert_eq!(position.found.iter().sum::<u8>(), 2);

    let both = vec![
        pile(PileRef::Foundation(0), "", "AD 2D"),
        pile(PileRef::Foundation(1), "", "AH 2H"),
        pile(PileRef::Tableau(0), "", "3S"),
    ];
    let position = Position::from_piles(DrawCount::One, &both).unwrap();
    assert!(position.is_safe(position.cols[0][0]));
    let one = vec![
        pile(PileRef::Foundation(0), "", "AD 2D"),
        pile(PileRef::Tableau(0), "", "3S"),
    ];
    let position = Position::from_piles(DrawCount::One, &one).unwrap();
    assert!(!position.is_safe(position.cols[0][0]));
}

#[test]
fn column_order_does_not_change_the_key() {
    let a = vec![pile(PileRef::Tableau(0), "9C", "KS QH"), pile(PileRef::Tableau(1), "", "5D")];
    let b = vec![pile(PileRef::Tableau(3), "9C", "KS QH"), pile(PileRef::Tableau(6), "", "5D")];
    let key = |piles: &[PileSnapshot]| Position::from_piles(DrawCount::One, piles).unwrap().key();
    assert_eq!(key(&a), key(&b));
    let c = vec![pile(PileRef::Tableau(0), "9C", "KS QH"), pile(PileRef::Tableau(1), "", "5C")];
    assert_ne!(key(&a), key(&c));
}

#[test]
fn drawing_matches_the_engine_order() {
    let piles = vec![pile(PileRef::Stock, "AC 2C 3C 4C 5C", "")];
    let mut position = Position::from_piles(DrawCount::Three, &piles).unwrap();
    position.talon.draw(3);
    assert_eq!(position.talon.waste_top(), Some(2));
    position.talon.draw(3);
    assert_eq!(position.talon.waste_top(), Some(0));
    position.talon.draw(3);
    assert!(position.talon.waste_top().is_none());
}
