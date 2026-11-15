use super::*;

const WASTE: PileRef = PileRef::Waste;

fn assert_hint(game: &Klondike, expected: Option<Move>) {
    assert_eq!(game.hint(), expected);
    if let Some(mv) = expected {
        assert!(game.legal_moves().contains(&mv), "{mv:?} is not legal");
    }
}

#[test]
fn a_foundation_move_comes_first() {
    let mut game = table();
    game.stock = cards("9S");
    game.waste = cards("6D");
    game.foundations[0] = cards("AS");
    game.columns[0] = column("9C", "2S");
    game.columns[1] = column("", "3H");
    game.columns[2] = column("", "7S");
    assert_hint(&game, Some(transfer(tab(0), found(0), 1)));
}

#[test]
fn a_waste_card_may_go_to_a_foundation() {
    let mut game = table();
    game.waste = cards("AD");
    game.columns[0] = column("", "7S");
    game.columns[1] = column("", "6D");
    assert_hint(&game, Some(transfer(WASTE, found(0), 1)));
}

#[test]
fn a_run_that_flips_a_card_beats_the_waste() {
    let mut game = table();
    game.stock = cards("9S");
    game.waste = cards("6D");
    game.columns[0] = column("9C", "6H");
    game.columns[1] = column("", "7S");
    assert_hint(&game, Some(transfer(tab(0), tab(1), 1)));
}

#[test]
fn a_whole_run_moves_as_one() {
    let mut game = table();
    game.columns[0] = column("9C", "7H 6S");
    game.columns[1] = column("", "8S");
    assert_hint(&game, Some(transfer(tab(0), tab(1), 2)));
}

#[test]
fn a_move_that_empties_a_column_beats_the_waste() {
    let mut game = table();
    game.waste = cards("6D");
    game.columns[0] = column("", "6H");
    game.columns[1] = column("", "7S");
    assert_hint(&game, Some(transfer(tab(0), tab(1), 1)));
}

#[test]
fn the_waste_to_the_tableau_beats_a_king_and_the_stock() {
    let mut game = table();
    game.stock = cards("9S");
    game.waste = cards("6D");
    game.columns[0] = column("", "7S");
    game.columns[1] = column("9C", "KH");
    assert_hint(&game, Some(transfer(WASTE, tab(0), 1)));
}

#[test]
fn a_waste_king_may_open_an_empty_column() {
    let mut game = table();
    game.waste = cards("KD");
    assert_hint(&game, Some(transfer(WASTE, tab(0), 1)));
}

#[test]
fn a_king_that_reveals_a_card_beats_the_stock() {
    let mut game = table();
    game.stock = cards("9S");
    game.columns[0] = column("9C", "KH QS");
    assert_hint(&game, Some(transfer(tab(0), tab(1), 2)));
}

#[test]
fn the_stock_is_the_last_resort_and_recycling_counts() {
    let mut game = table();
    game.stock = cards("9S");
    assert_hint(&game, Some(Move::Stock));
    game.stock.clear();
    game.waste = cards("9S");
    assert_hint(&game, Some(Move::Stock));
}

#[test]
fn no_hint_without_moves() {
    assert_hint(&table(), None);
}

#[test]
fn never_suggests_a_move_out_of_a_foundation() {
    let mut game = table();
    game.foundations[0] = cards("AS 2S");
    game.columns[0] = column("", "3H");
    assert!(game.legal_moves().contains(&transfer(found(0), tab(0), 1)));
    assert_hint(&game, None);
    game.stock = cards("9S");
    assert_hint(&game, Some(Move::Stock));
}

#[test]
fn never_suggests_a_king_from_the_bottom_of_a_column_to_an_empty_column() {
    let mut game = table();
    game.columns[0] = column("", "KH");
    assert!(game.legal_moves().contains(&transfer(tab(0), tab(1), 1)));
    assert_hint(&game, None);
    game.columns[0] = column("", "KH QS");
    assert!(game.legal_moves().contains(&transfer(tab(0), tab(1), 2)));
    assert_hint(&game, None);
}

#[test]
fn never_suggests_moving_part_of_a_run() {
    let mut game = table();
    game.columns[0] = column("9C", "7H 6S");
    game.columns[1] = column("", "7D");
    assert!(game.legal_moves().contains(&transfer(tab(0), tab(1), 1)));
    assert_hint(&game, None);
}
