use super::*;

fn assert_hint(game: &FreeCell, expected: Option<Move>) {
    assert_eq!(game.hint(), expected);
    if let Some(mv) = expected {
        assert!(game.legal_moves().contains(&mv), "{mv:?} is not legal");
    }
}

#[test]
fn a_column_card_to_a_foundation_comes_first() {
    let mut game = table();
    game.columns[0] = cards("7C AS");
    game.columns[1] = cards("8D");
    assert_hint(&game, Some(transfer(tab(0), found(0), 1)));
}

#[test]
fn a_cell_card_to_a_foundation_beats_a_column_move() {
    let mut game = table();
    game.cells[0] = cards("AH");
    game.columns[0] = cards("7C");
    game.columns[1] = cards("8D");
    assert_hint(&game, Some(transfer(cell(0), found(0), 1)));
}

#[test]
fn a_column_move_extending_a_run_beats_a_cell_to_column() {
    let mut game = table();
    game.cells[0] = cards("6D");
    game.columns[0] = cards("7S");
    game.columns[1] = cards("8H");
    assert_hint(&game, Some(transfer(tab(0), tab(1), 1)));
}

#[test]
fn a_cell_card_to_a_column_beats_a_column_to_an_empty_column() {
    let mut game = table();
    game.cells[0] = cards("6D");
    game.columns[0] = cards("7S");
    game.columns[2] = cards("KH");
    assert_hint(&game, Some(transfer(cell(0), tab(0), 1)));
}

#[test]
fn a_run_may_go_to_an_empty_column_before_a_cell() {
    let mut game = table();
    game.columns[0] = cards("KS 2H");
    assert_hint(&game, Some(transfer(tab(0), tab(1), 1)));
}

#[test]
fn a_card_resting_on_its_link_is_not_moved_aside() {
    let mut game = table();
    game.columns[0] = cards("KS QH");
    assert!(game.legal_moves().contains(&transfer(tab(0), tab(1), 1)));
    assert!(game.legal_moves().contains(&transfer(tab(0), cell(0), 1)));
    assert_hint(&game, None);
}

#[test]
fn following_the_hints_never_moves_a_card_back_and_forth() {
    let mut game = table();
    game.columns[0] = cards("9C 8H");
    game.columns[1] = cards("9S");
    for _ in 0..4 {
        let Some(mv) = game.hint() else { break };
        assert_eq!(game.apply(mv), Ok(()));
    }
    assert_eq!(game.columns[0], cards("9C 8H"));
    assert_eq!(game.columns[1], cards("9S"));
}

#[test]
fn a_column_card_goes_to_a_cell_last() {
    let mut game = table();
    game.columns[0] = cards("7S");
    game.columns[1] = cards("KH");
    assert_hint(&game, Some(transfer(tab(0), cell(0), 1)));
}

#[test]
fn a_lone_card_is_not_moved_to_an_empty_column_while_a_cell_is_free() {
    let mut game = table();
    game.columns[0] = cards("7S");
    assert!(game.legal_moves().contains(&transfer(tab(0), tab(1), 1)));
    assert_hint(&game, Some(transfer(tab(0), cell(0), 1)));
}

#[test]
fn no_cards_means_no_hint() {
    let game = table();
    assert_eq!(game.legal_moves(), Vec::new());
    assert_hint(&game, None);
}

#[test]
fn a_whole_run_is_never_hinted_to_an_empty_column() {
    let mut game = table();
    game.columns[0] = cards("8S 7H");
    assert!(game.legal_moves().contains(&transfer(tab(0), tab(1), 2)));
    assert_hint(&game, None);
}
