//! The column layout: a top row of stock, waste, cells and foundations above fanned tableau
//! columns. Cells and the stock sit on the left, foundations on the right.

use oxysol_core::{DrawCount, PileRef, PileSnapshot, Rank, Variant};
use ratatui::layout::Rect;

use super::fan::fan;
use super::size::{CardSize, FAN_BUDGET, spacing};
use super::{BoardGeometry, PileGeometry};

const MIN_GAP: u16 = 1;
const FAN_CARDS: u16 = 3;

#[derive(Debug, Clone, Copy, Default)]
struct Counts {
    tableau: u16,
    foundations: u16,
    cells: u16,
    stock: bool,
    waste: bool,
}

impl Counts {
    fn of(piles: &[PileSnapshot]) -> Self {
        let mut counts = Self::default();
        for snapshot in piles {
            match snapshot.pile {
                PileRef::Tableau(_) => counts.tableau += 1,
                PileRef::Foundation(_) => counts.foundations += 1,
                PileRef::Cell(_) => counts.cells += 1,
                PileRef::Stock => counts.stock = true,
                PileRef::Waste => counts.waste = true,
                PileRef::Pyramid { .. } => {}
            }
        }
        counts
    }
}

/// Columns of a covered waste card that stay visible: its left border and its corner label, `10♥`
/// or `Q♥`, so a fanned card is as wide as its label plus one.
fn covered_width(rank: Rank) -> u16 {
    if rank == Rank::Ten { 4 } else { 3 }
}

fn waste_columns(size: CardSize, gap: u16, fanned: bool) -> u16 {
    if !fanned {
        return 1;
    }
    let span = size.width + (FAN_CARDS - 1) * size.waste_step() + gap;
    span.div_ceil(size.width + gap)
}

fn columns_for(counts: Counts, size: CardSize, gap: u16, fanned: bool) -> u16 {
    let waste = if counts.waste {
        waste_columns(size, gap, fanned)
    } else {
        0
    };
    let left = u16::from(counts.stock) + waste + counts.cells;
    counts.tableau.max(left + counts.foundations)
}

fn fits(counts: Counts, fanned: bool, size: CardSize, area: Rect) -> bool {
    let columns = columns_for(counts, size, MIN_GAP, fanned);
    let width = columns * size.width + columns.saturating_sub(1) * MIN_GAP;
    let height = 2 * size.height + size.row_gap() + FAN_BUDGET;
    width <= area.width && height <= area.height
}

struct Grid {
    size: CardSize,
    x0: u16,
    pitch: u16,
    columns: u16,
    waste_columns: u16,
    fanned: bool,
    top_y: u16,
    tableau: Rect,
}

impl Grid {
    fn new(counts: Counts, fanned: bool, area: Rect, limit: Option<CardSize>) -> Self {
        let size = CardSize::largest_fit_within(limit, |size| fits(counts, fanned, size, area));
        let columns = columns_for(counts, size, MIN_GAP, fanned);
        let (gap, margin) = spacing(area.width, columns, size.width);
        let tableau_y = (area.y + size.height + size.row_gap()).min(area.bottom());
        Self {
            size,
            x0: area.x + margin,
            pitch: size.width + gap,
            columns: columns_for(counts, size, gap, fanned),
            waste_columns: waste_columns(size, gap, fanned),
            fanned,
            top_y: area.y,
            tableau: Rect::new(area.x, tableau_y, area.width, area.bottom() - tableau_y),
        }
    }

    fn column_x(&self, column: u16) -> u16 {
        self.x0 + column * self.pitch
    }

    fn stacked(&self, pile: &PileSnapshot, column: u16) -> PileGeometry {
        let slot = Rect::new(
            self.column_x(column),
            self.top_y,
            self.size.width,
            self.size.height,
        );
        PileGeometry {
            pile: pile.pile,
            slot,
            area: slot,
            cards: vec![slot; pile.cards.len()],
            collapsed: 0,
        }
    }

    fn waste(&self, pile: &PileSnapshot, column: u16) -> PileGeometry {
        let mut geometry = self.stacked(pile, column);
        if !self.fanned {
            return geometry;
        }
        geometry.area.width += (FAN_CARDS - 1) * self.size.waste_step();
        let len = geometry.cards.len();
        let shown = len.min(usize::from(FAN_CARDS));
        let mut x = geometry.slot.x;
        for (rect, table_card) in geometry.cards[len - shown..]
            .iter_mut()
            .zip(&pile.cards[len - shown..])
        {
            rect.x = x;
            x += covered_width(table_card.card.rank);
        }
        geometry
    }

    fn column(&self, pile: &PileSnapshot, column: u16) -> PileGeometry {
        let x = self.column_x(column);
        let y = self.tableau.y;
        let fan = fan(&pile.cards, self.size, self.tableau.height);
        let cards = fan
            .ys
            .iter()
            .enumerate()
            .map(|(index, offset)| {
                let height = if index < fan.collapsed {
                    1
                } else {
                    self.size.height
                };
                Rect::new(x, y + offset, self.size.width, height)
            })
            .collect();
        PileGeometry {
            pile: pile.pile,
            slot: Rect::new(x, y, self.size.width, self.size.height),
            area: Rect::new(x, y, self.size.width, self.tableau.height),
            cards,
            collapsed: fan.collapsed,
        }
    }
}

pub(super) fn compute(
    variant: Variant,
    piles: &[PileSnapshot],
    area: Rect,
    limit: Option<CardSize>,
) -> BoardGeometry {
    let counts = Counts::of(piles);
    let fanned = variant == Variant::Klondike(DrawCount::Three);
    let grid = Grid::new(counts, fanned, area, limit);
    let waste_column = u16::from(counts.stock);
    let cells_column = waste_column + if counts.waste { grid.waste_columns } else { 0 };
    let foundations_column = grid.columns.saturating_sub(counts.foundations);
    let placed = piles
        .iter()
        .map(|pile| match pile.pile {
            PileRef::Waste => grid.waste(pile, waste_column),
            PileRef::Cell(index) => grid.stacked(pile, cells_column + u16::from(index)),
            PileRef::Foundation(index) => grid.stacked(pile, foundations_column + u16::from(index)),
            PileRef::Tableau(index) => grid.column(pile, u16::from(index)),
            PileRef::Stock | PileRef::Pyramid { .. } => grid.stacked(pile, 0),
        })
        .collect();
    BoardGeometry {
        card: grid.size,
        piles: placed,
    }
}
