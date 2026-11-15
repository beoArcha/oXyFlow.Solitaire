use super::*;
use crate::moves::MoveError::{CountOutOfRange, EmptySource, InvalidSequence, NoSuchPile};
use crate::moves::MoveError::{TargetRejects, Unsupported};

#[test]
fn tableau_builds_down_in_any_suit() {
    let mut game = table();
    game.columns[0] = column("", "7H");
    game.columns[1] = column("", "8S");
    game.columns[2] = column("", "9S");
    game.columns[3] = column("", "7C");
    game.columns[4] = column("", "8D");
    assert_rejected(&game, transfer(tab(0), tab(2), 1), TargetRejects);
    assert_rejected(&game, transfer(tab(1), tab(0), 1), TargetRejects);
    assert_rejected(&game, transfer(tab(0), tab(3), 1), TargetRejects);
    assert_accepted(&mut game, transfer(tab(0), tab(1), 1));
    assert_eq!(cards_of(&game, tab(1)), cards("8S 7H"));
    assert_eq!(cards_of(&game, tab(0)), cards(""));
    assert_accepted(&mut game, transfer(tab(3), tab(4), 1));
    assert_eq!(cards_of(&game, tab(4)), cards("8D 7C"));
}

#[test]
fn a_same_suit_run_moves_as_a_group() {
    let mut game = table();
    game.columns[0] = column("", "9H 8S 7S");
    game.columns[1] = column("", "10C");
    game.columns[2] = column("", "9D");
    assert_rejected(&game, transfer(tab(0), tab(1), 3), InvalidSequence);
    assert_accepted(&mut game, transfer(tab(0), tab(2), 2));
    assert_eq!(cards_of(&game, tab(2)), cards("9D 8S 7S"));
    assert_eq!(cards_of(&game, tab(0)), cards("9H"));
}

#[test]
fn a_run_of_mixed_suits_does_not_move_as_a_group() {
    let mut game = table();
    game.columns[0] = column("", "8S 7H 6S");
    game.columns[1] = column("", "9C");
    game.columns[2] = column("", "7D");
    assert_rejected(&game, transfer(tab(0), tab(1), 2), InvalidSequence);
    assert_rejected(&game, transfer(tab(0), tab(1), 3), InvalidSequence);
    assert_accepted(&mut game, transfer(tab(0), tab(2), 1));
}

#[test]
fn a_run_must_descend_by_exactly_one_rank() {
    let mut game = table();
    game.columns[0] = column("", "9S 7S");
    game.columns[1] = column("", "10H");
    assert_rejected(&game, transfer(tab(0), tab(1), 2), InvalidSequence);
}

#[test]
fn an_empty_column_takes_any_card_or_run() {
    let mut game = table();
    game.columns[0] = column("", "9H 8H");
    game.columns[1] = column("", "3S");
    assert_accepted(&mut game, transfer(tab(1), tab(2), 1));
    assert_accepted(&mut game, transfer(tab(0), tab(1), 2));
    assert_eq!(cards_of(&game, tab(1)), cards("9H 8H"));
    assert_eq!(cards_of(&game, tab(2)), cards("3S"));
}

#[test]
fn counts_outside_the_face_up_cards_are_out_of_range() {
    let mut game = table();
    game.columns[0] = column("9C 8C", "7H 6H");
    game.columns[1] = column("", "8S");
    for count in [0, 3, 4, 255] {
        let mv = transfer(tab(0), tab(1), count);
        assert_rejected(&game, mv, CountOutOfRange { count });
    }
    assert_accepted(&mut game, transfer(tab(0), tab(1), 2));
}

#[test]
fn an_empty_column_gives_empty_source() {
    let mut game = table();
    game.columns[0] = column("", "7H");
    assert_rejected(&game, transfer(tab(1), tab(0), 1), EmptySource(tab(1)));
}

#[test]
fn nothing_moves_from_the_stock_or_a_foundation_and_nothing_goes_there() {
    let mut game = table();
    game.stock = cards("AS");
    game.foundations[0] = cards(SPADES_RUN);
    game.columns[0] = column("", "7H");
    assert_rejected(&game, transfer(PileRef::Stock, tab(0), 1), Unsupported);
    assert_rejected(&game, transfer(found(0), tab(0), 1), Unsupported);
    assert_rejected(&game, transfer(found(3), tab(0), 1), Unsupported);
    assert_rejected(&game, transfer(tab(0), found(3), 1), TargetRejects);
    assert_rejected(&game, transfer(tab(0), PileRef::Stock, 1), TargetRejects);
    assert_rejected(&game, transfer(tab(0), tab(0), 1), TargetRejects);
}

#[test]
fn remove_is_not_supported() {
    let game = table();
    let remove = Move::Remove {
        first: tab(0),
        second: None,
    };
    assert_rejected(&game, remove, Unsupported);
}

#[test]
fn missing_piles_are_reported_not_panicked_on() {
    let mut game = table();
    game.columns[0] = column("", "7H");
    let pyramid = PileRef::Pyramid { row: 0, index: 0 };
    let missing = [tab(10), found(8), PileRef::Waste, PileRef::Cell(0), pyramid];
    for pile in missing {
        assert_rejected(&game, transfer(pile, tab(0), 1), NoSuchPile(pile));
        assert_rejected(&game, transfer(tab(0), pile, 1), NoSuchPile(pile));
    }
}

#[test]
fn a_face_down_card_flips_when_exposed() {
    let mut game = table();
    game.columns[0] = column("9C QD", "JS");
    game.columns[1] = column("", "QH");
    assert_accepted(&mut game, transfer(tab(0), tab(1), 1));
    assert_eq!(game.columns[0].down, cards("9C"));
    assert_eq!(game.columns[0].up, cards("QD"));
}

#[test]
fn nothing_flips_when_a_column_empties_or_part_of_the_cards_stay() {
    let mut game = table();
    game.columns[0] = column("", "7S");
    game.columns[1] = column("", "8H");
    game.columns[2] = column("4C", "9H 7D");
    game.columns[3] = column("", "8D");
    assert_accepted(&mut game, transfer(tab(0), tab(1), 1));
    assert!(game.columns[0].up.is_empty() && game.columns[0].down.is_empty());
    assert_accepted(&mut game, transfer(tab(2), tab(3), 1));
    assert_eq!(game.columns[2].down, cards("4C"));
    assert_eq!(game.columns[2].up, cards("9H"));
}

#[test]
fn the_game_is_won_with_eight_complete_runs() {
    let mut game = table();
    assert!(!game.is_won());
    for foundation in &mut game.foundations {
        *foundation = cards(SPADES_RUN);
    }
    assert!(game.is_won());
    assert_eq!(game.hint(), None);
}

#[test]
fn the_last_move_wins_the_game() {
    let mut game = table();
    for foundation in game.foundations.iter_mut().take(7) {
        *foundation = cards(HEARTS_RUN);
    }
    game.columns[0] = column("", "KS QS JS 10S 9S 8S 7S 6S 5S 4S 3S 2S");
    game.columns[1] = column("", "AS");
    assert!(!game.is_won());
    assert_accepted(&mut game, transfer(tab(1), tab(0), 1));
    assert!(game.is_won());
}

#[test]
fn a_won_game_accepts_no_moves() {
    let mut game = table();
    for foundation in &mut game.foundations {
        *foundation = cards(SPADES_RUN);
    }
    assert!(game.legal_moves().is_empty());
    assert_eq!(game.hint(), None);
    assert_rejected(&game, Move::Stock, Unsupported);
    assert_rejected(&game, transfer(tab(0), tab(1), 1), Unsupported);
}
