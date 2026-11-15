use super::*;
use crate::moves::MoveError::NotEnoughSpace;

const LONG_RUN: &str = "KS QH JS 10H 9S 8H 7S 6H 5S 4H 3S 2H AS";
const FILLER: [&str; 6] = ["2C", "2D", "2H", "3C", "3D", "3H"];

/// Column 0 holds a 13-card run, column 1 is the empty target, the listed columns hold one filler
/// card each, the rest stay empty. Cells stay empty unless a test fills them.
fn long_run_table(filled: &[u8]) -> FreeCell {
    let mut game = table();
    game.columns[0] = cards(LONG_RUN);
    for &index in filled {
        game.columns[usize::from(index)] = cards(FILLER[usize::from(index) - 2]);
    }
    game
}

fn full_cells(game: &mut FreeCell) {
    game.cells = [cards("6D"), cards("7D"), cards("8D"), cards("9D")];
}

#[test]
fn four_free_cells_and_no_other_empty_column_allow_five_cards() {
    let mut game = long_run_table(&[2, 3, 4, 5, 6, 7]);
    assert_accepted(&mut game, transfer(tab(0), tab(1), 5));
    assert_eq!(cards_of(&game, tab(1)), cards("5S 4H 3S 2H AS"));
    let mut too_many = long_run_table(&[2, 3, 4, 5, 6, 7]);
    assert_rejected(&too_many, transfer(tab(0), tab(1), 6), NotEnoughSpace);
    too_many.cells[0] = cards("9D");
    assert_rejected(&too_many, transfer(tab(0), tab(1), 5), NotEnoughSpace);
}

#[test]
fn full_cells_and_two_other_empty_columns_allow_four_cards() {
    let mut game = long_run_table(&[4, 5, 6, 7]);
    full_cells(&mut game);
    assert_accepted(&mut game, transfer(tab(0), tab(1), 4));
    assert_eq!(cards_of(&game, tab(1)), cards("4H 3S 2H AS"));
    let mut too_many = long_run_table(&[4, 5, 6, 7]);
    full_cells(&mut too_many);
    assert_rejected(&too_many, transfer(tab(0), tab(1), 5), NotEnoughSpace);
}

#[test]
fn the_target_column_does_not_count_as_empty() {
    let mut game = long_run_table(&[2, 3, 4, 5, 6, 7]);
    full_cells(&mut game);
    assert_rejected(&game, transfer(tab(0), tab(1), 2), NotEnoughSpace);
    assert_accepted(&mut game, transfer(tab(0), tab(1), 1));
}

#[test]
fn full_cells_and_one_other_empty_column_allow_two_cards() {
    let mut game = long_run_table(&[3, 4, 5, 6, 7]);
    full_cells(&mut game);
    assert_accepted(&mut game, transfer(tab(0), tab(1), 2));
    assert_eq!(cards_of(&game, tab(1)), cards("2H AS"));
    let mut too_many = long_run_table(&[3, 4, 5, 6, 7]);
    full_cells(&mut too_many);
    assert_rejected(&too_many, transfer(tab(0), tab(1), 3), NotEnoughSpace);
}

#[test]
fn three_free_cells_and_one_other_empty_column_allow_eight_cards() {
    let mut game = long_run_table(&[3, 4, 5, 6, 7]);
    game.cells[0] = cards("AC");
    assert_accepted(&mut game, transfer(tab(0), tab(1), 8));
    assert_eq!(cards_of(&game, tab(1)), cards("8H 7S 6H 5S 4H 3S 2H AS"));
    let mut too_many = long_run_table(&[3, 4, 5, 6, 7]);
    too_many.cells[0] = cards("AC");
    assert_rejected(&too_many, transfer(tab(0), tab(1), 9), NotEnoughSpace);
}

#[test]
fn a_whole_thirteen_card_run_fits_with_three_other_empty_columns_and_free_cells() {
    let mut game = long_run_table(&[5, 6, 7]);
    assert_accepted(&mut game, transfer(tab(0), tab(1), 13));
    assert_eq!(cards_of(&game, tab(1)).len(), 13);
    assert!(cards_of(&game, tab(0)).is_empty());
}
