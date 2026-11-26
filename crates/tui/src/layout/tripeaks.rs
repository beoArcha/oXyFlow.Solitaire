//! The TriPeaks layout: four overlapping rows of 3, 6, 9, and 10 cards forming three peaks,
//! centered, with the stock and the waste in a row below.

use oxysol_core::{PileRef, PileSnapshot};
use ratatui::layout::Rect;

use super::size::CardSize;
use super::{BoardGeometry, PileGeometry};

const ROWS: u16 = 4;
const BASE_CARDS: u16 = 10;
const BASE_GAP: u16 = 2;

fn height_of(size: CardSize, step: u16) -> u16 {
    (ROWS - 1) * step + 2 * size.height + size.row_gap()
}

fn fits(size: CardSize, area: Rect, step: u16) -> bool {
    BASE_CARDS * size.width <= area.width && height_of(size, step) <= area.height
}

const fn half_units(row: u16, index: u16) -> u16 {
    match row {
        0 => 3 + 6 * index,
        1 => 2 + 2 * index + 2 * (index / 2),
        2 => 1 + 2 * index,
        _ => 2 * index,
    }
}

struct Table {
    size: CardSize,
    step: u16,
    x0: u16,
    y0: u16,
}

impl Table {
    fn card(&self, x: u16, y: u16) -> Rect {
        Rect::new(x, y, self.size.width, self.size.height)
    }

    fn peak_card(&self, row: u16, index: u16) -> Rect {
        let x = self.x0 + half_units(row, index) * self.size.width / 2;
        self.card(x, self.y0 + row * self.step)
    }

    fn base_slot(&self, offset: u16) -> Rect {
        let width = self.size.width;
        let span = 2 * width + BASE_GAP;
        let start = self.x0 + (BASE_CARDS * width).saturating_sub(span) / 2;
        let y = self.y0 + (ROWS - 1) * self.step + self.size.height + self.size.row_gap();
        self.card(start + offset * (width + BASE_GAP), y)
    }

    fn slot_for(&self, pile: PileRef) -> Rect {
        match pile {
            PileRef::Pyramid { row, index } => self.peak_card(u16::from(row), u16::from(index)),
            PileRef::Stock => self.base_slot(0),
            _ => self.base_slot(1),
        }
    }
}

pub(super) fn compute(
    piles: &[PileSnapshot],
    area: Rect,
    limit: Option<CardSize>,
    wider_fan: bool,
) -> BoardGeometry {
    let size = CardSize::largest_fit_within(limit, |size| fits(size, area, size.pyramid_step()));
    let wide = size.wide_pyramid_step();
    let step = if wider_fan && fits(size, area, wide) {
        wide
    } else {
        size.pyramid_step()
    };
    let margin = area.width.saturating_sub(BASE_CARDS * size.width) / 2;
    let table = Table {
        size,
        step,
        x0: area.x + margin,
        y0: area.y + area.height.saturating_sub(height_of(size, step)) / 2,
    };
    let placed = piles
        .iter()
        .map(|pile| {
            let slot = table.slot_for(pile.pile);
            PileGeometry {
                pile: pile.pile,
                slot,
                area: slot,
                cards: vec![slot; pile.cards.len()],
                collapsed: 0,
            }
        })
        .collect();
    BoardGeometry {
        card: size,
        piles: placed,
    }
}
