use super::*;
use crate::moves::MoveError::{CountOutOfRange, EmptySource, NoSuchPile, TargetRejects};

#[test]
fn a_group_out_of_sequence_moves_when_its_bottom_card_fits() {
    let mut game = table();
    game.columns[0] = column("4C", "6H 2S KD 9C");
    game.columns[1] = column("", "7S");
    assert_accepted(&mut game, transfer(tab(0), tab(1), 4));
    assert_eq!(cards_of(&game, tab(1)), cards("7S 6H 2S KD 9C"));
    assert_eq!(cards_of(&game, tab(0)), cards("4C"));
    assert_eq!(face_up_of(&game, tab(0)), vec![true]);
}

#[test]
fn only_the_bottom_card_of_the_group_is_checked() {
    let mut game = table();
    game.columns[0] = column("", "6S 5H QC");
    game.columns[1] = column("", "7S");
    game.columns[2] = column("", "6C");
    assert_rejected(&game, transfer(tab(0), tab(1), 3), TargetRejects);
    assert_rejected(&game, transfer(tab(0), tab(2), 1), TargetRejects);
    assert_accepted(&mut game, transfer(tab(0), tab(2), 2));
    assert_eq!(cards_of(&game, tab(2)), cards("6C 5H QC"));
}

#[test]
fn columns_build_down_in_alternating_colours() {
    let mut game = table();
    game.columns[0] = column("", "7H");
    game.columns[1] = column("", "8H");
    game.columns[2] = column("", "9S");
    game.columns[3] = column("", "6H");
    game.columns[4] = column("", "8C");
    assert_rejected(&game, transfer(tab(0), tab(1), 1), TargetRejects);
    assert_rejected(&game, transfer(tab(0), tab(2), 1), TargetRejects);
    assert_rejected(&game, transfer(tab(3), tab(0), 1), TargetRejects);
    assert_accepted(&mut game, transfer(tab(0), tab(4), 1));
    assert_eq!(cards_of(&game, tab(4)), cards("8C 7H"));
}

#[test]
fn a_face_down_card_turns_over_when_it_is_uncovered() {
    let mut game = table();
    game.columns[0] = column("2D 3C", "5H");
    game.columns[1] = column("", "6S");
    assert_accepted(&mut game, transfer(tab(0), tab(1), 1));
    assert_eq!(cards_of(&game, tab(0)), cards("2D 3C"));
    assert_eq!(face_up_of(&game, tab(0)), vec![false, true]);
}

#[test]
fn face_down_cards_cannot_move() {
    let mut game = table();
    game.columns[0] = column("9D", "5H");
    game.columns[1] = column("", "10S");
    assert_rejected(
        &game,
        transfer(tab(0), tab(1), 2),
        CountOutOfRange { count: 2 },
    );
    assert_rejected(
        &game,
        transfer(tab(0), tab(1), 0),
        CountOutOfRange { count: 0 },
    );
}

#[test]
fn an_empty_column_takes_only_a_king_or_a_group_with_a_king_at_the_bottom() {
    let mut game = table();
    game.columns[0] = column("3S", "QH KC 4D");
    game.columns[2] = column("", "QS");
    assert_rejected(&game, transfer(tab(0), tab(1), 1), TargetRejects);
    assert_rejected(&game, transfer(tab(0), tab(1), 3), TargetRejects);
    assert_rejected(&game, transfer(tab(2), tab(1), 1), TargetRejects);
    assert_accepted(&mut game, transfer(tab(0), tab(1), 2));
    assert_eq!(cards_of(&game, tab(1)), cards("KC 4D"));
    assert_eq!(cards_of(&game, tab(0)), cards("3S QH"));
}

#[test]
fn foundations_build_up_by_suit_one_card_at_a_time() {
    let mut game = table();
    game.columns[0] = column("", "2S AS");
    game.columns[1] = column("", "AH 3S");
    assert_rejected(&game, transfer(tab(1), found(0), 1), TargetRejects);
    assert_rejected(
        &game,
        transfer(tab(0), found(0), 2),
        CountOutOfRange { count: 2 },
    );
    assert_accepted(&mut game, transfer(tab(0), found(0), 1));
    assert_accepted(&mut game, transfer(tab(0), found(0), 1));
    assert_rejected(&game, transfer(tab(1), found(1), 1), TargetRejects);
    assert_accepted(&mut game, transfer(tab(1), found(0), 1));
    assert_eq!(cards_of(&game, found(0)), cards("AS 2S 3S"));
    assert_accepted(&mut game, transfer(tab(1), found(1), 1));
    assert_eq!(cards_of(&game, found(1)), cards("AH"));
}

#[test]
fn a_foundation_card_may_come_back_to_a_column_but_not_to_another_foundation() {
    let mut game = table();
    game.foundations[0] = cards("AD 2D 3D 4D 5D 6D");
    game.columns[0] = column("", "7C");
    assert_rejected(&game, transfer(found(0), found(1), 1), TargetRejects);
    assert_accepted(&mut game, transfer(found(0), tab(0), 1));
    assert_eq!(cards_of(&game, tab(0)), cards("7C 6D"));
}

#[test]
fn bad_piles_and_empty_sources_are_refused() {
    let mut game = table();
    game.columns[0] = column("", "KS");
    assert_rejected(&game, transfer(tab(7), tab(0), 1), NoSuchPile(tab(7)));
    assert_rejected(&game, transfer(tab(0), found(4), 1), NoSuchPile(found(4)));
    assert_rejected(
        &game,
        transfer(PileRef::Stock, tab(0), 1),
        NoSuchPile(PileRef::Stock),
    );
    assert_rejected(
        &game,
        transfer(tab(0), PileRef::Waste, 1),
        NoSuchPile(PileRef::Waste),
    );
    assert_rejected(
        &game,
        transfer(tab(0), PileRef::Cell(0), 1),
        NoSuchPile(PileRef::Cell(0)),
    );
    assert_rejected(&game, transfer(tab(1), tab(0), 1), EmptySource(tab(1)));
    assert_rejected(&game, transfer(tab(0), tab(0), 1), TargetRejects);
    assert_accepted(&mut game, transfer(tab(0), tab(1), 1));
}

#[test]
fn a_won_game_is_closed() {
    let mut game = table();
    foundations_up_to(&mut game, 12);
    game.columns[0] = column("", "KC KD KH");
    game.columns[1] = column("", "KS");
    assert!(!game.is_won());
    for (index, from) in [(3, 1), (2, 0), (1, 0), (0, 0)] {
        assert_accepted(&mut game, transfer(tab(from), found(index), 1));
    }
    assert!(game.is_won());
    assert!(game.legal_moves().is_empty());
    assert_eq!(game.hint(), None);
    assert_eq!(game.forced_finish(), None);
    assert_eq!(
        game.apply(transfer(found(0), tab(0), 1)),
        Err(MoveError::Unsupported)
    );
}

#[test]
fn legal_moves_match_apply_on_dealt_games() {
    let targets: Vec<PileRef> = (0..4).map(found).chain((0..7).map(tab)).collect();
    for seed in 0..10 {
        let game = Yukon::deal(YukonRules::Classic, seed);
        let legal = game.legal_moves();
        for &from in &targets {
            for &to in &targets {
                for count in 0..=12 {
                    let mv = transfer(from, to, count);
                    let accepted = game.clone().apply(mv).is_ok();
                    assert_eq!(accepted, legal.contains(&mv), "seed {seed}: {mv:?}");
                }
            }
        }
    }
}
