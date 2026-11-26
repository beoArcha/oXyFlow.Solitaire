//! The column layout: a top row of stock, waste, cells and foundations above fanned tableau
//! columns. Cells and the stock sit on the left, foundations on the right.

use oxysol_core::{DrawCount, PileRef, PileSnapshot, Rank, Variant};
use ratatui::layout::Rect;

use super::fan::{fan_for, fits_cleanly};
use super::scale::{FanScale, InterfaceSize, Scale};
use super::size::{CardSize, FAN_BUDGET, cap_limit, spacing_with};
use super::{BoardGeometry, PileGeometry};

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

fn waste_columns(size: CardSize, step: u16, gap: u16, fanned: bool) -> u16 {
    if !fanned {
        return 1;
    }
    let span = size.width + (FAN_CARDS - 1) * step + gap;
    span.div_ceil(size.width + gap)
}

fn columns_for(counts: Counts, size: CardSize, (waste_step, gap): (u16, u16), fanned: bool) -> u16 {
    let waste = if counts.waste {
        waste_columns(size, waste_step, gap, fanned)
    } else {
        0
    };
    let left = u16::from(counts.stock) + waste + counts.cells;
    counts.tableau.max(left + counts.foundations)
}

/// The extra width of the board: `gap` when the top row has both cells and foundations.
fn group_gap(counts: Counts, gap: u16) -> u16 {
    if counts.cells > 0 && counts.foundations > 0 {
        gap
    } else {
        0
    }
}

fn row_width(columns: u16, size: CardSize, gap: u16, counts: Counts, group: u16) -> u16 {
    columns * size.width + columns.saturating_sub(1) * gap + group_gap(counts, group)
}

fn fits_bounds(counts: Counts, fanned: bool, size: CardSize, area: Rect, scale: &Scale) -> bool {
    let columns = columns_for(
        counts,
        size,
        (scale.fan.waste_step(size), scale.gaps.min_gap),
        fanned,
    );
    let width = row_width(
        columns,
        size,
        scale.gaps.min_gap,
        counts,
        scale.gaps.group_gap,
    );
    let height = 2 * size.height + scale.fan.row_gap(size) + FAN_BUDGET;
    width <= area.width && height <= area.height
}

fn fits(
    counts: Counts,
    fanned: bool,
    piles: &[PileSnapshot],
    size: CardSize,
    area: Rect,
    wider_fan: bool,
    scale: &Scale,
) -> bool {
    if !fits_bounds(counts, fanned, size, area, scale) {
        return false;
    }
    let room = area
        .height
        .saturating_sub(size.height + scale.fan.row_gap(size));
    piles
        .iter()
        .filter(|pile| matches!(pile.pile, PileRef::Tableau(_)))
        .all(|pile| fits_cleanly(&pile.cards, size, room, wider_fan, scale.fan))
}

struct Grid {
    size: CardSize,
    x0: u16,
    pitch: u16,
    waste_columns: u16,
    waste_step: u16,
    fanned: bool,
    wider_fan: bool,
    fan_scale: FanScale,
    top_y: u16,
    tableau: Rect,
    group_gap: u16,
    foundations_column: u16,
}

impl Grid {
    fn new(
        counts: Counts,
        fanned: bool,
        piles: &[PileSnapshot],
        area: Rect,
        limit: Option<CardSize>,
        wider_fan: bool,
        interface_size: InterfaceSize,
    ) -> Self {
        let scale = interface_size
            .fit(|scale| fits_bounds(counts, fanned, CardSize::smallest(), area, scale))
            .scale();
        let limit = cap_limit(limit, scale.card_cap);
        let size = CardSize::largest_fit_within(limit, |size| {
            fits(counts, fanned, piles, size, area, wider_fan, scale)
        });
        let group_gap = group_gap(counts, scale.gaps.group_gap);
        let wide_step = scale.fan.wide_waste_step(size);
        let wide_cols = columns_for(counts, size, (wide_step, scale.gaps.min_gap), fanned);
        let waste_step = if wider_fan
            && row_width(
                wide_cols,
                size,
                scale.gaps.min_gap,
                counts,
                scale.gaps.group_gap,
            ) <= area.width
        {
            wide_step
        } else {
            scale.fan.waste_step(size)
        };
        let columns = columns_for(counts, size, (waste_step, scale.gaps.min_gap), fanned);
        let (gap, margin) = spacing_with(
            area.width.saturating_sub(group_gap),
            columns,
            size.width,
            scale.gaps.min_gap,
            scale.gaps.max_gap,
        );
        let tableau_y = (area.y + size.height + scale.fan.row_gap(size)).min(area.bottom());
        let columns = columns_for(counts, size, (waste_step, gap), fanned);
        Self {
            size,
            x0: area.x + margin,
            pitch: size.width + gap,
            waste_columns: waste_columns(size, waste_step, gap, fanned),
            waste_step,
            fanned,
            wider_fan,
            fan_scale: scale.fan,
            top_y: area.y,
            tableau: Rect::new(area.x, tableau_y, area.width, area.bottom() - tableau_y),
            group_gap,
            foundations_column: columns.saturating_sub(counts.foundations),
        }
    }

    fn column_x(&self, column: u16) -> u16 {
        let shift = if column >= self.foundations_column {
            self.group_gap
        } else {
            0
        };
        self.x0 + column * self.pitch + shift
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
        geometry.area.width += (FAN_CARDS - 1) * self.waste_step;
        let len = geometry.cards.len();
        let shown = len.min(usize::from(FAN_CARDS));
        let mut x = geometry.slot.x;
        for (rect, table_card) in geometry.cards[len - shown..]
            .iter_mut()
            .zip(&pile.cards[len - shown..])
        {
            rect.x = x;
            let step = covered_width(table_card.card.rank);
            x += if self.wider_fan {
                step.max(self.waste_step)
            } else {
                step
            };
        }
        geometry
    }

    fn column(&self, pile: &PileSnapshot, column: u16) -> PileGeometry {
        let x = self.column_x(column);
        let y = self.tableau.y;
        let fan = fan_for(
            &pile.cards,
            self.size,
            self.tableau.height,
            self.wider_fan,
            self.fan_scale,
        );
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
    wider_fan: bool,
    interface_size: InterfaceSize,
) -> BoardGeometry {
    let counts = Counts::of(piles);
    let fanned = variant == Variant::Klondike(DrawCount::Three);
    let grid = Grid::new(
        counts,
        fanned,
        piles,
        area,
        limit,
        wider_fan,
        interface_size,
    );
    let waste_column = u16::from(counts.stock);
    let cells_column = waste_column + if counts.waste { grid.waste_columns } else { 0 };
    let placed = piles
        .iter()
        .map(|pile| match pile.pile {
            PileRef::Waste => grid.waste(pile, waste_column),
            PileRef::Cell(index) => grid.stacked(pile, cells_column + u16::from(index)),
            PileRef::Foundation(index) => {
                grid.stacked(pile, grid.foundations_column + u16::from(index))
            }
            PileRef::Tableau(index) => grid.column(pile, u16::from(index)),
            PileRef::Stock | PileRef::Pyramid { .. } => grid.stacked(pile, 0),
        })
        .collect();
    BoardGeometry {
        card: grid.size,
        piles: placed,
    }
}
