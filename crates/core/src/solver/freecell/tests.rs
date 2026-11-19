use super::*;
use crate::game::{Game, Variant};
use crate::pile::PileSnapshot;
use crate::solver::search::{Outcome, Strategy, run};
use crate::solver::tests::{check_steps, no_finish, pile, run_of, same_moves, walk};

fn make(game: &dyn Game) -> Option<Position> {
    Position::from_piles(&game.piles())
}

fn normalized(position: &Position, mv: Move) -> Move {
    let Move::Transfer { from, to, count } = mv else {
        return mv;
    };
    let to = match to {
        PileRef::Foundation(_) => PileRef::Foundation(0),
        PileRef::Cell(_) => cell(position.first_free_cell().unwrap_or(0)),
        PileRef::Tableau(i) if position.len[usize::from(i)] == 0 => {
            tableau(position.first_empty_column().unwrap_or(0))
        }
        other => other,
    };
    Move::Transfer { from, to, count }
}

#[test]
fn direct_steps_are_the_engine_moves() {
    walk(Variant::FreeCell, |game, seed, history| {
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
            &format!("seed {seed} after {}", history.len()),
        );
    });
}

#[test]
fn steps_play_legally_and_reach_the_engine_position() {
    check_steps(Variant::FreeCell, make);
}

fn spades_behind_cells(cells: usize) -> Vec<PileSnapshot> {
    let mut piles = vec![
        pile(PileRef::Foundation(0), "", &run_of('C', 1)),
        pile(PileRef::Foundation(1), "", &run_of('D', 1)),
        pile(PileRef::Foundation(2), "", &run_of('H', 1)),
        pile(PileRef::Tableau(0), "", "AS 2S"),
    ];
    for (i, card) in ["7S", "8S", "9S", "10S", "JS", "QS", "KS"]
        .iter()
        .enumerate()
    {
        piles.push(pile(
            PileRef::Tableau(u8::try_from(i + 1).unwrap()),
            "",
            card,
        ));
    }
    for (i, card) in ["3S", "4S", "5S", "6S"].iter().take(cells).enumerate() {
        piles.push(pile(PileRef::Cell(u8::try_from(i).unwrap()), "", card));
    }
    piles
}

#[test]
fn four_full_cells_and_no_empty_column_are_a_dead_end() {
    let position = Position::from_piles(&spades_behind_cells(4)).unwrap();
    let searched = run(Strategy::BestFirst, &position, 1_000, &no_finish);
    assert!(matches!(searched.outcome, Outcome::Exhausted));
}

#[test]
fn a_free_cell_makes_the_same_layout_winnable() {
    let mut piles = spades_behind_cells(3);
    piles.retain(|snapshot| snapshot.pile != PileRef::Tableau(1));
    piles.push(pile(PileRef::Tableau(1), "", "7S 6S"));
    let position = Position::from_piles(&piles).unwrap();
    let searched = run(Strategy::BestFirst, &position, 10_000, &no_finish);
    assert!(matches!(searched.outcome, Outcome::Won(_)));
}

#[test]
fn capacity_counts_free_cells_and_empty_columns() {
    let piles = vec![pile(PileRef::Tableau(0), "", "KS")];
    let position = Position::from_piles(&piles).unwrap();
    assert_eq!(position.capacity(false), 5 << 7);
    assert_eq!(position.capacity(true), 5 << 6);
}

#[test]
fn cell_and_column_order_do_not_change_the_key() {
    let a = vec![
        pile(PileRef::Tableau(0), "", "KS QH"),
        pile(PileRef::Tableau(1), "", "5D"),
        pile(PileRef::Cell(0), "", "2C"),
        pile(PileRef::Cell(2), "", "9H"),
    ];
    let b = vec![
        pile(PileRef::Tableau(5), "", "KS QH"),
        pile(PileRef::Tableau(2), "", "5D"),
        pile(PileRef::Cell(3), "", "2C"),
        pile(PileRef::Cell(1), "", "9H"),
    ];
    let key = |piles: &[PileSnapshot]| Position::from_piles(piles).unwrap().key();
    assert_eq!(key(&a), key(&b));
}

#[test]
fn falling_columns_are_a_forced_finish() {
    let piles = vec![
        pile(PileRef::Tableau(0), "", "KS 9H 4C"),
        pile(PileRef::Tableau(1), "", "QD 7S 2H"),
    ];
    assert!(Position::from_piles(&piles).unwrap().columns_descend());
    let piles = vec![pile(PileRef::Tableau(0), "", "4C 9H")];
    assert!(!Position::from_piles(&piles).unwrap().columns_descend());
}
