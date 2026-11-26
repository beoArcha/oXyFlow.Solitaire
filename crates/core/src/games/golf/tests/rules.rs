use super::*;

fn with_column(waste: &str, column: &str) -> Golf {
    let mut game = table(waste, "");
    set_column(&mut game, 0, column);
    game
}

#[test]
fn a_card_one_rank_above_the_waste_top_is_played() {
    let mut game = with_column("6D", "9S 7H");
    assert_accepted(&mut game, play(0));
    assert_eq!(cards_of(&game, PileRef::Waste), cards("6D 7H"));
    assert_eq!(cards_of(&game, tab(0)), cards("9S"));
}

#[test]
fn a_card_one_rank_below_the_waste_top_is_played() {
    let mut game = with_column("6D", "9S 5H");
    assert_accepted(&mut game, play(0));
    assert_eq!(cards_of(&game, PileRef::Waste), cards("6D 5H"));
}

#[test]
fn suit_and_color_do_not_matter() {
    for code in ["7C", "7D", "7H", "7S"] {
        let mut game = with_column("6D", code);
        assert_accepted(&mut game, play(0));
    }
}

#[test]
fn only_the_top_card_of_a_column_can_be_played() {
    let game = with_column("6D", "7H 9S");
    assert_rejected(&game, play(0), MoveError::TargetRejects);
}

#[test]
fn the_same_rank_the_waste_top_or_a_farther_rank_is_refused() {
    for code in ["6C", "8S", "4H", "KD", "AD"] {
        let game = with_column("6D", code);
        assert_rejected(&game, play(0), MoveError::TargetRejects);
    }
}

#[test]
fn a_king_does_not_go_onto_an_ace_and_an_ace_not_onto_a_king() {
    let game = with_column("AS", "KH");
    assert_rejected(&game, play(0), MoveError::TargetRejects);
    let game = with_column("KS", "AH");
    assert_rejected(&game, play(0), MoveError::TargetRejects);
}

#[test]
fn only_a_queen_goes_onto_a_king_and_only_a_two_onto_an_ace() {
    let mut game = with_column("KS", "QH");
    assert_accepted(&mut game, play(0));
    let mut game = with_column("AS", "2H");
    assert_accepted(&mut game, play(0));
}

#[test]
fn each_played_card_becomes_the_new_waste_top() {
    let mut game = with_column("7D", "3C 4H 5S 6S");
    for expected in ["6S", "5S", "4H", "3C"] {
        assert_accepted(&mut game, play(0));
        assert_eq!(game.waste.last().copied(), Some(card(expected)));
    }
    assert!(game.is_won());
}

#[test]
fn an_empty_column_has_nothing_to_play() {
    let game = with_column("6D", "7H");
    assert_rejected(&game, play(1), MoveError::EmptySource(tab(1)));
}

#[test]
fn a_count_other_than_one_is_refused() {
    let game = with_column("6D", "5C 7H");
    for count in [0, 2, 5] {
        let mv = Move::Transfer {
            from: tab(0),
            to: PileRef::Waste,
            count,
        };
        assert_rejected(&game, mv, MoveError::CountOutOfRange { count });
    }
}

#[test]
fn only_the_waste_is_a_target() {
    let mut game = with_column("6D", "7H");
    set_column(&mut game, 1, "KS");
    for to in [tab(1), tab(0), PileRef::Stock] {
        let mv = Move::Transfer {
            from: tab(0),
            to,
            count: 1,
        };
        assert_rejected(&game, mv, MoveError::TargetRejects);
    }
}

#[test]
fn cards_cannot_leave_the_waste_or_the_stock() {
    let mut game = table("6D", "9C");
    set_column(&mut game, 0, "7H");
    for from in [PileRef::Waste, PileRef::Stock] {
        let mv = Move::Transfer {
            from,
            to: tab(0),
            count: 1,
        };
        assert_rejected(&game, mv, MoveError::Unsupported);
    }
}

