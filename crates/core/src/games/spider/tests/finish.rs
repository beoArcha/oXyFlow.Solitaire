use super::*;
use crate::game::SuitCount;

const SPADES_HIGH: &str = "KS QS JS 10S 9S 8S";
const SPADES_LOW: &str = "7S 6S 5S 4S 3S 2S AS";
const HEARTS_HIGH: &str = "KH QH JH 10H 9H 8H";
const HEARTS_LOW: &str = "7H 6H 5H 4H 3H 2H AH";

fn with_runs(complete: &[&str], columns: &[&str]) -> Spider {
    let mut game = table();
    for (foundation, run) in game.foundations.iter_mut().zip(complete) {
        *foundation = cards(run);
    }
    for (slot, codes) in game.columns.iter_mut().zip(columns) {
        *slot = column("", codes);
    }
    game
}

fn assert_wins(game: &Spider) -> Vec<Move> {
    let moves = game.forced_finish().expect("a forced win");
    assert_eq!(game.forced_finish(), Some(moves.clone()));
    let mut copy = game.clone();
    for &mv in &moves {
        assert_accepted(&mut copy, mv);
    }
    assert!(copy.is_won());
    moves
}

fn two_spade_runs() -> Spider {
    with_runs(
        &[SPADES_RUN; 6],
        &[SPADES_HIGH, SPADES_LOW, SPADES_HIGH, SPADES_LOW],
    )
}

#[test]
fn runs_that_continue_each_other_are_a_forced_win() {
    assert_eq!(assert_wins(&two_spade_runs()).len(), 2);
}

#[test]
fn a_run_split_in_three_is_joined_step_by_step() {
    let game = with_runs(
        &[SPADES_RUN; 7],
        &["KS QS JS", "10S 9S 8S 7S", "6S 5S 4S 3S 2S AS"],
    );
    assert_eq!(assert_wins(&game).len(), 2);
}

#[test]
fn two_suits_are_joined_within_their_own_suit() {
    let complete = [
        SPADES_RUN, SPADES_RUN, SPADES_RUN, HEARTS_RUN, HEARTS_RUN, HEARTS_RUN,
    ];
    let columns = [SPADES_HIGH, HEARTS_HIGH, HEARTS_LOW, SPADES_LOW];
    assert_eq!(assert_wins(&with_runs(&complete, &columns)).len(), 2);
}

#[test]
fn runs_of_different_suits_do_not_join() {
    let game = with_runs(&[SPADES_RUN; 6], &[SPADES_HIGH, HEARTS_LOW]);
    assert_eq!(game.forced_finish(), None);
}

#[test]
fn a_card_in_the_stock_is_not_a_forced_win() {
    let mut game = two_spade_runs();
    game.stock = cards("KS");
    assert_eq!(game.forced_finish(), None);
}

#[test]
fn a_face_down_card_is_not_a_forced_win() {
    let mut game = two_spade_runs();
    game.columns[3] = column("KS", SPADES_LOW);
    assert_eq!(game.forced_finish(), None);
}

#[test]
fn a_column_that_is_not_one_run_is_not_a_forced_win() {
    let mut game = two_spade_runs();
    game.columns[0] = column("", "KS QH 8S");
    assert_eq!(game.forced_finish(), None);
}

#[test]
fn a_won_game_has_nothing_to_finish() {
    assert_eq!(with_runs(&[SPADES_RUN; 8], &[]).forced_finish(), None);
}

#[test]
fn golden_deals_are_not_forced_wins() {
    for suits in [SuitCount::One, SuitCount::Two, SuitCount::Four] {
        assert_eq!(Spider::deal(suits, 42).forced_finish(), None, "{suits:?}");
    }
}
