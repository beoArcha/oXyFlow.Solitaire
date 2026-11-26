//! The pyramid layout: seven overlapping rows, centered, with the stock, the waste and the pile of
//! removed cards in a row below. Cards of a row touch, so the row above is covered completely
//! except for its top rows, and no fragment of it shows between two cards.

use oxysol_core::{PileRef, PileSnapshot};
use ratatui::layout::Rect;

use super::scale::{FanScale, InterfaceSize};
use super::size::{CardSize, cap_limit};
use super::{BoardGeometry, PileGeometry};

const ROWS: u16 = 7;

fn height_of(size: CardSize, step: u16, fan: FanScale) -> u16 {
    (ROWS - 1) * step + 2 * size.height + fan.row_gap(size)
}

fn fits(size: CardSize, area: Rect, step: u16, fan: FanScale) -> bool {
    ROWS * size.width <= area.width && height_of(size, step, fan) <= area.height
}

struct Table {
    size: CardSize,
    step: u16,
    row_gap: u16,
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
        self.card(x, self.y0 + row * self.step)
    }

    fn base_slot(&self, offset: u16) -> Rect {
        let width = self.size.width;
        let y = self.y0 + (ROWS - 1) * self.step + self.size.height;
        let x = self.x0 + 2 * width - 1 + offset * (width + 1);
        self.card(x, y + self.row_gap)
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

pub(super) fn compute(
    piles: &[PileSnapshot],
    area: Rect,
    limit: Option<CardSize>,
    wider_fan: bool,
    interface_size: InterfaceSize,
) -> BoardGeometry {
    let scale = interface_size
        .fit(|scale| {
            let small = CardSize::smallest();
            fits(small, area, scale.fan.pyramid_step(small), scale.fan)
        })
        .scale();
    let fan = scale.fan;
    let limit = cap_limit(limit, scale.card_cap);
    let size =
        CardSize::largest_fit_within(limit, |size| fits(size, area, fan.pyramid_step(size), fan));
    let wide = fan.wide_pyramid_step(size);
    let step = if wider_fan && fits(size, area, wide, fan) {
        wide
    } else {
        fan.pyramid_step(size)
    };
    let margin = area.width.saturating_sub(ROWS * size.width) / 2;
    let table = Table {
        size,
        step,
        row_gap: fan.row_gap(size),
        x0: area.x + margin,
        y0: area.y + area.height.saturating_sub(height_of(size, step, fan)) / 2,
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
