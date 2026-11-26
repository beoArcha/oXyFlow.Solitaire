use super::*;
use crate::game::StockStatus;

fn status(game: &Klondike) -> StockStatus {
    game.stock_status().expect("Klondike has a stock")
}

fn draw_until_empty(game: &mut Klondike) {
    while !game.stock.is_empty() {
        assert_accepted(game, Move::Stock);
    }
}

#[test]
fn a_fresh_deal_has_no_recycle_and_no_limit() {
    for draw in [DrawCount::One, DrawCount::Three] {
        let status = Klondike::deal(draw, 42).stock_status();
        let expected = StockStatus {
            recycles: 0,
            recycles_left: None,
            deals_left: None,
        };
        assert_eq!(status, Some(expected), "{draw:?}");
    }
}

#[test]
fn drawing_through_the_stock_is_not_a_recycle() {
    let mut game = Klondike::empty(DrawCount::Three);
    game.stock = cards("AS 2S 3S 4S 5S");
    draw_until_empty(&mut game);
    assert!(game.stock.is_empty());
    assert_eq!(status(&game).recycles, 0);
}

#[test]
fn the_recycle_that_turns_the_waste_back_is_counted() {
    let mut game = table();
    game.stock = cards("AS 2S");
    draw_until_empty(&mut game);
    assert_accepted(&mut game, Move::Stock);
    assert_eq!(status(&game).recycles, 1);
    draw_until_empty(&mut game);
    assert_eq!(status(&game).recycles, 1);
}

#[test]
fn every_recycle_is_counted_and_the_limit_stays_unlimited() {
    let mut game = Klondike::empty(DrawCount::Three);
    game.stock = cards("AS 2S 3S 4S 5S");
    for recycles in 1..=4 {
        draw_until_empty(&mut game);
        assert_accepted(&mut game, Move::Stock);
        assert_eq!(status(&game).recycles, recycles);
    }
    assert_eq!(status(&game).recycles_left, None);
}

#[test]
fn a_rejected_recycle_is_not_counted() {
    let game = table();
    assert_rejected(&game, Move::Stock, MoveError::StockEmpty);
    assert_eq!(status(&game).recycles, 0);
}

#[test]
fn three_passes_allows_two_recycles_then_refuses_with_passes_exhausted() {
    let options = crate::game::GameOptions {
        klondike_passes: crate::game::StockPasses::Three,
        ..Default::default()
    };
    let mut game = Klondike::empty_with(DrawCount::One, options);
    game.stock = cards("AS 2S");
    assert_eq!(status(&game).recycles_left, Some(2));
    draw_until_empty(&mut game);
    assert!(game.legal_moves().contains(&Move::Stock));
    assert_accepted(&mut game, Move::Stock);
    assert_eq!(status(&game).recycles, 1);
    assert_eq!(status(&game).recycles_left, Some(1));
    draw_until_empty(&mut game);
    assert_accepted(&mut game, Move::Stock);
    assert_eq!(status(&game).recycles, 2);
    assert_eq!(status(&game).recycles_left, Some(0));
    draw_until_empty(&mut game);
    assert!(!game.legal_moves().contains(&Move::Stock));
    assert_rejected(&game, Move::Stock, MoveError::PassesExhausted);
    assert_eq!(status(&game).recycles, 2);
}

#[test]
fn one_pass_allows_no_recycles_and_refuses_once_stock_is_empty() {
    let options = crate::game::GameOptions {
        klondike_passes: crate::game::StockPasses::One,
        ..Default::default()
    };
    let mut game = Klondike::empty_with(DrawCount::Three, options);
    game.stock = cards("AS 2S 3S 4S");
    assert_eq!(status(&game).recycles_left, Some(0));
    draw_until_empty(&mut game);
    assert!(!game.legal_moves().contains(&Move::Stock));
    assert_rejected(&game, Move::Stock, MoveError::PassesExhausted);
    assert_eq!(status(&game).recycles, 0);
}
