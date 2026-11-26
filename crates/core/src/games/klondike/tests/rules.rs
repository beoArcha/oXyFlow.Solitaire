use super::*;
use crate::moves::MoveError::{CountOutOfRange, EmptySource, NoSuchPile, TargetRejects};

#[test]
fn an_ace_goes_to_any_empty_foundation() {
    let mut game = table();
    game.columns[0] = column("", "AS");
    for index in 0..4 {
        let mv = transfer(tab(0), found(index), 1);
        assert!(game.legal_moves().contains(&mv));
    }
    assert_accepted(&mut game, transfer(tab(0), found(2), 1));
    assert_eq!(cards_of(&game, found(2)), cards("AS"));
}

#[test]
fn foundations_build_up_in_suit() {
    let mut game = table();
    game.foundations[0] = cards("AS");
    game.foundations[1] = cards("AH");
    game.columns[0] = column("", "2S");
    game.columns[1] = column("", "2H");
    game.columns[2] = column("", "3S");
    game.columns[3] = column("", "2C");
    assert_rejected(&game, transfer(tab(0), found(1), 1), TargetRejects);
    assert_rejected(&game, transfer(tab(2), found(0), 1), TargetRejects);
    assert_rejected(&game, transfer(tab(3), found(2), 1), TargetRejects);
    assert_rejected(&game, transfer(tab(3), found(0), 1), TargetRejects);
    assert_accepted(&mut game, transfer(tab(0), found(0), 1));
    assert_accepted(&mut game, transfer(tab(2), found(0), 1));
    assert_eq!(cards_of(&game, found(0)), cards("AS 2S 3S"));
}

#[test]
fn a_foundation_takes_only_one_card_at_a_time() {
    let mut game = table();
    game.columns[0] = column("", "2S AH");
    let mv = transfer(tab(0), found(0), 2);
    assert_rejected(&game, mv, CountOutOfRange { count: 2 });
}

#[test]
fn tableau_builds_down_in_alternating_colors() {
    let mut game = table();
    game.columns[0] = column("", "7H");
    game.columns[1] = column("", "8S");
    game.columns[2] = column("", "8H");
    game.columns[3] = column("", "9S");
    assert_rejected(&game, transfer(tab(0), tab(2), 1), TargetRejects);
    assert_rejected(&game, transfer(tab(0), tab(3), 1), TargetRejects);
    assert_rejected(&game, transfer(tab(1), tab(0), 1), TargetRejects);
    assert_accepted(&mut game, transfer(tab(0), tab(1), 1));
    assert_eq!(cards_of(&game, tab(1)), cards("8S 7H"));
    assert_eq!(cards_of(&game, tab(0)), cards(""));
}

#[test]
fn a_whole_run_moves_and_a_part_of_it_too() {
    let mut part = table();
    part.columns[0] = column("9C", "7H 6S 5D");
    part.columns[1] = column("", "6C");
    assert_accepted(&mut part, transfer(tab(0), tab(1), 1));
    assert_eq!(cards_of(&part, tab(1)), cards("6C 5D"));
    let mut whole = table();
    whole.columns[0] = column("9C", "7H 6S 5D");
    whole.columns[1] = column("", "8S");
    assert_accepted(&mut whole, transfer(tab(0), tab(1), 3));
    assert_eq!(cards_of(&whole, tab(1)), cards("8S 7H 6S 5D"));
}

#[test]
fn counts_outside_the_run_are_out_of_range() {
    let mut game = table();
    game.columns[0] = column("9C 8C", "7H 6S");
    game.columns[1] = column("", "8S");
    for count in [0, 3, 4, 255] {
        let mv = transfer(tab(0), tab(1), count);
        assert_rejected(&game, mv, CountOutOfRange { count });
    }
    game.waste = cards("6S 5H");
    let two = transfer(PileRef::Waste, tab(1), 2);
    assert_rejected(&game, two, CountOutOfRange { count: 2 });
    let zero = transfer(PileRef::Waste, tab(1), 0);
    assert_rejected(&game, zero, CountOutOfRange { count: 0 });
}

#[test]
fn an_empty_column_takes_only_a_king_or_a_run_starting_with_one() {
    let mut game = table();
    game.columns[0] = column("", "KH QS");
    game.columns[1] = column("", "QD");
    game.columns[2] = column("", "JC");
    assert_rejected(&game, transfer(tab(1), tab(3), 1), TargetRejects);
    assert_rejected(&game, transfer(tab(2), tab(3), 1), TargetRejects);
    assert_rejected(&game, transfer(tab(0), tab(3), 1), TargetRejects);
    assert_accepted(&mut game, transfer(tab(0), tab(3), 2));
    assert_eq!(cards_of(&game, tab(3)), cards("KH QS"));
}

