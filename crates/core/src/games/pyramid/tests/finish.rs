use super::*;

fn rows<'a>(row5: &'a str, row6: &'a str) -> [&'a str; 7] {
    [".", ". .", ". . .", ". . . .", ". . . . .", row5, row6]
}

fn assert_wins(game: &Pyramid) -> Vec<Move> {
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
fn a_pair_that_frees_a_king_is_a_forced_win() {
    let game = table(
        PyramidRules::Relaxed,
        rows("KH . . . . .", "5C 8D . . . . ."),
        "",
        "",
    );
    let expected = vec![remove(pyr(6, 0), Some(pyr(6, 1))), remove(pyr(5, 0), None)];
    assert_eq!(assert_wins(&game), expected);
}

#[test]
fn the_stock_top_and_the_waste_top_take_part() {
    let game = table(
        PyramidRules::Relaxed,
        rows(". . . . . .", "5C 4S . . . . ."),
        "8D",
        "9H",
    );
    assert_eq!(assert_wins(&game).len(), 2);
}

#[test]
fn classic_needs_the_stock_and_the_waste_cleared_too() {
    let mut game = table(
        PyramidRules::Classic,
        rows(". . . . . .", "KH 8D . . . . ."),
        "KD",
        "5C",
    );
    game.foundation = vec![card("AS"); 48];
    assert_eq!(assert_wins(&game).len(), 3);
}

#[test]
fn classic_with_a_card_buried_in_the_stock_gives_none() {
    let mut game = table(
        PyramidRules::Classic,
        rows(". . . . . .", "KH . . . . . ."),
        "2C KD",
        "",
    );
    game.foundation = vec![card("AS"); 49];
    assert_eq!(game.forced_finish(), None);
}

#[test]
fn relaxed_ignores_the_stock_once_the_pyramid_is_empty() {
    let game = table(
        PyramidRules::Relaxed,
        rows(". . . . . .", "KH . . . . . ."),
        "2C 3C",
        "",
    );
    assert_eq!(assert_wins(&game), vec![remove(pyr(6, 0), None)]);
}

#[test]
fn a_card_without_a_partner_gives_none() {
    let game = table(
        PyramidRules::Relaxed,
        rows(". . . . . .", "5C 6D . . . . ."),
        "2C",
        "",
    );
    assert_eq!(game.forced_finish(), None);
}

#[test]
fn a_partner_that_needs_a_draw_gives_none() {
    let game = table(
        PyramidRules::Relaxed,
        rows(". . . . . .", "5C . . . . . ."),
        "8D 2C",
        "",
    );
    assert_eq!(game.forced_finish(), None);
}

#[test]
fn a_won_game_has_nothing_to_finish() {
    assert_eq!(empty(PyramidRules::Relaxed).forced_finish(), None);
}

#[test]
fn golden_deals_are_not_forced_wins() {
    for rules in [PyramidRules::Relaxed, PyramidRules::Classic] {
        assert_eq!(Pyramid::deal(rules, 42).forced_finish(), None, "{rules:?}");
    }
}
