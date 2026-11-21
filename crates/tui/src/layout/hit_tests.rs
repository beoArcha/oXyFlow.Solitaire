use super::*;
use oxysol_core::{DrawCount, PyramidRules, deal};

const SMALL: Rect = Rect::new(0, 0, 100, 27);
const LABEL_WIDTH: u16 = 3;

fn all_rects(geometry: &BoardGeometry) -> Vec<(usize, usize, Rect)> {
    let mut rects = Vec::new();
    for (pile, placed) in geometry.piles.iter().enumerate() {
        for (index, rect) in placed.cards.iter().enumerate() {
            rects.push((pile, index, *rect));
        }
    }
    rects
}

fn label_rect(rect: Rect) -> Rect {
    Rect::new(rect.x + 1, rect.y, LABEL_WIDTH, 1)
}

fn without_cards(mut piles: Vec<PileSnapshot>, pile: PileRef) -> Vec<PileSnapshot> {
    for snapshot in piles.iter_mut().filter(|snapshot| snapshot.pile == pile) {
        snapshot.cards.clear();
    }
    piles
}

#[test]
fn hit_finds_the_topmost_card_and_the_free_part_of_a_column() {
    let variant = Variant::Klondike(DrawCount::One);
    let piles = deal(variant, 7).piles();
    let geometry = compute(variant, &piles, SMALL);
    for (pile, placed) in piles.iter().zip(&geometry.piles) {
        let Some(top) = placed.cards.last() else {
            continue;
        };
        let hit = geometry.hit(top.x + 1, top.bottom() - 1);
        assert_eq!(
            hit,
            Some(Hit {
                pile: pile.pile,
                card: Some(pile.cards.len() - 1)
            })
        );
    }
    let last = geometry
        .pile(PileRef::Tableau(6))
        .map(|p| (p.area, p.cards.len()));
    let (area, len) = last.unwrap_or_default();
    let hit = geometry.hit(area.x, area.bottom() - 1);
    assert_eq!(
        hit.map(|h| (h.pile, h.card)),
        Some((PileRef::Tableau(6), None))
    );
    assert_eq!(len, 7);
    assert_eq!(geometry.hit(SMALL.right() + 3, 0), None);
}

#[test]
fn hit_picks_each_visible_face_up_card_by_its_label() {
    for variant in Variant::ALL {
        let piles = deal(variant, 11).piles();
        let geometry = compute(variant, &piles, SMALL);
        let rects = all_rects(&geometry);
        for (position, (pile, index, rect)) in rects.iter().enumerate() {
            let stacked = geometry.piles[*pile].cards.get(index + 1) == Some(rect);
            if stacked || !piles[*pile].cards[*index].face_up {
                continue;
            }
            let covered = rects[position + 1..]
                .iter()
                .any(|(_, _, later)| label_rect(*rect).intersects(*later));
            assert!(!covered);
            let hit = geometry.hit(rect.x + 1, rect.y);
            let expected = Hit {
                pile: piles[*pile].pile,
                card: Some(*index),
            };
            assert_eq!(hit, Some(expected), "{variant:?} {:?}", piles[*pile].pile);
        }
    }
}

#[test]
fn an_empty_pyramid_position_does_not_steal_clicks_from_the_row_above() {
    let variant = Variant::Pyramid(PyramidRules::Relaxed);
    let mut piles = deal(variant, 3).piles();
    let removed = PileRef::Pyramid { row: 6, index: 0 };
    piles = without_cards(piles, removed);
    let geometry = compute(variant, &piles, SMALL);
    let above = geometry
        .pile(PileRef::Pyramid { row: 5, index: 0 })
        .map(|p| p.slot);
    let above = above.unwrap_or_default();
    let hit = geometry.hit(above.x + 1, above.bottom() - 1);
    assert_eq!(
        hit.map(|h| (h.pile, h.card)),
        Some((PileRef::Pyramid { row: 5, index: 0 }, Some(0)))
    );
}

#[test]
fn a_click_on_each_free_cell_and_foundation_hits_that_slot() {
    let variant = Variant::FreeCell;
    let piles = deal(variant, 5).piles();
    for area in [SMALL, Rect::new(0, 0, 100, 30)] {
        let geometry = compute(variant, &piles, area);
        for pile in (0..4)
            .map(PileRef::Cell)
            .chain((0..4).map(PileRef::Foundation))
        {
            let slot = geometry.pile(pile).map(|p| p.slot).unwrap_or_default();
            let at = (slot.x + slot.width / 2, slot.y + slot.height / 2);
            let hit = geometry.hit(at.0, at.1);
            assert_eq!(hit, Some(Hit { pile, card: None }), "{pile:?} in {area:?}");
        }
    }
}

#[test]
fn the_group_gap_between_the_cells_and_the_foundations_is_no_slot() {
    let variant = Variant::FreeCell;
    let piles = deal(variant, 5).piles();
    let geometry = compute(variant, &piles, SMALL);
    let cell = geometry
        .pile(PileRef::Cell(3))
        .map(|p| p.slot)
        .unwrap_or_default();
    let foundation = geometry
        .pile(PileRef::Foundation(0))
        .map(|p| p.slot)
        .unwrap_or_default();
    let middle = cell.right() + (foundation.x - cell.right()) / 2;
    assert_eq!(geometry.hit(middle, cell.y), None);
}
