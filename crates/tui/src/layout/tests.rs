use super::*;
use oxysol_core::{Card, CardId, DrawCount, Rank, Session, Suit, SuitCount, TableCard, deal};

const SMALL: Rect = Rect::new(0, 0, 100, 27);
const LARGE: Rect = Rect::new(0, 0, 160, 47);

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

fn label_rect(rect: Rect, rank: Rank) -> Rect {
    let width = if rank == Rank::Ten { 3 } else { 2 };
    Rect::new(rect.x + 1, rect.y, width, 1)
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
        let label = label_rect(*rect, snapshot.cards[*index].card.rank);
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
    if !matches!(variant, Variant::Pyramid(_) | Variant::TriPeaks) {
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
        if !matches!(variant, Variant::TriPeaks) {
            assert_eq!(small, CardSize::smallest(), "{variant:?}");
        }
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
fn the_tallest_yukon_column_keeps_every_rank_at_the_minimum_size() {
    let variant = Variant::Yukon(oxysol_core::YukonRules::Classic);
    let piles = with_cards(deal(variant, 9).piles(), PileRef::Tableau(6), column(6, 18));
    let geometry = check(variant, &piles, SMALL, "tallest yukon column");
    assert_eq!(
        geometry.pile(PileRef::Tableau(6)).map(|p| p.collapsed),
        Some(6)
    );
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
    let size = CardSize::new(13, 11);
    let natural = fan::fan(&column(6, 8), size, 60);
    assert_eq!(natural.collapsed, 0);
    assert_eq!(natural.ys[..3], [0, 2, 4]);
    let down_shrunk = fan::fan(&column(6, 8), size, 52);
    assert_eq!(down_shrunk.collapsed, 0);
    assert_eq!(down_shrunk.ys[..3], [0, 1, 2]);
    let up_shrunk = fan::fan(&column(6, 8), size, 40);
    assert_eq!(up_shrunk.collapsed, 0);
    let tighter = fan::fan(&column(6, 8), size, 23);
    assert_eq!(tighter.collapsed, 6);
    assert_eq!(tighter.ys[..6], [0; 6]);
    assert_eq!(tighter.ys[6], 1);
}

#[test]
fn at_a_down_step_of_one_two_leading_face_down_cards_collapse_at_once() {
    for size in [CardSize::smallest(), CardSize::new(9, 7)] {
        let roomy = fan::fan(&column(3, 3), size, 40);
        assert_eq!(roomy.collapsed, 3);
        assert_eq!(roomy.ys[..4], [0, 0, 0, 1]);
        let single = fan::fan(&column(1, 3), size, 40);
        assert_eq!(single.collapsed, 0);
        assert_eq!(single.ys[..2], [0, 1]);
    }
    let larger = fan::fan(&column(3, 3), CardSize::new(11, 9), 40);
    assert_eq!(larger.collapsed, 0);
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
    let covered = |index: usize| {
        let rank = piles[waste_index(&piles)].cards[index].card.rank;
        if rank == Rank::Ten { 4 } else { 3 }
    };
    let second = slot.x + covered(2);
    assert_eq!(
        xs,
        vec![slot.x, slot.x, slot.x, second, second + covered(3)]
    );
    assert_eq!(area.right(), slot.right() + 2 * step);
    for other in geometry.piles.iter().filter(|p| p.pile != PileRef::Waste) {
        assert!(!area.intersects(other.area), "{:?}", other.pile);
    }
}

fn waste_index(piles: &[PileSnapshot]) -> usize {
    piles
        .iter()
        .position(|snapshot| snapshot.pile == PileRef::Waste)
        .unwrap_or_default()
}

#[test]
fn klondike_draw_one_waste_is_a_single_card() {
    let variant = Variant::Klondike(DrawCount::One);
    let piles = with_cards(deal(variant, 4).piles(), PileRef::Waste, column(0, 5));
    let geometry = check(variant, &piles, SMALL, "draw 1 waste");
    let waste = geometry.pile(PileRef::Waste);
    assert!(waste.is_some_and(|p| p.cards.iter().all(|rect| *rect == p.slot)));
}

fn slot_of(geometry: &BoardGeometry, pile: PileRef) -> Rect {
    geometry.pile(pile).map(|p| p.slot).unwrap_or_default()
}

fn free_cell_gaps(geometry: &BoardGeometry) -> (Vec<u16>, u16) {
    let cells: Vec<Rect> = (0..4)
        .map(|index| slot_of(geometry, PileRef::Cell(index)))
        .collect();
    let foundations: Vec<Rect> = (0..4)
        .map(|index| slot_of(geometry, PileRef::Foundation(index)))
        .collect();
    let within = cells
        .windows(2)
        .chain(foundations.windows(2))
        .map(|pair| pair[1].x - pair[0].right())
        .collect();
    let between = foundations[0].x - cells[3].right();
    (within, between)
}

#[test]
fn the_free_cells_and_the_foundations_are_set_apart_at_every_size() {
    let variant = Variant::FreeCell;
    let piles = deal(variant, 5).piles();
    for size in SIZES {
        let geometry = compute_within(variant, &piles, LARGE, Some(size));
        assert_eq!(geometry.card, size);
        let (within, between) = free_cell_gaps(&geometry);
        let widest_within = within.iter().copied().max().unwrap_or_default();
        assert!(
            between > widest_within,
            "{size:?}: gap {between} between the groups, {within:?} within them"
        );
        assert!(within.iter().all(|gap| *gap == within[0]), "{size:?}");
    }
}

#[test]
fn the_tableau_stays_under_the_foundations_after_the_group_gap() {
    let variant = Variant::FreeCell;
    let piles = deal(variant, 5).piles();
    let geometry = check(variant, &piles, LARGE, "free cell group gap");
    for index in 0..4 {
        let foundation = slot_of(&geometry, PileRef::Foundation(index));
        let column = slot_of(&geometry, PileRef::Tableau(index + 4));
        assert_eq!(foundation.x, column.x, "foundation {index}");
    }
}

#[test]
fn free_cell_fits_at_a_hundred_by_thirty() {
    let variant = Variant::FreeCell;
    let piles = deal(variant, 5).piles();
    let area = Rect::new(0, 0, 100, 30);
    let geometry = check(variant, &piles, area, "free cell 100x30");
    assert_eq!(geometry.card, CardSize::new(9, 7));
    let (within, between) = free_cell_gaps(&geometry);
    assert!(
        within.iter().all(|gap| between > *gap),
        "{between} {within:?}"
    );
}

#[test]
fn games_without_cells_keep_their_uniform_gaps() {
    for variant in Variant::ALL {
        if matches!(variant, Variant::FreeCell | Variant::Pyramid(_)) {
            continue;
        }
        let piles = deal(variant, 5).piles();
        let geometry = compute(variant, &piles, LARGE);
        let xs: Vec<u16> = geometry
            .piles
            .iter()
            .filter_map(|p| match p.pile {
                PileRef::Tableau(_) => Some(p.slot.x),
                _ => None,
            })
            .collect();
        let gaps: Vec<u16> = xs.windows(2).map(|pair| pair[1] - pair[0]).collect();
        assert!(
            gaps.windows(2).all(|pair| pair[0] == pair[1]),
            "{variant:?}"
        );
    }
}
