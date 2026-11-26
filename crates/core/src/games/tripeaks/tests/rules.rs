use super::*;

fn with_bottom(waste: &str, code: &str) -> TriPeaks {
    let mut game = empty_game(waste, "");
    put(&mut game, 3, 0, code, true);
    game
}

#[test]
fn a_card_one_rank_above_or_below_the_waste_top_is_played() {
    let mut game = with_bottom("6D", "7H");
    assert_accepted(&mut game, play(3, 0));
    assert_eq!(cards_of(&game, PileRef::Waste), cards("6D 7H"));
    assert_eq!(cards_of(&game, peak(3, 0)), Vec::new());

    let mut game = with_bottom("6D", "5H");
    assert_accepted(&mut game, play(3, 0));
    assert_eq!(cards_of(&game, PileRef::Waste), cards("6D 5H"));
}

#[test]
fn suit_and_color_do_not_matter() {
    for code in ["7C", "7D", "7H", "7S"] {
        let mut game = with_bottom("6D", code);
        assert_accepted(&mut game, play(3, 0));
    }
}

#[test]
fn king_and_ace_wrap_in_both_directions() {
    let mut game = with_bottom("AS", "KH");
    assert_accepted(&mut game, play(3, 0));
    assert_eq!(cards_of(&game, PileRef::Waste), cards("AS KH"));

    let mut game = with_bottom("KS", "AH");
    assert_accepted(&mut game, play(3, 0));
    assert_eq!(cards_of(&game, PileRef::Waste), cards("KS AH"));
}

#[test]
fn the_same_rank_or_a_farther_rank_is_refused() {
    for code in ["6C", "8S", "4H", "KD", "AD"] {
        let game = with_bottom("6D", code);
        assert_rejected(&game, play(3, 0), MoveError::TargetRejects);
    }
}

#[test]
fn a_card_turns_face_up_only_when_both_cards_covering_it_are_gone() {
    let mut game = empty_game("6D", "");
    put(&mut game, 2, 0, "9S", false);
    put(&mut game, 3, 0, "7H", true);
    put(&mut game, 3, 1, "8C", true);
    assert!(game.is_covered(peak(2, 0)));
    assert_rejected(&game, play(2, 0), MoveError::Blocked);

    assert_accepted(&mut game, play(3, 0));
    assert!(game.is_covered(peak(2, 0)));
    assert!(!game.slot(2, 0).unwrap().face_up);
    assert_rejected(&game, play(2, 0), MoveError::Blocked);

    assert_accepted(&mut game, play(3, 1));
    assert!(!game.is_covered(peak(2, 0)));
    assert!(game.slot(2, 0).unwrap().face_up);
    assert_accepted(&mut game, play(2, 0));
    assert!(game.is_won());
}

#[test]
fn covering_follows_all_three_peaks_correctly() {
    let mut game = empty_game("6D", "");
    put(&mut game, 0, 1, "2C", false);
    put(&mut game, 1, 2, "3C", false);
    put(&mut game, 1, 3, "4C", false);
    put(&mut game, 2, 3, "5C", false);
    put(&mut game, 2, 4, "7C", false);
    put(&mut game, 2, 5, "8C", false);
    put(&mut game, 3, 4, "7H", true);
    assert!(game.is_covered(peak(2, 3)));
    assert!(game.is_covered(peak(2, 4)));
    assert!(!game.is_covered(peak(2, 5)));

    assert_accepted(&mut game, play(3, 4));
    assert!(!game.is_covered(peak(2, 3)));
    assert!(!game.is_covered(peak(2, 4)));
    assert!(game.slot(2, 3).unwrap().face_up);
    assert!(game.slot(2, 4).unwrap().face_up);
    assert!(game.is_covered(peak(1, 2)));
    assert!(game.is_covered(peak(1, 3)));
    assert!(game.is_covered(peak(0, 1)));
}

#[test]
fn an_empty_slot_has_nothing_to_play() {
    let game = with_bottom("6D", "7H");
    assert_rejected(&game, play(3, 1), MoveError::EmptySource(peak(3, 1)));
}

#[test]
fn a_count_other_than_one_is_refused() {
    let game = with_bottom("6D", "7H");
    for count in [0, 2, 5] {
        let mv = Move::Transfer {
            from: peak(3, 0),
            to: PileRef::Waste,
            count,
        };
        assert_rejected(&game, mv, MoveError::CountOutOfRange { count });
    }
}

#[test]
fn only_the_waste_is_a_target_and_cards_cannot_leave_waste_or_stock() {
    let mut game = empty_game("6D", "9C");
    put(&mut game, 3, 0, "7H", true);
    put(&mut game, 3, 1, "KS", true);
    for to in [peak(3, 1), peak(3, 0), PileRef::Stock] {
        let mv = Move::Transfer {
            from: peak(3, 0),
            to,
            count: 1,
        };
        assert_rejected(&game, mv, MoveError::TargetRejects);
    }
    for from in [PileRef::Waste, PileRef::Stock] {
        let mv = Move::Transfer {
            from,
            to: peak(3, 0),
            count: 1,
        };
        assert_rejected(&game, mv, MoveError::Unsupported);
    }
}

#[test]
fn piles_outside_tripeaks_do_not_exist() {
    let game = with_bottom("6D", "7H");
    for pile in [
        PileRef::Foundation(0),
        PileRef::Cell(0),
        PileRef::Tableau(0),
        peak(0, 3),
        peak(1, 6),
        peak(2, 9),
        peak(3, 10),
        peak(4, 0),
    ] {
        let from = Move::Transfer {
            from: pile,
            to: PileRef::Waste,
            count: 1,
        };
        assert_rejected(&game, from, MoveError::NoSuchPile(pile));
    }
}

#[test]
fn removing_is_unsupported() {
    let game = with_bottom("6D", "7H");
    let remove = Move::Remove {
        first: peak(3, 0),
        second: None,
    };
    assert_rejected(&game, remove, MoveError::Unsupported);
}

#[test]
fn the_stock_turns_one_card_onto_the_waste_with_no_redeal() {
    let mut game = empty_game("6D", "2C 9H");
    put(&mut game, 3, 0, "KS", true);
    assert_accepted(&mut game, Move::Stock);
    assert_eq!(cards_of(&game, PileRef::Waste), cards("6D 9H"));
    assert_eq!(cards_of(&game, PileRef::Stock), cards("2C"));

    assert_accepted(&mut game, Move::Stock);
    assert_eq!(cards_of(&game, PileRef::Stock), Vec::new());
    assert_rejected(&game, Move::Stock, MoveError::StockEmpty);
}

#[test]
fn clearing_the_last_peak_card_wins_even_with_cards_left_in_the_stock() {
    let mut game = empty_game("6D", "2C 9H");
    put(&mut game, 0, 0, "7H", true);
    assert!(!game.is_won());
    assert_accepted(&mut game, play(0, 0));
    assert!(game.is_won());
    assert_eq!(game.legal_moves(), Vec::new());
    assert_eq!(game.hint(), None);
    assert_eq!(game.apply(Move::Stock), Err(MoveError::Unsupported));
}
