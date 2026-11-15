use super::*;
use oxysol_core::{Card, CardId, DrawCount, Rank, Session, Suit, SuitCount, TableCard, deal};

const SMALL: Rect = Rect::new(0, 0, 100, 27);
const LARGE: Rect = Rect::new(0, 0, 160, 47);
const LABEL_WIDTH: u16 = 3;

fn card(id: u16, face_up: bool) -> TableCard {
    TableCard {
        card: Card {
            id: CardId(id),
            suit: Suit::ALL[usize::from(id % 4)],
            rank: Rank::ALL[usize::from(id % 13)],
        },
        face_up,
    }
}

fn column(down: u16, up: u16) -> Vec<TableCard> {
    (0..down + up).map(|id| card(id, id >= down)).collect()
}

fn with_cards(
    mut piles: Vec<PileSnapshot>,
    pile: PileRef,
    cards: Vec<TableCard>,
) -> Vec<PileSnapshot> {
    for snapshot in piles.iter_mut().filter(|snapshot| snapshot.pile == pile) {
        snapshot.cards = cards.clone();
    }
    piles
}

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

fn assert_inside(geometry: &BoardGeometry, area: Rect, what: &str) {
    for placed in &geometry.piles {
        let rects = placed.cards.iter().chain([&placed.slot, &placed.area]);
        for rect in rects {
            assert_eq!(rect.intersection(area), *rect, "{what}: {:?}", placed.pile);
        }
    }
}

fn assert_ranks_visible(piles: &[PileSnapshot], geometry: &BoardGeometry, what: &str) {
    let rects = all_rects(geometry);
    for (position, (pile, index, rect)) in rects.iter().enumerate() {
        let snapshot = &piles[*pile];
        let stacked = geometry.piles[*pile].cards.get(index + 1) == Some(rect);
        if !snapshot.cards[*index].face_up || stacked {
            continue;
        }
        let label = label_rect(*rect);
        for (_, _, later) in &rects[position + 1..] {
            assert!(
                !label.intersects(*later),
                "{what}: {:?} card {index} is covered",
                snapshot.pile
            );
        }
    }
}

fn assert_tops_visible(piles: &[PileSnapshot], geometry: &BoardGeometry, what: &str) {
    for (pile, placed) in geometry.piles.iter().enumerate() {
        let Some(top) = placed.cards.last() else {
            continue;
        };
        assert_eq!(
            (top.width, top.height),
            (geometry.card.width, geometry.card.height),
            "{what}: {:?}",
            piles[pile].pile
        );
        for (other, rect) in all_rects(geometry)
            .into_iter()
            .filter_map(|(other, _, rect)| (other != pile).then_some((other, rect)))
        {
            assert!(
                !top.intersects(rect),
                "{what}: top of {:?} is covered by {:?}",
                piles[pile].pile,
                piles[other].pile
            );
        }
    }
}

fn check(variant: Variant, piles: &[PileSnapshot], area: Rect, what: &str) -> BoardGeometry {
    let geometry = compute(variant, piles, area);
    assert_eq!(geometry.piles.len(), piles.len(), "{what}");
    for (snapshot, placed) in piles.iter().zip(&geometry.piles) {
        assert_eq!(placed.pile, snapshot.pile, "{what}");
        assert_eq!(placed.cards.len(), snapshot.cards.len(), "{what}");
    }
    assert_inside(&geometry, area, what);
    assert_ranks_visible(piles, &geometry, what);
    if !matches!(variant, Variant::Pyramid(_)) {
        assert_tops_visible(piles, &geometry, what);
    }
    geometry
}

fn played_states(variant: Variant, seed: u64) -> Vec<Vec<PileSnapshot>> {
    let mut session = Session::new(variant, seed);
    let mut states = vec![session.game().piles()];
    for step in 0..160_usize {
        let moves = session.game().legal_moves();
        if moves.is_empty() {
            break;
        }
        if session.apply(moves[step * 7 % moves.len()]).is_err() {
            break;
        }
        if step % 4 == 0 {
            states.push(session.game().piles());
        }
    }
    states
}

