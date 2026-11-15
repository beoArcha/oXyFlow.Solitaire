use super::*;
use crate::moves::MoveError::{
    CountOutOfRange, EmptySource, NoSuchPile, TargetRejects, Unsupported,
};

#[test]
fn an_ace_goes_to_any_empty_foundation() {
    let mut game = table();
    game.columns[0] = cards("AS");
    for index in 0..4 {
        let mv = transfer(tab(0), found(index), 1);
        assert!(game.legal_moves().contains(&mv));
    }
    assert_accepted(&mut game, transfer(tab(0), found(2), 1));
    assert_eq!(cards_of(&game, found(2)), cards("AS"));
}

#[test]
fn foundations_build_up_in_suit_from_cells_and_columns() {
    let mut game = table();
    game.foundations[0] = cards("AS");
    game.foundations[1] = cards("AH");
    game.cells[0] = cards("2S");
    game.columns[0] = cards("2H");
    game.columns[1] = cards("3S");
    game.columns[2] = cards("2C");
    assert_rejected(&game, transfer(cell(0), found(1), 1), TargetRejects);
    assert_rejected(&game, transfer(tab(0), found(0), 1), TargetRejects);
    assert_rejected(&game, transfer(tab(2), found(0), 1), TargetRejects);
    assert_accepted(&mut game, transfer(cell(0), found(0), 1));
    assert_accepted(&mut game, transfer(tab(1), found(0), 1));
    assert_eq!(cards_of(&game, found(0)), cards("AS 2S 3S"));
}

#[test]
fn a_foundation_never_gives_cards_back() {
    let mut game = table();
    game.foundations[0] = cards("AS 2S");
    game.columns[0] = cards("3H");
    game.cells[0] = cards("AD");
    let from_foundation = transfer(found(0), tab(0), 1);
    assert_rejected(&game, from_foundation, Unsupported);
    assert_rejected(&game, transfer(found(0), cell(0), 1), Unsupported);
    assert_rejected(&game, transfer(found(0), found(1), 1), Unsupported);
    let remove = Move::Remove {
        first: tab(0),
        second: None,
    };
    assert_rejected(&game, remove, Unsupported);
    assert_rejected(&game, Move::Stock, Unsupported);
}

#[test]
fn a_cell_takes_one_card_into_an_empty_cell_only() {
    let mut game = table();
    game.columns[0] = cards("7H 6S");
    game.cells[0] = cards("QD");
    game.cells[1] = cards("");
    assert_rejected(&game, transfer(tab(0), cell(0), 1), TargetRejects);
    assert_rejected(
        &game,
        transfer(tab(0), cell(2), 2),
        CountOutOfRange { count: 2 },
    );
    assert_accepted(&mut game, transfer(tab(0), cell(1), 1));
    assert_eq!(cards_of(&game, cell(1)), cards("6S"));
    assert_eq!(cards_of(&game, tab(0)), cards("7H"));
}

#[test]
fn a_cell_card_goes_back_to_a_column_but_not_to_another_cell() {
    let mut game = table();
    game.cells[0] = cards("6D");
    game.cells[1] = cards("KS");
    game.columns[0] = cards("7S");
    game.columns[1] = cards("9H");
    assert_rejected(&game, transfer(cell(0), cell(2), 1), TargetRejects);
    assert_rejected(&game, transfer(cell(0), cell(1), 1), TargetRejects);
    assert_rejected(&game, transfer(cell(0), tab(1), 1), TargetRejects);
    assert_accepted(&mut game, transfer(cell(0), tab(0), 1));
    assert_eq!(cards_of(&game, tab(0)), cards("7S 6D"));
    assert_eq!(cards_of(&game, cell(0)), cards(""));
}

#[test]
fn tableau_builds_down_in_alternating_colors() {
    let mut game = table();
    game.columns[0] = cards("7H");
    game.columns[1] = cards("8S");
    game.columns[2] = cards("8H");
    game.columns[3] = cards("9S");
    assert_rejected(&game, transfer(tab(0), tab(2), 1), TargetRejects);
    assert_rejected(&game, transfer(tab(0), tab(3), 1), TargetRejects);
    assert_rejected(&game, transfer(tab(1), tab(0), 1), TargetRejects);
    assert_accepted(&mut game, transfer(tab(0), tab(1), 1));
    assert_eq!(cards_of(&game, tab(1)), cards("8S 7H"));
    assert_eq!(cards_of(&game, tab(0)), cards(""));
}

