use super::*;
use crate::game::{Game, StockStatus, SuitCount};

fn deals_left(game: &Spider) -> Option<u32> {
    game.stock_status().and_then(|status| status.deals_left)
}

#[test]
fn a_fresh_deal_has_five_rows_left_and_no_recycle() {
    for suits in [SuitCount::One, SuitCount::Two, SuitCount::Four] {
        let status = Spider::deal(suits, 7).stock_status();
        let expected = StockStatus {
            recycles: 0,
            recycles_left: Some(0),
            deals_left: Some(5),
        };
        assert_eq!(status, Some(expected), "{suits:?}");
    }
}

#[test]
fn each_row_deal_takes_one_row_until_none_is_left() {
    let mut game = Spider::deal(SuitCount::Four, 7);
    for left in (0..5).rev() {
        assert_eq!(game.apply(Move::Stock), Ok(()));
        assert_eq!(deals_left(&game), Some(left));
    }
    assert_eq!(game.apply(Move::Stock), Err(MoveError::StockEmpty));
    assert_eq!(deals_left(&game), Some(0));
}

#[test]
fn a_rejected_row_deal_is_not_counted() {
    let mut game = Spider::deal(SuitCount::One, 7);
    for _ in 0..5 {
        assert_eq!(game.apply(Move::Stock), Ok(()));
    }
    let before = game.stock_status();
    assert_eq!(game.apply(Move::Stock), Err(MoveError::StockEmpty));
    assert_eq!(game.stock_status(), before);
}

#[test]
fn deal_with_empty_column_is_refused_by_default_and_accepted_when_enabled() {
    let mut blocked = table();
    blocked.stock = cards("AS 2S 3S 4S 5S 6S 7S 8S 9S 10S");
    blocked.columns[0] = column("", "KC");
    assert_rejected(&blocked, Move::Stock, MoveError::EmptyColumnBlocksDeal);

    let options = crate::game::GameOptions {
        spider_deal_empty: true,
        ..Default::default()
    };
    let mut allowed = Spider::empty_with(options);
    allowed.stock = cards("AS 2S 3S 4S 5S 6S 7S 8S 9S 10S");
    allowed.columns[0] = column("", "KC");
    assert_accepted(&mut allowed, Move::Stock);
    assert!(allowed.stock.is_empty());
    assert_eq!(cards_of(&allowed, tab(0)), cards("KC 10S"));
    assert_eq!(cards_of(&allowed, tab(1)), cards("9S"));
}
