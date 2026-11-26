use super::*;

fn replay(mut game: TriPeaks, moves: &[Move]) -> TriPeaks {
    for &mv in moves {
        assert_eq!(game.apply(mv), Ok(()), "{mv:?}");
    }
    game
}

#[test]
fn a_chain_the_waste_top_can_clear_is_a_forced_finish() {
    let mut game = empty_game("7D", "KC");
    put(&mut game, 2, 0, "4S", false);
    put(&mut game, 3, 0, "6S", true);
    put(&mut game, 3, 1, "5H", true);
    let moves = game.forced_finish().unwrap();
    assert_eq!(moves, vec![play(3, 0), play(3, 1), play(2, 0)]);
    assert!(replay(game, &moves).is_won());
}

#[test]
fn a_finish_never_turns_the_stock() {
    let mut game = empty_game("KD", "6C");
    put(&mut game, 3, 0, "7S", true);
    assert_eq!(game.forced_finish(), None);
}

#[test]
fn a_stuck_position_has_no_finish() {
    let mut game = empty_game("6D", "");
    put(&mut game, 3, 0, "9S", true);
    assert_eq!(game.forced_finish(), None);
}

#[test]
fn a_won_game_has_no_finish() {
    assert_eq!(empty_game("6D", "2C").forced_finish(), None);
}
