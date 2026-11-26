//! The TriPeaks layout: four overlapping rows of 3, 6, 9, and 10 cards forming three peaks,
//! centered, with the stock and the waste in a row below.

use oxysol_core::{PileRef, PileSnapshot};
use ratatui::layout::Rect;

use super::scale::{FanScale, InterfaceSize};
use super::size::{CardSize, cap_limit};
use super::{BoardGeometry, PileGeometry};

const ROWS: u16 = 4;
const BASE_CARDS: u16 = 10;

fn height_of(size: CardSize, step: u16, fan: FanScale) -> u16 {
    (ROWS - 1) * step + 2 * size.height + fan.row_gap(size)
}

fn fits(size: CardSize, area: Rect, step: u16, fan: FanScale) -> bool {
    BASE_CARDS * size.width <= area.width && height_of(size, step, fan) <= area.height
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
    base_gap: u16,
    row_gap: u16,
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
        let span = 2 * width + self.base_gap;
        let start = self.x0 + (BASE_CARDS * width).saturating_sub(span) / 2;
        let y = self.y0 + (ROWS - 1) * self.step + self.size.height + self.row_gap;
        self.card(start + offset * (width + self.base_gap), y)
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
    let margin = area.width.saturating_sub(BASE_CARDS * size.width) / 2;
    let table = Table {
        size,
        step,
        base_gap: scale.gaps.tripeaks_base_gap,
        row_gap: fan.row_gap(size),
        x0: area.x + margin,
        y0: area.y + area.height.saturating_sub(height_of(size, step, fan)) / 2,
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
