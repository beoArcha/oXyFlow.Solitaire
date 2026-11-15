use super::*;

const EMPTY_ROWS: [&str; 7] = [
    ".",
    ". .",
    ". . .",
    ". . . .",
    ". . . . .",
    ". . . . . .",
    ". . . . . . .",
];
const BASE_ROW: &str = "6C 7D 2H JS 8C 5D KH";

fn base_table() -> Pyramid {
    let mut rows = EMPTY_ROWS;
    rows[6] = BASE_ROW;
    table(PyramidRules::Relaxed, rows, "", "")
}

const INERT_ROW: &str = ". . . . . . 9C";

/// A table whose pyramid keeps one nine that pairs with none of the stock or waste cards used in
/// these tests, so that a Relaxed game is not already won.
fn stock_table(rules: PyramidRules, stock: &str, waste: &str) -> Pyramid {
    let mut rows = EMPTY_ROWS;
    rows[6] = INERT_ROW;
    table(rules, rows, stock, waste)
}

fn cleared_table(stock: &str) -> Pyramid {
    table(PyramidRules::Relaxed, EMPTY_ROWS, stock, "")
}

#[test]
fn values_follow_the_card() {
    let values: Vec<u8> = ["AS", "2C", "10H", "JD", "QC", "KS"]
        .into_iter()
        .map(|code| value(card(code)))
        .collect();
    assert_eq!(values, [1, 2, 10, 11, 12, 13]);
}

#[test]
fn a_pair_summing_to_13_goes_to_the_foundation() {
    let mut game = base_table();
    assert_accepted(&mut game, remove(pyr(6, 0), Some(pyr(6, 1))));
    assert_eq!(cards_of(&game, PileRef::Foundation(0)), cards("6C 7D"));
    assert!(cards_of(&game, pyr(6, 0)).is_empty());
    assert!(cards_of(&game, pyr(6, 1)).is_empty());
}

#[test]
fn either_order_of_a_pair_is_accepted_but_only_one_is_listed() {
    let mut game = base_table();
    let listed = remove(pyr(6, 2), Some(pyr(6, 3)));
    assert!(game.legal_moves().contains(&listed));
    assert!(!game.legal_moves().contains(&reversed(listed)));
    assert_eq!(game.apply(reversed(listed)), Ok(()));
    assert_eq!(cards_of(&game, PileRef::Foundation(0)), cards("JS 2H"));
}

#[test]
fn pairs_must_sum_to_13_and_must_differ() {
    let game = base_table();
    assert_rejected(
        &game,
        remove(pyr(6, 0), Some(pyr(6, 2))),
        MoveError::NotAPair,
    );
    assert_rejected(
        &game,
        remove(pyr(6, 0), Some(pyr(6, 0))),
        MoveError::NotAPair,
    );
}

#[test]
fn a_king_is_removed_alone() {
    let mut game = base_table();
    assert_accepted(&mut game, remove(pyr(6, 6), None));
    assert_eq!(cards_of(&game, PileRef::Foundation(0)), cards("KH"));
    assert_rejected(&game, remove(pyr(6, 0), None), MoveError::NotAPair);
}

#[test]
fn a_card_under_another_card_is_blocked_until_both_below_are_gone() {
    let mut rows = EMPTY_ROWS;
    rows[5] = "KS . . . . .";
    rows[6] = "6C . . . . . .";
    let mut game = table(PyramidRules::Relaxed, rows, "7S", "");
    assert_rejected(&game, remove(pyr(5, 0), None), MoveError::Blocked);
    assert_accepted(&mut game, remove(PileRef::Stock, Some(pyr(6, 0))));
    assert_accepted(&mut game, remove(pyr(5, 0), None));
}

#[test]
fn a_removed_position_is_an_empty_source() {
    let mut rows = EMPTY_ROWS;
    rows[6] = ". 7D 2H JS 8C 5D KH";
    let game = table(PyramidRules::Relaxed, rows, "", "");
    assert_rejected(
        &game,
        remove(pyr(6, 0), None),
        MoveError::EmptySource(pyr(6, 0)),
    );
}

#[test]
fn missing_piles_and_the_foundation_as_a_source_are_refused() {
    let game = base_table();
    let missing = [
        PileRef::Pyramid { row: 7, index: 0 },
        PileRef::Pyramid { row: 1, index: 2 },
        PileRef::Foundation(1),
        PileRef::Tableau(0),
        PileRef::Cell(0),
    ];
    for pile in missing {
        assert_rejected(&game, remove(pile, None), MoveError::NoSuchPile(pile));
    }
    assert_rejected(
        &game,
        remove(PileRef::Foundation(0), None),
        MoveError::Unsupported,
    );
    assert_rejected(
        &game,
        remove(pyr(6, 0), Some(PileRef::Foundation(0))),
        MoveError::Unsupported,
    );
}

