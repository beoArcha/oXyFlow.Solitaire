use super::*;

#[test]
fn a_foundation_move_comes_first() {
    let mut game = table();
    game.columns[0] = column("", "AS");
    game.columns[1] = column("9D", "5H");
    game.columns[2] = column("", "6C");
    assert_eq!(game.hint(), Some(transfer(tab(0), found(0), 1)));
}

#[test]
fn then_a_group_that_uncovers_a_face_down_card() {
    let mut game = table();
    game.columns[0] = column("", "8S 4D 6C");
    game.columns[1] = column("9D", "5H 3C");
    game.columns[2] = column("", "7H");
    assert_eq!(game.hint(), Some(transfer(tab(1), tab(0), 2)));
}

#[test]
fn then_a_group_that_leaves_a_card_it_did_not_fit_on() {
    let mut game = table();
    game.columns[0] = column("", "6C 8S 4D 2H");
    game.columns[1] = column("", "5S");
    assert_eq!(game.hint(), Some(transfer(tab(0), tab(1), 2)));
}

#[test]
fn a_group_that_already_fits_where_it_lies_is_not_suggested() {
    let mut game = table();
    game.columns[0] = column("", "5S 4D");
    game.columns[1] = column("", "5C");
    assert!(game.legal_moves().contains(&transfer(tab(0), tab(1), 1)));
    assert_eq!(game.hint(), None);
}

#[test]
fn a_king_is_not_moved_from_an_empty_column_bottom_to_another_empty_column() {
    let mut game = table();
    game.columns[0] = column("", "KH QS");
    assert!(game.legal_moves().contains(&transfer(tab(0), tab(1), 2)));
    assert_eq!(game.hint(), None);
}

#[test]
fn a_king_buried_in_a_column_goes_to_an_empty_column() {
    let mut game = table();
    game.columns[0] = column("", "3C KH QS");
    assert_eq!(game.hint(), Some(transfer(tab(0), tab(1), 2)));
}

#[test]
fn following_the_hints_never_repeats_a_position() {
    for seed in 0..30 {
        let mut game = Yukon::deal(YukonRules::Classic, seed);
        let mut seen = vec![game.piles()];
        while let Some(hint) = game.hint() {
            assert_eq!(game.apply(hint), Ok(()), "seed {seed}: {hint:?}");
            let piles = game.piles();
            assert!(!seen.contains(&piles), "seed {seed}: a position came back");
            seen.push(piles);
        }
    }
}
