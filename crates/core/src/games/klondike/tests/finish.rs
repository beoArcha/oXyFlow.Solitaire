use super::*;

const COLUMNS: usize = 7;
const KINGS_AND_QUEENS: [&str; COLUMNS] = ["KH QS", "KS QH", "", "", "", "", ""];

fn open_table(columns: [&str; COLUMNS]) -> Klondike {
    let mut game = table();
    full_foundations(&mut game);
    for (index, codes) in columns.into_iter().enumerate() {
        let up = cards(codes);
        for foundation in &mut game.foundations {
            foundation.retain(|card| !up.iter().any(|placed| placed.id == card.id));
        }
        game.columns[index] = column("", codes);
    }
    game
}

fn assert_wins(game: &Klondike) -> Vec<Move> {
    let moves = game.forced_finish().expect("a forced win");
    assert_eq!(game.forced_finish(), Some(moves.clone()));
    let mut copy = game.clone();
    for &mv in &moves {
        assert_accepted(&mut copy, mv);
    }
    assert!(copy.is_won());
    moves
}

#[test]
fn an_open_table_without_stock_is_a_forced_win() {
    let moves = assert_wins(&open_table(KINGS_AND_QUEENS));
    assert_eq!(moves.len(), 4);
}

#[test]
fn interleaved_runs_are_finished_in_a_legal_order() {
    let game = open_table([
        "KH QS JH 10S",
        "KS QH JS 10H",
        "KD QC JD 10C",
        "KC QD JC 10D",
        "",
        "",
        "",
    ]);
    assert_eq!(assert_wins(&game).len(), 16);
}

#[test]
fn a_card_left_in_the_stock_is_not_a_forced_win() {
    let mut game = open_table(KINGS_AND_QUEENS);
    game.stock = cards("KC");
    game.foundations[0].pop();
    assert_eq!(game.forced_finish(), None);
}

#[test]
fn a_card_left_in_the_waste_is_not_a_forced_win() {
    let mut game = open_table(KINGS_AND_QUEENS);
    game.waste = cards("KC");
    game.foundations[0].pop();
    assert_eq!(game.forced_finish(), None);
}

#[test]
fn a_face_down_card_is_not_a_forced_win() {
    let mut game = open_table(KINGS_AND_QUEENS);
    game.columns[2] = column("KC", "QD");
    game.foundations[0].pop();
    game.foundations[1].pop();
    assert_eq!(game.forced_finish(), None);
}

#[test]
fn a_won_game_has_nothing_to_finish() {
    let mut game = table();
    full_foundations(&mut game);
    assert_eq!(game.forced_finish(), None);
}

#[test]
fn golden_deals_are_not_forced_wins() {
    for draw in [DrawCount::One, DrawCount::Three] {
        assert_eq!(Klondike::deal(draw, 42).forced_finish(), None, "{draw:?}");
    }
}

#[test]
fn deals_are_not_forced_wins_for_any_seed() {
    for seed in 0..50 {
        assert_eq!(Klondike::deal(DrawCount::One, seed).forced_finish(), None);
    }
}