#[test]
fn every_variant_fits_the_minimum_and_a_large_window() {
    for variant in Variant::ALL {
        for area in [SMALL, LARGE] {
            for seed in [1, 2, 3] {
                for (state, piles) in played_states(variant, seed).iter().enumerate() {
                    let what = format!("{variant:?} seed {seed} state {state} {area:?}");
                    check(variant, piles, area, &what);
                }
            }
        }
    }
}

#[test]
fn a_larger_window_gives_a_larger_card() {
    for variant in Variant::ALL {
        let piles = deal(variant, 5).piles();
        let small = compute(variant, &piles, SMALL).card;
        let large = compute(variant, &piles, LARGE).card;
        assert_eq!(small, CardSize::smallest(), "{variant:?}");
        assert!(
            large.width > small.width && large.height > small.height,
            "{variant:?}"
        );
    }
}

#[test]
fn card_size_grows_step_by_step_with_the_window() {
    let piles = deal(Variant::Spider(SuitCount::Four), 5).piles();
    let sizes: Vec<u16> = [(100, 27), (120, 37), (140, 41), (160, 47)]
        .into_iter()
        .map(|(width, height)| {
            compute(
                Variant::Spider(SuitCount::Four),
                &piles,
                Rect::new(0, 0, width, height),
            )
            .card
            .width
        })
        .collect();
    assert!(sizes.windows(2).all(|pair| pair[0] <= pair[1]), "{sizes:?}");
    assert_eq!(sizes.first(), Some(&7));
    assert_eq!(sizes.last(), Some(&13));
}

#[test]
fn gaps_and_margins_grow_with_the_window() {
    let variant = Variant::FreeCell;
    let piles = deal(variant, 5).piles();
    let gap_at = |area: Rect| {
        let geometry = compute(variant, &piles, area);
        let first = geometry
            .piles
            .iter()
            .find(|p| p.pile == PileRef::Tableau(0));
        let second = geometry
            .piles
            .iter()
            .find(|p| p.pile == PileRef::Tableau(1));
        match (first, second) {
            (Some(a), Some(b)) => b.slot.x - a.slot.right(),
            _ => 0,
        }
    };
    assert_eq!(gap_at(Rect::new(0, 0, 63, 27)), 1);
    assert!(gap_at(SMALL) >= 2);
}

#[test]
fn the_worst_spider_column_keeps_every_rank_at_the_minimum_size() {
    let variant = Variant::Spider(SuitCount::Four);
    let piles = with_cards(deal(variant, 9).piles(), PileRef::Tableau(0), column(5, 16));
    let geometry = check(variant, &piles, SMALL, "worst spider column");
    let placed = geometry
        .pile(PileRef::Tableau(0))
        .map(|p| (p.collapsed, p.cards.clone()));
    let (collapsed, cards) = placed.unwrap_or_default();
    assert_eq!(collapsed, 5);
    assert!(cards[5..].windows(2).all(|pair| pair[1].y == pair[0].y + 1));
    assert!(cards.iter().all(|rect| rect.bottom() <= SMALL.bottom()));
}

#[test]
fn an_absurd_column_still_fits_and_keeps_its_top_cards() {
    let variant = Variant::Spider(SuitCount::One);
    let piles = with_cards(deal(variant, 9).piles(), PileRef::Tableau(3), column(5, 60));
    let geometry = check_without_ranks(variant, &piles, SMALL);
    let cards = geometry.pile(PileRef::Tableau(3)).map(|p| p.cards.clone());
    let cards = cards.unwrap_or_default();
    assert!(
        cards[cards.len() - 10..]
            .windows(2)
            .all(|pair| pair[1].y == pair[0].y + 1)
    );
}