#[test]
fn transfers_are_unsupported() {
    let game = base_table();
    let transfer = Move::Transfer {
        from: PileRef::Stock,
        to: PileRef::Waste,
        count: 1,
    };
    assert_rejected(&game, transfer, MoveError::Unsupported);
}

#[test]
fn a_stock_card_pairs_with_a_waste_card() {
    let mut game = stock_table(PyramidRules::Relaxed, "6S", "7H");
    assert_accepted(&mut game, remove(PileRef::Stock, Some(PileRef::Waste)));
    assert_eq!(cards_of(&game, PileRef::Foundation(0)), cards("6S 7H"));
    assert!(cards_of(&game, PileRef::Stock).is_empty());
    assert!(cards_of(&game, PileRef::Waste).is_empty());
}

#[test]
fn a_stock_or_waste_king_is_removed_alone() {
    let mut game = stock_table(PyramidRules::Relaxed, "KS", "KD");
    assert_accepted(&mut game, remove(PileRef::Waste, None));
    assert_accepted(&mut game, remove(PileRef::Stock, None));
    assert_eq!(cards_of(&game, PileRef::Foundation(0)), cards("KD KS"));
}

#[test]
fn an_empty_stock_is_an_empty_source() {
    let game = stock_table(PyramidRules::Relaxed, "", "7H");
    assert_rejected(
        &game,
        remove(PileRef::Stock, Some(PileRef::Waste)),
        MoveError::EmptySource(PileRef::Stock),
    );
}

#[test]
fn stock_draws_the_top_card_to_the_waste() {
    let mut game = stock_table(PyramidRules::Relaxed, "AS 2S 3S", "");
    assert_accepted(&mut game, Move::Stock);
    assert_eq!(cards_of(&game, PileRef::Stock), cards("AS 2S"));
    assert_eq!(cards_of(&game, PileRef::Waste), cards("3S"));
}

#[test]
fn an_empty_stock_and_waste_is_stock_empty() {
    let game = stock_table(PyramidRules::Relaxed, "", "");
    assert_rejected(&game, Move::Stock, MoveError::StockEmpty);
}

#[test]
fn recycling_returns_the_waste_in_the_original_order() {
    let mut game = stock_table(PyramidRules::Relaxed, "", "AS 2S");
    assert_accepted(&mut game, Move::Stock);
    assert_eq!(cards_of(&game, PileRef::Stock), cards("2S AS"));
    assert!(cards_of(&game, PileRef::Waste).is_empty());
    assert_accepted(&mut game, Move::Stock);
    assert_eq!(cards_of(&game, PileRef::Waste), cards("AS"));
}

#[test]
fn relaxed_allows_two_recycles_after_the_first_pass() {
    let mut game = stock_table(PyramidRules::Relaxed, "", "AS");
    for _ in 0..2 {
        assert_accepted(&mut game, Move::Stock);
        assert_accepted(&mut game, Move::Stock);
    }
    assert_rejected(&game, Move::Stock, MoveError::StockEmpty);
}

#[test]
fn classic_has_no_recycle() {
    let game = stock_table(PyramidRules::Classic, "", "AS");
    assert_rejected(&game, Move::Stock, MoveError::StockEmpty);
    let mut game = stock_table(PyramidRules::Classic, "2S", "AS");
    assert_accepted(&mut game, Move::Stock);
    assert_rejected(&game, Move::Stock, MoveError::StockEmpty);
}

#[test]
fn relaxed_is_won_when_the_pyramid_is_empty() {
    let game = cleared_table("AS");
    assert!(game.is_won());
    assert!(!base_table().is_won());
}

#[test]
fn classic_is_won_when_all_52_cards_are_on_the_foundation() {
    let mut game = stock_table(PyramidRules::Classic, "", "");
    game.foundation = crate::deck::standard();
    assert!(game.is_won());
    game.foundation.pop();
    assert!(!game.is_won());
}

#[test]
fn a_won_game_accepts_no_moves() {
    let game = cleared_table("KS");
    assert!(game.is_won());
    assert!(game.legal_moves().is_empty());
    assert_eq!(game.hint(), None);
    assert_rejected(&game, Move::Stock, MoveError::Unsupported);
    let king = remove(PileRef::Stock, None);
    assert_rejected(&game, king, MoveError::Unsupported);
}