#[test]
fn the_waste_gives_only_its_top_card() {
    let mut game = table();
    game.waste = cards("AS 6D");
    game.columns[0] = column("", "7S");
    game.foundations[0] = cards("AH");
    assert_accepted(&mut game, transfer(PileRef::Waste, tab(0), 1));
    assert_eq!(cards_of(&game, PileRef::Waste), cards("AS"));
    assert_accepted(&mut game, transfer(PileRef::Waste, found(1), 1));
    assert_eq!(cards_of(&game, PileRef::Waste), cards(""));
    let empty = transfer(PileRef::Waste, tab(0), 1);
    assert_rejected(&game, empty, EmptySource(PileRef::Waste));
}

#[test]
fn a_foundation_card_can_come_back_to_the_tableau() {
    let mut game = table();
    game.foundations[0] = cards("AS 2S");
    game.columns[0] = column("", "3H");
    let two = transfer(found(0), tab(0), 2);
    assert_rejected(&game, two, CountOutOfRange { count: 2 });
    assert_rejected(&game, transfer(found(0), found(1), 1), TargetRejects);
    assert_rejected(&game, transfer(found(0), found(0), 1), TargetRejects);
    assert_accepted(&mut game, transfer(found(0), tab(0), 1));
    assert_eq!(cards_of(&game, found(0)), cards("AS"));
    assert_rejected(&game, transfer(found(0), found(1), 1), TargetRejects);
}

#[test]
fn other_targets_and_sources_are_refused() {
    let mut game = table();
    game.stock = cards("AS");
    game.waste = cards("2S");
    game.columns[0] = column("", "7H");
    assert_rejected(&game, transfer(tab(0), PileRef::Stock, 1), TargetRejects);
    assert_rejected(&game, transfer(tab(0), PileRef::Waste, 1), TargetRejects);
    assert_rejected(&game, transfer(tab(0), tab(0), 1), TargetRejects);
    let from_stock = transfer(PileRef::Stock, tab(0), 1);
    assert_rejected(&game, from_stock, MoveError::Unsupported);
    let remove = Move::Remove {
        first: tab(0),
        second: None,
    };
    assert_rejected(&game, remove, MoveError::Unsupported);
    assert_rejected(&game, transfer(tab(1), tab(0), 1), EmptySource(tab(1)));
    assert_rejected(&game, transfer(found(3), tab(0), 1), EmptySource(found(3)));
}

#[test]
fn missing_piles_are_reported_not_panicked_on() {
    let mut game = table();
    game.columns[0] = column("", "7H");
    let pyramid = PileRef::Pyramid { row: 0, index: 0 };
    for missing in [tab(7), found(4), PileRef::Cell(0), pyramid] {
        let from = transfer(missing, tab(0), 1);
        assert_rejected(&game, from, NoSuchPile(missing));
        let to = transfer(tab(0), missing, 1);
        assert_rejected(&game, to, NoSuchPile(missing));
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
fn nothing_flips_when_a_column_empties_or_part_of_a_run_stays() {
    let mut game = table();
    game.columns[0] = column("", "KS");
    game.columns[1] = column("", "AS");
    game.columns[2] = column("4C", "7H 6S");
    game.columns[3] = column("", "7D");
    assert_accepted(&mut game, transfer(tab(0), tab(4), 1));
    assert!(game.columns[0].up.is_empty() && game.columns[0].down.is_empty());
    assert_accepted(&mut game, transfer(tab(2), tab(3), 1));
    assert_eq!(game.columns[2].down, cards("4C"));
    assert_eq!(game.columns[2].up, cards("7H"));
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
fn the_last_move_wins_the_game() {
    let mut game = table();
    full_foundations(&mut game);
    let king = game.foundations[3].pop().unwrap();
    game.columns[0].up.push(king);
    assert!(!game.is_won());
    assert_accepted(&mut game, transfer(tab(0), found(3), 1));
    assert!(game.is_won());
}

#[test]
fn a_won_game_accepts_no_moves() {
    let mut game = table();
    full_foundations(&mut game);
    assert!(game.legal_moves().is_empty());
    assert_eq!(game.hint(), None);
    let king_back = transfer(found(3), tab(0), 1);
    assert_rejected(&game, king_back, MoveError::Unsupported);
    assert_rejected(&game, Move::Stock, MoveError::Unsupported);
}

#[test]
fn any_card_on_empty_column_allows_non_kings_when_enabled() {
    let options = crate::game::GameOptions {
        klondike_any_empty: true,
        ..Default::default()
    };
    let mut game = Klondike::empty_with(DrawCount::One, options);
    game.columns[0] = column("", "KH QS");
    game.columns[1] = column("", "QD");
    game.waste = cards("7C");
    assert_accepted(&mut game, transfer(tab(1), tab(3), 1));
    assert_eq!(cards_of(&game, tab(3)), cards("QD"));
    assert_accepted(&mut game, transfer(PileRef::Waste, tab(4), 1));
    assert_eq!(cards_of(&game, tab(4)), cards("7C"));
    assert_accepted(&mut game, transfer(tab(0), tab(5), 2));
    assert_eq!(cards_of(&game, tab(5)), cards("KH QS"));
}
