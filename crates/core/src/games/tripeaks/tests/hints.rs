use super::*;

#[test]
fn the_hint_prefers_a_play_that_uncovers_a_card() {
    let mut game = empty_game("6D", "2C");
    put(&mut game, 2, 0, "KS", false);
    put(&mut game, 3, 0, "7H", true);
    put(&mut game, 3, 5, "5S", true);
    assert_eq!(game.hint(), Some(play(3, 0)));
}

#[test]
fn without_a_play_the_hint_is_the_stock_turn() {
    let mut game = empty_game("6D", "2C");
    put(&mut game, 3, 0, "9S", true);
    assert_eq!(game.hint(), Some(Move::Stock));
}

#[test]
fn without_a_play_and_a_stock_there_is_no_hint() {
    let mut game = empty_game("6D", "");
    put(&mut game, 3, 0, "9S", true);
    assert_eq!(game.hint(), None);
}

#[test]
fn the_hint_of_a_fresh_deal_is_a_legal_move() {
    for seed in 0..20 {
        let game = TriPeaks::deal(seed);
        let hint = game.hint().unwrap();
        assert!(game.legal_moves().contains(&hint), "seed {seed}: {hint:?}");
    }
}
