use super::*;

const EMPTY_ROWS: [&str; 7] = [
    ".",
    ". .",
    ". . .",
    ". . . .",
    ". . . . .",
    ". . . . . .",
    ". . . . . . .",
];

fn with_bottom_row(bottom: &str, stock: &str, waste: &str) -> Pyramid {
    let mut rows = EMPTY_ROWS;
    rows[6] = bottom;
    table(PyramidRules::Relaxed, rows, stock, waste)
}

#[test]
fn a_king_is_hinted_before_a_pair() {
    let game = with_bottom_row("6C 7D 2H JS 8C 5D KH", "", "");
    assert_eq!(game.hint(), Some(remove(pyr(6, 6), None)));
}

#[test]
fn a_pair_of_pyramid_cards_is_hinted_before_a_stock_pair() {
    let game = with_bottom_row("6C 7D . . . . .", "6S", "7H");
    assert_eq!(game.hint(), Some(remove(pyr(6, 0), Some(pyr(6, 1)))));
}

#[test]
fn a_pyramid_card_with_the_stock_is_hinted_before_a_stock_pair() {
    let game = with_bottom_row("6C . . . . . .", "7S", "6S");
    assert_eq!(game.hint(), Some(remove(PileRef::Stock, Some(pyr(6, 0)))));
}

#[test]
fn a_stock_and_waste_pair_is_hinted_before_the_stock_move() {
    let game = with_bottom_row(". . . . . . .", "6S", "7H");
    assert_eq!(
        game.hint(),
        Some(remove(PileRef::Stock, Some(PileRef::Waste)))
    );
}

#[test]
fn the_stock_move_is_the_fallback() {
    let game = with_bottom_row(". . . . . . .", "AS 2S", "");
    assert_eq!(game.hint(), Some(Move::Stock));
}

#[test]
fn there_is_no_hint_without_a_legal_move() {
    let game = with_bottom_row(". . . . . . .", "", "");
    assert_eq!(game.hint(), None);
}

#[test]
fn a_covered_king_is_not_hinted() {
    let mut rows = EMPTY_ROWS;
    rows[5] = "KS . . . . .";
    rows[6] = "6C 7D . . . . .";
    let game = table(PyramidRules::Relaxed, rows, "", "");
    assert_eq!(game.hint(), Some(remove(pyr(6, 0), Some(pyr(6, 1)))));
}
