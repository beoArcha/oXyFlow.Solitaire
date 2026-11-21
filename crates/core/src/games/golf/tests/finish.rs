use super::*;

fn replay(mut game: Golf, moves: &[Move]) -> Golf {
    for &mv in moves {
        assert_eq!(game.apply(mv), Ok(()), "{mv:?}");
    }
    game
}

#[test]
fn a_chain_the_waste_top_can_play_out_is_a_forced_finish() {
    let mut game = table("7D", "KC");
    set_column(&mut game, 0, "3C 4H 5S 6S");
    let moves = game.forced_finish().unwrap();
    assert_eq!(moves, vec![play(0); 4]);
    assert!(replay(game, &moves).is_won());
}

#[test]
fn the_finish_may_switch_between_columns() {
    let mut game = table("7D", "");
    set_column(&mut game, 0, "3D 5H");
    set_column(&mut game, 1, "4C 6S");
    let moves = game.forced_finish().unwrap();
    assert_eq!(moves, vec![play(1), play(0), play(1), play(0)]);
    assert!(replay(game, &moves).is_won());
}

#[test]
fn a_finish_never_turns_the_stock() {
    let mut game = table("KD", "6C");
    set_column(&mut game, 0, "7S");
    assert_eq!(game.forced_finish(), None);
}

#[test]
fn a_stuck_position_has_no_finish() {
    let mut game = table("6D", "");
    set_column(&mut game, 0, "9S");
    assert_eq!(game.forced_finish(), None);
}

#[test]
fn a_won_game_has_no_finish() {
    assert_eq!(table("6D", "2C").forced_finish(), None);
}
