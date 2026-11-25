use super::*;

const TEN_CARDS: &str = "AS 2S 3S 4S 5S 6S 7S 8S 9S 10S";

fn assert_hint(game: &Spider, expected: Option<Move>) {
    assert_eq!(game.hint(), expected);
    if let Some(mv) = expected {
        assert!(game.legal_moves().contains(&mv), "{mv:?} is not legal");
    }
}

#[test]
fn extending_a_same_suit_run_comes_first() {
    let mut game = table();
    game.columns[0] = column("", "5S");
    game.columns[1] = column("", "6S");
    game.columns[2] = column("9C", "4D");
    game.columns[3] = column("", "5C");
    fill_empty_columns(&mut game);
    assert_hint(&game, Some(transfer(tab(0), tab(1), 1)));
}

#[test]
fn flipping_a_card_beats_emptying_a_column() {
    let mut game = table();
    game.columns[0] = column("", "5S");
    game.columns[1] = column("", "6D");
    game.columns[2] = column("9C", "4D");
    game.columns[3] = column("", "5C");
    fill_empty_columns(&mut game);
    assert_hint(&game, Some(transfer(tab(2), tab(0), 1)));
}

#[test]
fn emptying_a_column_beats_building_on_another_suit() {
    let mut game = table();
    game.columns[0] = column("", "5S");
    game.columns[1] = column("", "6D");
    game.columns[2] = column("", "9C 4D");
    fill_empty_columns(&mut game);
    assert_hint(&game, Some(transfer(tab(0), tab(1), 1)));
}

#[test]
fn building_on_another_suit_beats_the_stock() {
    let mut game = table();
    game.columns[0] = column("", "5S");
    game.columns[1] = column("", "9C 4D");
    game.stock = cards(TEN_CARDS);
    fill_empty_columns(&mut game);
    assert_hint(&game, Some(transfer(tab(1), tab(0), 1)));
}

#[test]
fn the_stock_is_the_last_resort() {
    let mut game = table();
    game.stock = cards(TEN_CARDS);
    fill_empty_columns(&mut game);
    assert_hint(&game, Some(Move::Stock));
}

#[test]
fn no_hint_without_moves() {
    assert_hint(&table(), None);
    let mut game = table();
    game.stock = cards("AS");
    game.columns[0] = column("", "5S");
    assert!(!game.legal_moves().contains(&Move::Stock));
    assert_hint(&game, None);
}

#[test]
fn an_empty_column_is_filled_so_that_the_stock_can_be_dealt() {
    let mut game = table();
    game.stock = cards(TEN_CARDS);
    game.columns[0] = column("", "9C 4D");
    fill_empty_columns(&mut game);
    game.columns[1] = column("", "");
    assert!(!game.legal_moves().contains(&Move::Stock));
    assert_hint(&game, Some(transfer(tab(0), tab(1), 1)));
}

#[test]
fn a_linked_card_is_not_moved_into_an_empty_column() {
    let mut game = table();
    game.stock = cards(TEN_CARDS);
    game.columns[0] = column("", "5C 4D");
    fill_empty_columns(&mut game);
    game.columns[1] = column("", "");
    assert_hint(&game, None);
}

#[test]
fn a_sideways_move_is_not_suggested() {
    let mut game = table();
    game.columns[0] = column("", "7H 6S");
    game.columns[1] = column("", "7D");
    fill_empty_columns(&mut game);
    assert!(game.legal_moves().contains(&transfer(tab(0), tab(1), 1)));
    assert_hint(&game, None);
}

#[test]
fn never_suggests_moving_a_whole_column_to_an_empty_column() {
    let mut game = table();
    game.columns[0] = column("", "6S 5S");
    assert!(game.legal_moves().contains(&transfer(tab(0), tab(1), 2)));
    assert_hint(&game, None);
}

#[test]
fn a_column_with_face_down_cards_may_move_to_an_empty_column_to_flip() {
    let mut game = table();
    game.columns[0] = column("9C", "5S");
    assert_hint(&game, Some(transfer(tab(0), tab(1), 1)));
}

#[test]
fn never_suggests_moving_part_of_a_run() {
    let mut game = table();
    game.columns[0] = column("9C", "8S 7S");
    game.columns[1] = column("", "8H");
    fill_empty_columns(&mut game);
    assert!(game.legal_moves().contains(&transfer(tab(0), tab(1), 1)));
    assert_hint(&game, None);
}
