use super::*;
use std::collections::HashSet;

const RULES: [PyramidRules; 2] = [PyramidRules::Relaxed, PyramidRules::Classic];
const BOTTOM_ROW: [&str; 7] = ["2C", "8H", "8C", "4D", "7H", "KS", "10S"];
const STOCK_TOP: &str = "10C";

fn expected_order() -> Vec<PileRef> {
    let mut order = vec![PileRef::Stock, PileRef::Waste, PileRef::Foundation(0)];
    for row in 0..7 {
        order.extend((0..=row).map(|index| pyr(row, index)));
    }
    order
}

#[test]
fn the_deal_has_the_pyramid_layout() {
    let piles = Pyramid::deal(PyramidRules::Relaxed, 7).piles();
    let order: Vec<PileRef> = piles.iter().map(|p| p.pile).collect();
    assert_eq!(order, expected_order());
    assert_eq!(piles.len(), 31);
    assert_eq!(piles[0].cards.len(), 24);
    let face_up: Vec<bool> = piles[0].cards.iter().map(|c| c.face_up).collect();
    assert_eq!(face_up.iter().filter(|up| **up).count(), 1);
    assert_eq!(face_up.last(), Some(&true));
    assert!(piles[1].cards.is_empty());
    assert!(piles[2].cards.is_empty());
    for pile in &piles[3..] {
        assert_eq!(pile.cards.len(), 1, "{:?}", pile.pile);
        assert!(pile.cards[0].face_up, "{:?}", pile.pile);
    }
    let ids: HashSet<u16> = piles
        .iter()
        .flat_map(|p| p.cards.iter().map(|c| c.card.id.0))
        .collect();
    assert_eq!(ids.len(), 52);
}

#[test]
fn both_rule_sets_deal_the_same_layout_for_a_seed() {
    for seed in [0, 1, 42, u64::MAX] {
        let relaxed = Pyramid::deal(PyramidRules::Relaxed, seed).piles();
        assert_eq!(relaxed, Pyramid::deal(PyramidRules::Classic, seed).piles());
    }
}

#[test]
fn golden_deal_for_seed_42() {
    for rules in RULES {
        let game = Pyramid::deal(rules, 42);
        let bottom: Vec<String> = game.rows[6]
            .iter()
            .map(|slot| code(&slot.unwrap()))
            .collect();
        assert_eq!(bottom, BOTTOM_ROW, "{rules:?}");
        let top = game.stock.last().map(code);
        assert_eq!(top.as_deref(), Some(STOCK_TOP), "{rules:?}");
    }
}

#[test]
fn a_fresh_game_is_not_won() {
    for rules in RULES {
        assert!(!Pyramid::deal(rules, 42).is_won(), "{rules:?}");
    }
}

#[test]
fn a_fresh_deal_covers_every_card_but_the_bottom_row() {
    for rules in RULES {
        let game = Pyramid::deal(rules, 7);
        let pyramid: Vec<PileRef> = game
            .piles()
            .iter()
            .map(|pile| pile.pile)
            .filter(|pile| matches!(pile, PileRef::Pyramid { .. }))
            .collect();
        assert_eq!(pyramid.len(), 28);
        let uncovered: Vec<PileRef> = pyramid
            .iter()
            .copied()
            .filter(|&pile| !game.is_covered(pile))
            .collect();
        let bottom: Vec<PileRef> = (0..7).map(|index| pyr(6, index)).collect();
        assert_eq!(uncovered, bottom, "{rules:?}");
        assert_eq!(pyramid.len() - uncovered.len(), 21);
    }
}

#[test]
fn a_removed_or_non_pyramid_pile_is_never_covered() {
    let game = Pyramid::deal(PyramidRules::Relaxed, 7);
    assert!(!game.is_covered(PileRef::Stock));
    assert!(!game.is_covered(PileRef::Waste));
    assert!(!game.is_covered(PileRef::Foundation(0)));
    assert!(!game.is_covered(pyr(7, 0)));
    assert!(!game.is_covered(pyr(0, 9)));
}
