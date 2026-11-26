use super::*;
use crate::game::{StockStatus, Variant, deal};
use crate::pile::PileSnapshot;

fn dealt(seed: u64) -> TriPeaks {
    TriPeaks::deal(seed)
}

fn pile_len(piles: &[PileSnapshot], pile: PileRef) -> usize {
    piles
        .iter()
        .find(|p| p.pile == pile)
        .map_or(0, |p| p.cards.len())
}

#[test]
fn the_deal_has_three_overlapping_peaks_one_waste_and_twenty_three_in_stock() {
    let game = dealt(7);
    let piles = game.piles();
    assert_eq!(piles.len(), 30);
    assert_eq!(pile_len(&piles, PileRef::Waste), 1);
    assert_eq!(pile_len(&piles, PileRef::Stock), 23);
    for (row, &expected_len) in ROW_LENS.iter().enumerate() {
        let count = piles
            .iter()
            .filter(|p| matches!(p.pile, PileRef::Pyramid { row: r, .. } if usize::from(r) == row))
            .count();
        assert_eq!(count, expected_len);
        for index in 0..expected_len {
            let pile = peak(row as u8, index as u8);
            let snap = piles.iter().find(|p| p.pile == pile).unwrap();
            assert_eq!(snap.cards.len(), 1, "{pile:?}");
            assert_eq!(snap.cards[0].face_up, row == 3, "{pile:?}");
            assert_eq!(game.is_covered(pile), row < 3, "{pile:?}");
        }
    }
}

#[test]
fn the_stock_is_face_down_and_the_waste_is_face_up() {
    for snapshot in dealt(7).piles() {
        match snapshot.pile {
            PileRef::Stock => assert!(snapshot.cards.iter().all(|c| !c.face_up)),
            PileRef::Waste => assert!(snapshot.cards.iter().all(|c| c.face_up)),
            _ => {}
        }
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
        deal(Variant::TriPeaks, 99).piles(),
        deal(Variant::TriPeaks, 99).piles()
    );
    assert_ne!(
        deal(Variant::TriPeaks, 99).piles(),
        deal(Variant::TriPeaks, 100).piles()
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