#[test]
fn a_run_moves_as_one_and_a_part_of_it_too() {
    let mut part = table();
    part.columns[0] = cards("9C 7H 6S 5D");
    part.columns[1] = cards("6C");
    assert_accepted(&mut part, transfer(tab(0), tab(1), 1));
    assert_eq!(cards_of(&part, tab(1)), cards("6C 5D"));
    let mut whole = table();
    whole.columns[0] = cards("9C 7H 6S 5D");
    whole.columns[1] = cards("8S");
    assert_accepted(&mut whole, transfer(tab(0), tab(1), 3));
    assert_eq!(cards_of(&whole, tab(1)), cards("8S 7H 6S 5D"));
}

#[test]
fn counts_outside_the_run_are_out_of_range() {
    let mut game = table();
    game.columns[0] = cards("9C 8C 7H 6S");
    game.columns[1] = cards("8S");
    game.columns[2] = cards("8H 5D");
    for count in [0, 5, 255] {
        let mv = transfer(tab(0), tab(1), count);
        assert_rejected(&game, mv, CountOutOfRange { count });
    }
    let below_the_run = transfer(tab(2), tab(1), 2);
    assert_rejected(&game, below_the_run, CountOutOfRange { count: 2 });
    let zero = transfer(tab(2), tab(1), 0);
    assert_rejected(&game, zero, CountOutOfRange { count: 0 });
}

#[test]
fn an_empty_column_takes_any_card_or_run() {
    let mut game = table();
    game.columns[0] = cards("KH QS");
    game.columns[1] = cards("3D 2C");
    assert_accepted(&mut game, transfer(tab(0), tab(2), 2));
    assert_eq!(cards_of(&game, tab(2)), cards("KH QS"));
    assert_accepted(&mut game, transfer(tab(1), tab(3), 1));
    assert_eq!(cards_of(&game, tab(3)), cards("2C"));
}

#[test]
fn an_empty_source_gives_nothing() {
    let mut game = table();
    game.columns[0] = cards("7H");
    assert_rejected(&game, transfer(tab(1), tab(0), 1), EmptySource(tab(1)));
    assert_rejected(&game, transfer(cell(0), tab(0), 1), EmptySource(cell(0)));
}

#[test]
fn missing_piles_are_reported_not_panicked_on() {
    let mut game = table();
    game.columns[0] = cards("7H");
    let missing = [
        tab(8),
        found(4),
        cell(4),
        PileRef::Stock,
        PileRef::Waste,
        PileRef::Pyramid { row: 0, index: 0 },
    ];
    for pile in missing {
        assert_rejected(&game, transfer(pile, tab(0), 1), NoSuchPile(pile));
        assert_rejected(&game, transfer(tab(0), pile, 1), NoSuchPile(pile));
    }
}

#[test]
fn moving_a_card_never_turns_anything_over() {
    let mut game = table();
    game.columns[0] = cards("9C 8D");
    game.columns[1] = cards("9S");
    assert_accepted(&mut game, transfer(tab(0), tab(1), 1));
    assert_eq!(cards_of(&game, tab(0)), cards("9C"));
    assert_eq!(game.columns[0].len(), 1);
    assert!(
        game.piles()
            .iter()
            .flat_map(|p| &p.cards)
            .all(|c| c.face_up)
    );
}

#[test]
fn the_game_is_won_with_52_cards_on_the_foundations() {
    let mut game = table();
    assert!(!game.is_won());
    full_foundations(&mut game);
    assert!(game.is_won());
    assert_eq!(game.hint(), None);
}

#[test]
fn the_last_card_wins_the_game() {
    let mut game = table();
    full_foundations(&mut game);
    let king = game.foundations[3].pop().unwrap();
    game.cells[0].push(king);
    assert!(!game.is_won());
    assert_accepted(&mut game, transfer(cell(0), found(3), 1));
    assert!(game.is_won());
}