#[test]
fn piles_of_other_games_do_not_exist() {
    let game = with_column("6D", "7H");
    for pile in [
        PileRef::Foundation(0),
        PileRef::Cell(0),
        PileRef::Pyramid { row: 0, index: 0 },
        tab(7),
    ] {
        let from = Move::Transfer {
            from: pile,
            to: PileRef::Waste,
            count: 1,
        };
        assert_rejected(&game, from, MoveError::NoSuchPile(pile));
        let to = Move::Transfer {
            from: tab(0),
            to: pile,
            count: 1,
        };
        assert_rejected(&game, to, MoveError::NoSuchPile(pile));
    }
}

#[test]
fn removing_a_pair_is_not_a_golf_move() {
    let game = with_column("6D", "7H");
    let remove = Move::Remove {
        first: tab(0),
        second: None,
    };
    assert_rejected(&game, remove, MoveError::Unsupported);
}

#[test]
fn the_stock_turns_one_card_onto_the_waste() {
    let mut game = table("6D", "2C 9H");
    set_column(&mut game, 0, "KS");
    assert_accepted(&mut game, Move::Stock);
    assert_eq!(cards_of(&game, PileRef::Waste), cards("6D 9H"));
    assert_eq!(cards_of(&game, PileRef::Stock), cards("2C"));
    assert!(game.piles().iter().any(|p| p.pile == PileRef::Stock));
}

#[test]
fn an_empty_stock_cannot_be_turned_and_is_not_redealt() {
    let mut game = table("6D 9H", "");
    set_column(&mut game, 0, "KS");
    assert_rejected(&game, Move::Stock, MoveError::StockEmpty);
    assert_eq!(cards_of(&game, PileRef::Waste), cards("6D 9H"));
}

#[test]
fn the_legal_moves_are_the_stock_and_every_playable_column() {
    let mut game = table("6D", "2C");
    set_column(&mut game, 0, "9S 7H");
    set_column(&mut game, 1, "KS");
    set_column(&mut game, 3, "5C");
    set_column(&mut game, 6, "7S");
    let mut moves = game.legal_moves();
    moves.sort_by_key(|mv| format!("{mv:?}"));
    let mut expected = vec![Move::Stock, play(0), play(3), play(6)];
    expected.sort_by_key(|mv| format!("{mv:?}"));
    assert_eq!(moves, expected);
}

#[test]
fn clearing_the_last_column_wins_even_with_cards_left_in_the_stock() {
    let mut game = table("6D", "2C 9H");
    set_column(&mut game, 0, "7H");
    assert!(!game.is_won());
    assert_accepted(&mut game, play(0));
    assert!(game.is_won());
    assert_eq!(game.legal_moves(), Vec::new());
    assert_eq!(game.hint(), None);
    assert_eq!(game.apply(Move::Stock), Err(MoveError::Unsupported));
}

#[test]
fn every_dealt_game_keeps_its_cards_through_random_play() {
    for seed in 0..10 {
        let mut game = Golf::deal(seed);
        for _ in 0..60 {
            let moves = game.legal_moves();
            let Some(&mv) = moves.last() else { break };
            assert_eq!(game.apply(mv), Ok(()), "seed {seed} {mv:?}");
        }
        let total: usize = game.piles().iter().map(|p| p.cards.len()).sum();
        assert_eq!(total, 52, "seed {seed}");
        assert!(game.columns.len() == COLUMNS);
    }
}

#[test]
fn king_ace_wrap_allows_king_on_ace_and_ace_on_king_when_enabled() {
    let mut game = with_column("AS", "KH");
    game.wrap = true;
    assert_accepted(&mut game, play(0));

    let mut game = with_column("KS", "AH");
    game.wrap = true;
    assert_accepted(&mut game, play(0));
}

#[test]
fn king_ends_waste_refuses_queen_and_wrapped_ace_on_king() {
    let mut game = with_column("KS", "QH");
    game.king_stops = true;
    assert_rejected(&game, play(0), MoveError::TargetRejects);

    let mut wrapped = with_column("KS", "AH");
    wrapped.wrap = true;
    wrapped.king_stops = true;
    assert_rejected(&wrapped, play(0), MoveError::TargetRejects);

    let mut onto_ace = with_column("AS", "KH");
    onto_ace.wrap = true;
    onto_ace.king_stops = true;
    assert_accepted(&mut onto_ace, play(0));
}
