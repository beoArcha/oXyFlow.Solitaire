use super::*;
use crate::moves::MoveError::{InvalidSequence, StockEmpty, TargetRejects};

const ALMOST_SPADES: &str = "KS QS JS 10S 9S 8S 7S 6S 5S 4S 3S 2S";
const ALMOST_HEARTS: &str = "KH QH JH 10H 9H 8H 7H 6H 5H 4H 3H 2H";
const TEN_CARDS: &str = "AS 2S 3S 4S 5S 6S 7S 8S 9S 10S";

#[test]
fn a_completed_run_goes_to_the_first_free_foundation_and_the_card_beneath_flips() {
    let mut game = table();
    game.columns[0] = column("9C", ALMOST_SPADES);
    game.columns[1] = column("", "AS");
    assert_accepted(&mut game, transfer(tab(1), tab(0), 1));
    assert_eq!(cards_of(&game, found(0)), cards(SPADES_RUN));
    assert_eq!(game.columns[0].down, cards(""));
    assert_eq!(game.columns[0].up, cards("9C"));
    assert!(game.columns[1].up.is_empty());
}

#[test]
fn the_first_free_foundation_is_used() {
    let mut game = table();
    game.foundations[0] = cards(HEARTS_RUN);
    game.columns[0] = column("", ALMOST_SPADES);
    game.columns[1] = column("", "AS");
    assert_accepted(&mut game, transfer(tab(1), tab(0), 1));
    assert_eq!(cards_of(&game, found(0)), cards(HEARTS_RUN));
    assert_eq!(cards_of(&game, found(1)), cards(SPADES_RUN));
    assert!(game.columns[0].up.is_empty());
}

#[test]
fn a_collected_run_leaves_the_cards_under_it_alone() {
    let mut game = table();
    game.columns[0] = column("9C 4D", "6H KS QS JS 10S 9S 8S 7S 6S 5S 4S 3S 2S");
    game.columns[1] = column("", "AS");
    assert_accepted(&mut game, transfer(tab(1), tab(0), 1));
    assert_eq!(game.columns[0].up, cards("6H"));
    assert_eq!(game.columns[0].down, cards("9C 4D"));
}

#[test]
fn a_run_of_mixed_suits_is_not_collected() {
    let mut game = table();
    game.columns[0] = column("", "KS QS JS 10S 9S 8S 7S 6S 5S 4S 3S 2H");
    game.columns[1] = column("", "AS");
    assert_accepted(&mut game, transfer(tab(1), tab(0), 1));
    assert_eq!(game.columns[0].up.len(), 13);
    assert!(game.foundations.iter().all(Vec::is_empty));
}

#[test]
fn a_run_must_start_with_a_king() {
    let mut game = table();
    game.columns[0] = column("", "QS JS 10S 9S 8S 7S 6S 5S 4S 3S 2S");
    game.columns[1] = column("", "AS");
    assert_accepted(&mut game, transfer(tab(1), tab(0), 1));
    assert_eq!(game.columns[0].up.len(), 12);
    assert!(game.foundations.iter().all(Vec::is_empty));
}

#[test]
fn the_stock_deals_one_face_up_card_onto_every_column() {
    let mut game = table();
    for (index, column) in game.columns.iter_mut().enumerate() {
        column.up = cards("KC");
        column.down = if index == 3 { cards("4D") } else { Vec::new() };
    }
    game.stock = cards(TEN_CARDS);
    assert_accepted(&mut game, Move::Stock);
    assert!(game.stock.is_empty());
    assert_eq!(game.columns[0].up, cards("KC 10S"));
    assert_eq!(game.columns[9].up, cards("KC AS"));
    assert_eq!(game.columns[3].down, cards("4D"));
    assert_eq!(game.columns[3].up, cards("KC 7S"));
}

#[test]
fn a_dealt_card_may_break_the_order_of_a_column() {
    let mut game = table();
    game.columns[0] = column("", "9H 8H");
    fill_empty_columns(&mut game);
    game.stock = cards("AC 2C 3C 5C 6C 7C 8C 9C 4C 3S");
    assert_accepted(&mut game, Move::Stock);
    assert_eq!(game.columns[0].up, cards("9H 8H 3S"));
    assert!(game.legal_moves().contains(&transfer(tab(0), tab(1), 1)));
    assert_rejected(&game, transfer(tab(0), tab(1), 2), InvalidSequence);
}

#[test]
fn a_stock_deal_collects_a_completed_run_and_flips() {
    let mut game = table();
    game.columns[0] = column("9C", ALMOST_SPADES);
    fill_empty_columns(&mut game);
    game.stock = cards("2C 3C 4C 5C 6C 7C 8C 9C 10C AS");
    assert_accepted(&mut game, Move::Stock);
    assert_eq!(cards_of(&game, found(0)), cards(SPADES_RUN));
    assert_eq!(game.columns[0].up, cards("9C"));
    assert!(game.columns[0].down.is_empty());
}

#[test]
fn a_stock_deal_can_complete_several_columns_at_once() {
    let mut game = table();
    game.columns[0] = column("9C", ALMOST_SPADES);
    game.columns[1] = column("4D", ALMOST_HEARTS);
    fill_empty_columns(&mut game);
    game.stock = cards("2C 3C 4C 5C 6C 7C 8C 9C AH AS");
    assert_accepted(&mut game, Move::Stock);
    assert_eq!(cards_of(&game, found(0)), cards(SPADES_RUN));
    assert_eq!(cards_of(&game, found(1)), cards(HEARTS_RUN));
    assert_eq!(game.columns[0].up, cards("9C"));
    assert_eq!(game.columns[1].up, cards("4D"));
}

#[test]
fn the_stock_is_refused_when_it_is_empty() {
    let mut game = table();
    fill_empty_columns(&mut game);
    assert_rejected(&game, Move::Stock, StockEmpty);
}

#[test]
fn the_stock_is_refused_while_a_column_is_empty() {
    let mut game = table();
    fill_empty_columns(&mut game);
    game.columns[4] = Column::default();
    game.stock = cards(TEN_CARDS);
    assert_rejected(&game, Move::Stock, TargetRejects);
    assert_eq!(game.stock.len(), 10);
}
