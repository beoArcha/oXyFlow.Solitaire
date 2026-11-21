use super::*;

#[test]
fn the_hint_is_the_play_from_the_tallest_column() {
    let mut game = table("6D", "2C");
    set_column(&mut game, 0, "9S 7H");
    set_column(&mut game, 1, "KS 9D 8C 5H");
    set_column(&mut game, 2, "7S");
    assert_eq!(game.hint(), Some(play(1)));
}

#[test]
fn equally_tall_columns_give_the_lowest_one() {
    let mut game = table("6D", "2C");
    set_column(&mut game, 2, "7S");
    set_column(&mut game, 4, "5S");
    assert_eq!(game.hint(), Some(play(2)));
}

#[test]
fn without_a_play_the_hint_is_the_stock_turn() {
    let mut game = table("6D", "2C");
    set_column(&mut game, 0, "9S");
    assert_eq!(game.hint(), Some(Move::Stock));
}

#[test]
fn without_a_play_and_a_stock_there_is_no_hint() {
    let mut game = table("6D", "");
    set_column(&mut game, 0, "9S");
    assert_eq!(game.hint(), None);
}

#[test]
fn the_hint_of_a_fresh_deal_is_a_legal_move() {
    for seed in 0..20 {
        let game = Golf::deal(seed);
        let hint = game.hint().unwrap();
        assert!(game.legal_moves().contains(&hint), "seed {seed}: {hint:?}");
    }
}
