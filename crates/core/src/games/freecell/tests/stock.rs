use super::*;

#[test]
fn freecell_has_no_stock_status() {
    let mut game = FreeCell::deal(42);
    assert_eq!(game.stock_status(), None);
    let first = game.legal_moves().first().copied();
    assert!(first.is_some());
    if let Some(mv) = first {
        assert_eq!(game.apply(mv), Ok(()));
    }
    assert_eq!(game.stock_status(), None);
}
