use super::*;

const NO_CELLS: [&str; 4] = ["", "", "", ""];
const NO_COLUMNS: [&str; 8] = ["", "", "", "", "", "", "", ""];

fn nearly_won(cells: [&str; 4], columns: [&str; 8]) -> FreeCell {
    let mut game = table();
    full_foundations(&mut game);
    let placed: Vec<Card> = cells
        .iter()
        .chain(&columns)
        .flat_map(|c| cards(c))
        .collect();
    for foundation in &mut game.foundations {
        foundation.retain(|card| !placed.iter().any(|other| other.id == card.id));
    }
    for (slot, codes) in game.cells.iter_mut().zip(cells) {
        *slot = cards(codes);
    }
    for (slot, codes) in game.columns.iter_mut().zip(columns) {
        *slot = cards(codes);
    }
    game
}

fn assert_wins(game: &FreeCell) -> Vec<Move> {
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
fn cells_and_column_tops_that_fit_the_foundations_are_a_forced_win() {
    let cells = ["KC", "", "", ""];
    let columns = ["KS QH", "KH QS", "KD", "", "", "", "", ""];
    let moves = assert_wins(&nearly_won(cells, columns));
    assert_eq!(moves.len(), 6);
}

#[test]
fn a_column_is_dug_through_from_the_top() {
    let columns = ["KS KH KC KD", "", "", "", "", "", "", ""];
    assert_eq!(assert_wins(&nearly_won(NO_CELLS, columns)).len(), 4);
}

#[test]
fn a_buried_card_that_blocks_the_closure_gives_none() {
    let columns = ["QH KS", "QS KH", "", "", "", "", "", ""];
    assert_eq!(nearly_won(NO_CELLS, columns).forced_finish(), None);
}

#[test]
fn a_card_whose_predecessor_is_missing_gives_none() {
    let mut columns = NO_COLUMNS;
    columns[0] = "KS";
    let mut game = nearly_won(NO_CELLS, columns);
    game.foundations[3].pop();
    assert_eq!(game.forced_finish(), None);
}

#[test]
fn a_won_game_has_nothing_to_finish() {
    assert_eq!(nearly_won(NO_CELLS, NO_COLUMNS).forced_finish(), None);
}

#[test]
fn deals_are_not_forced_wins() {
    for seed in 0..50 {
        assert_eq!(FreeCell::deal(seed).forced_finish(), None, "seed {seed}");
    }
}
