use super::*;
use crate::game::StockStatus;

const ROWS: [&str; 7] = [
    ".",
    ". .",
    ". . .",
    ". . . .",
    ". . . . .",
    ". . . . . .",
    ". . . . . . 9C",
];

fn status(game: &Pyramid) -> StockStatus {
    game.stock_status().expect("Pyramid has a stock")
}

fn stock_status(recycles: u32, recycles_left: u32) -> StockStatus {
    StockStatus {
        recycles,
        recycles_left: Some(recycles_left),
        deals_left: None,
    }
}

fn draw_until_empty(game: &mut Pyramid) {
    while !game.stock.is_empty() {
        assert_accepted(game, Move::Stock);
    }
}

#[test]
fn a_fresh_deal_has_no_recycle_and_the_rule_set_limit() {
    let relaxed = Pyramid::deal(PyramidRules::Relaxed, 7).stock_status();
    assert_eq!(relaxed, Some(stock_status(0, 2)));
    let classic = Pyramid::deal(PyramidRules::Classic, 7).stock_status();
    assert_eq!(classic, Some(stock_status(0, 0)));
}

#[test]
fn drawing_through_the_stock_is_not_a_recycle() {
    let mut game = table(PyramidRules::Relaxed, ROWS, "AS 2S", "");
    draw_until_empty(&mut game);
    assert_eq!(status(&game), stock_status(0, 2));
}

#[test]
fn relaxed_counts_its_two_recycles_then_refuses_the_third() {
    let mut game = table(PyramidRules::Relaxed, ROWS, "AS 2S", "");
    for recycles in 1..=2 {
        draw_until_empty(&mut game);
        assert_accepted(&mut game, Move::Stock);
        assert_eq!(status(&game), stock_status(recycles, 2 - recycles));
    }
    draw_until_empty(&mut game);
    assert_rejected(&game, Move::Stock, MoveError::StockEmpty);
    assert_eq!(status(&game), stock_status(2, 0));
}

#[test]
fn classic_has_no_recycle_and_refuses_it_at_once() {
    let mut game = table(PyramidRules::Classic, ROWS, "AS 2S", "");
    draw_until_empty(&mut game);
    assert_rejected(&game, Move::Stock, MoveError::StockEmpty);
    assert_eq!(status(&game), stock_status(0, 0));
}

#[test]
fn a_rejected_recycle_with_an_empty_waste_is_not_counted() {
    let game = table(PyramidRules::Relaxed, ROWS, "", "");
    assert_rejected(&game, Move::Stock, MoveError::StockEmpty);
    assert_eq!(status(&game), stock_status(0, 2));
}