fn check_without_ranks(variant: Variant, piles: &[PileSnapshot], area: Rect) -> BoardGeometry {
    let geometry = compute(variant, piles, area);
    assert_inside(&geometry, area, "absurd column");
    assert_tops_visible(piles, &geometry, "absurd column");
    geometry
}

#[test]
fn face_down_cards_compress_before_the_face_up_ones() {
    let size = CardSize::smallest();
    let natural = fan::fan(&column(3, 3), size, 21);
    assert_eq!(natural.collapsed, 0);
    assert_eq!(natural.ys, vec![0, 1, 2, 3, 5, 7]);
    let tight = fan::fan(&column(6, 8), size, 18);
    assert_eq!(tight.collapsed, 0);
    let tighter = fan::fan(&column(6, 8), size, 17);
    assert_eq!(tighter.collapsed, 6);
    assert_eq!(tighter.ys[..6], [0; 6]);
    assert_eq!(tighter.ys[6], 1);
}

#[test]
fn klondike_draw_three_fans_the_top_three_waste_cards() {
    let variant = Variant::Klondike(DrawCount::Three);
    let piles = with_cards(deal(variant, 4).piles(), PileRef::Waste, column(0, 5));
    let geometry = check(variant, &piles, SMALL, "draw 3 waste");
    let waste = geometry
        .pile(PileRef::Waste)
        .map(|p| (p.slot, p.area, p.cards.clone()));
    let (slot, area, cards) = waste.unwrap_or_default();
    let xs: Vec<u16> = cards.iter().map(|rect| rect.x).collect();
    let step = geometry.card.waste_step();
    assert_eq!(
        xs,
        vec![slot.x, slot.x, slot.x, slot.x + step, slot.x + 2 * step]
    );
    assert_eq!(area.right(), slot.right() + 2 * step);
    for other in geometry.piles.iter().filter(|p| p.pile != PileRef::Waste) {
        assert!(!area.intersects(other.area), "{:?}", other.pile);
    }
}

#[test]
fn klondike_draw_one_waste_is_a_single_card() {
    let variant = Variant::Klondike(DrawCount::One);
    let piles = with_cards(deal(variant, 4).piles(), PileRef::Waste, column(0, 5));
    let geometry = check(variant, &piles, SMALL, "draw 1 waste");
    let waste = geometry.pile(PileRef::Waste);
    assert!(waste.is_some_and(|p| p.cards.iter().all(|rect| *rect == p.slot)));
}

#[test]
fn chrome_leaves_the_rest_to_the_body() {
    let parts = chrome(Rect::new(0, 0, 100, 30));
    assert_eq!(
        (parts.header.y, parts.body.y, parts.body.height),
        (0, 1, 25)
    );
    assert_eq!(
        (
            parts.status.y,
            parts.now.y,
            parts.possible.y,
            parts.footer.y
        ),
        (26, 27, 28, 29)
    );
    assert_eq!((parts.guide, parts.body.width), (None, 100));
}

#[test]
fn the_guide_takes_the_right_edge_only_when_the_board_keeps_its_width() {
    let wide = chrome_with(Rect::new(0, 0, 140, 40), true);
    assert_eq!(wide.guide, Some(Rect::new(110, 1, GUIDE_WIDTH, 35)));
    assert_eq!(wide.body, Rect::new(0, 1, 110, 35));
    let edge = chrome_with(Rect::new(0, 0, BOARD_MIN_WIDTH + GUIDE_WIDTH, 30), true);
    assert!(edge.guide.is_some());
    assert_eq!(edge.body.width, BOARD_MIN_WIDTH);
    let narrow = chrome_with(Rect::new(0, 0, BOARD_MIN_WIDTH + GUIDE_WIDTH - 1, 30), true);
    assert_eq!(narrow.guide, None);
    assert_eq!(narrow.body.width, BOARD_MIN_WIDTH + GUIDE_WIDTH - 1);
    assert_eq!(chrome_with(Rect::new(0, 0, 140, 40), false).guide, None);
}
