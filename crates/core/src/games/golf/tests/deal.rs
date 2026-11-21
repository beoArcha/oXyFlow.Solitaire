use super::*;
use crate::game::{StockStatus, Variant, deal};
use crate::pile::PileSnapshot;

fn dealt(seed: u64) -> Golf {
    Golf::deal(seed)
}

fn pile_len(piles: &[PileSnapshot], pile: PileRef) -> usize {
    piles
        .iter()
        .find(|p| p.pile == pile)
        .map_or(0, |p| p.cards.len())
}

#[test]
fn the_deal_is_seven_columns_of_five_a_waste_card_and_sixteen_in_the_stock() {
    let piles = dealt(7).piles();
    for index in 0..7u8 {
        assert_eq!(pile_len(&piles, tab(index)), 5, "column {index}");
    }
    assert_eq!(pile_len(&piles, PileRef::Waste), 1);
    assert_eq!(pile_len(&piles, PileRef::Stock), 16);
    assert_eq!(piles.len(), 9);
}

#[test]
fn every_column_card_and_the_waste_card_are_face_up_and_the_stock_is_face_down() {
    for snapshot in dealt(7).piles() {
        let expected = snapshot.pile != PileRef::Stock;
        assert!(
            snapshot.cards.iter().all(|c| c.face_up == expected),
            "{:?}",
            snapshot.pile
        );
    }
}

#[test]
fn the_deal_uses_the_whole_deck_once() {
    let mut ids: Vec<u16> = dealt(11)
        .piles()
        .iter()
        .flat_map(|pile| pile.cards.iter().map(|c| c.card.id.0))
        .collect();
    ids.sort_unstable();
    assert_eq!(ids, (0..52).collect::<Vec<u16>>());
}

#[test]
fn the_same_seed_gives_the_same_deal() {
    assert_eq!(
        deal(Variant::Golf, 99).piles(),
        deal(Variant::Golf, 99).piles()
    );
    assert_ne!(
        deal(Variant::Golf, 99).piles(),
        deal(Variant::Golf, 100).piles()
    );
}

#[test]
fn the_stock_is_counted_without_a_redeal() {
    assert_eq!(
        dealt(3).stock_status(),
        Some(StockStatus {
            recycles: 0,
            recycles_left: Some(0),
            deals_left: None,
        })
    );
}

#[test]
fn a_fresh_deal_is_not_won_and_has_no_forced_finish() {
    for seed in 0..20 {
        let game = dealt(seed);
        assert!(!game.is_won(), "seed {seed}");
        assert_eq!(game.forced_finish(), None, "seed {seed}");
    }
}
