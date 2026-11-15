use super::*;
use std::collections::HashSet;

const GOLDEN_BOTTOM_TWO: [[&str; 2]; 8] = [
    ["7C", "3H"],
    ["KD", "10D"],
    ["6H", "5D"],
    ["2C", "8H"],
    ["4H", "3D"],
    ["7S", "3S"],
    ["5H", "6C"],
    ["3C", "9H"],
];
const GOLDEN_TOP: [&str; 8] = ["AH", "4S", "7D", "10S", "6S", "KH", "QC", "10C"];

#[test]
fn the_deal_has_the_freecell_layout() {
    let piles = FreeCell::deal(7).piles();
    let order: Vec<PileRef> = piles.iter().map(|p| p.pile).collect();
    let expected: Vec<PileRef> = (0..4)
        .map(found)
        .chain((0..4).map(cell))
        .chain((0..8).map(tab))
        .collect();
    assert_eq!(order, expected);
    assert!(piles[..8].iter().all(|p| p.cards.is_empty()));
    let sizes: Vec<usize> = piles[8..].iter().map(|p| p.cards.len()).collect();
    assert_eq!(sizes, [7, 7, 7, 7, 6, 6, 6, 6]);
    assert!(piles.iter().flat_map(|p| &p.cards).all(|c| c.face_up));
    let ids: HashSet<u16> = piles
        .iter()
        .flat_map(|p| p.cards.iter().map(|c| c.card.id.0))
        .collect();
    assert_eq!(ids.len(), 52);
}

#[test]
fn the_same_seed_gives_the_same_layout() {
    for seed in [0, 1, 42, u64::MAX] {
        let first = FreeCell::deal(seed).piles();
        assert_eq!(first, FreeCell::deal(seed).piles());
    }
    assert_ne!(FreeCell::deal(43).piles(), FreeCell::deal(42).piles());
}

#[test]
fn golden_deal_for_seed_42() {
    let game = FreeCell::deal(42);
    let columns = &game.piles()[8..];
    for (index, pile) in columns.iter().enumerate() {
        let codes: Vec<String> = pile.cards.iter().map(|c| code(&c.card)).collect();
        assert_eq!(
            codes[..2],
            GOLDEN_BOTTOM_TWO[index],
            "column {index} bottom"
        );
        assert_eq!(codes.last().map(String::as_str), Some(GOLDEN_TOP[index]));
    }
}
