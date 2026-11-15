//! The pyramid layout: seven overlapping rows, centered, with the stock, the waste and the pile of
//! removed cards in a row below. Cards of a row touch, so the row above is covered completely
//! except for its top rows, and no fragment of it shows between two cards.

use oxysol_core::{PileRef, PileSnapshot};
use ratatui::layout::Rect;

use super::size::CardSize;
use super::{BoardGeometry, PileGeometry};

const ROWS: u16 = 7;

fn height_of(size: CardSize) -> u16 {
    (ROWS - 1) * size.pyramid_step() + 2 * size.height + size.row_gap()
}

fn fits(size: CardSize, area: Rect) -> bool {
    ROWS * size.width <= area.width && height_of(size) <= area.height
}

struct Table {
    size: CardSize,
    x0: u16,
    y0: u16,
    pitch: u16,
}

impl Table {
    fn card(&self, x: u16, y: u16) -> Rect {
        Rect::new(x, y, self.size.width, self.size.height)
    }

    fn pyramid_card(&self, row: u16, index: u16) -> Rect {
        let x = self.x0 + (ROWS - 1 - row + 2 * index) * self.pitch / 2;
        self.card(x, self.y0 + row * self.size.pyramid_step())
    }

    fn base_slot(&self, offset: u16) -> Rect {
        let width = self.size.width;
        let y = self.y0 + (ROWS - 1) * self.size.pyramid_step() + self.size.height;
        let x = self.x0 + 2 * width - 1 + offset * (width + 1);
        self.card(x, y + self.size.row_gap())
    }

    fn slot_for(&self, pile: PileRef) -> Rect {
        match pile {
            PileRef::Pyramid { row, index } => self.pyramid_card(u16::from(row), u16::from(index)),
            PileRef::Stock => self.base_slot(0),
            PileRef::Waste => self.base_slot(1),
            _ => self.base_slot(2),
        }
    }
}

pub(super) fn compute(piles: &[PileSnapshot], area: Rect) -> BoardGeometry {
    let size = CardSize::largest_fit(|size| fits(size, area));
    let margin = area.width.saturating_sub(ROWS * size.width) / 2;
    let table = Table {
        size,
        x0: area.x + margin,
        y0: area.y + area.height.saturating_sub(height_of(size)) / 2,
        pitch: size.width,
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
