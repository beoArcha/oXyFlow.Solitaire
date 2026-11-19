//! Board geometry: where every pile and every card sits on the terminal grid. One
//! [`BoardGeometry`] feeds both drawing and mouse hit-testing, so what is drawn is what is clicked.
//!
//! Two layouts exist: columns (Klondike, Spider, FreeCell) and the pyramid. The card size is the
//! largest of [`SIZES`] that fits the window; gaps and margins grow with it.

mod chrome;
#[cfg(test)]
mod chrome_tests;
mod columns;
mod fan;
#[cfg(test)]
mod hit_tests;
mod pyramid;
#[cfg(test)]
mod render_tests;
mod size;
#[cfg(test)]
mod tests;

use oxysol_core::{PileRef, PileSnapshot, Variant};
use ratatui::layout::{Position, Rect};

pub use chrome::{
    BOARD_MIN_WIDTH, Chrome, Class, GUIDE_WIDTH, GUIDE_WIDTH_WIDE, MOVES_ROWS, MOVES_ROWS_TALL,
    STANDARD_MIN_WIDTH, WIDE_MIN_WIDTH, chrome, chrome_panels, chrome_with,
};
pub use size::{CardSize, FAN_BUDGET, SIZES};

/// The place of one pile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PileGeometry {
    pub pile: PileRef,
    /// Where the first card of the pile sits; the outline of an empty pile.
    pub slot: Rect,
    /// Bounding box and drop zone: the whole strip of a column, the fan of a waste.
    pub area: Rect,
    /// One rectangle per card, bottom to top. Covered cards overlap the cards above them; cards
    /// that are never visible share the rectangle of the pile.
    pub cards: Vec<Rect>,
    /// Leading face-down cards drawn as one row with a counter; their rectangles are that row.
    pub collapsed: usize,
}

/// What lies under a terminal cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hit {
    pub pile: PileRef,
    /// The topmost visible card under the cell; `None` for an empty slot or the free part of a
    /// column.
    pub card: Option<usize>,
}

/// The geometry of a whole board. `piles` follows the order of `Game::piles()`, which is also the
/// drawing order: a later card is drawn over an earlier one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardGeometry {
    pub card: CardSize,
    pub piles: Vec<PileGeometry>,
}

impl BoardGeometry {
    /// The geometry of `pile`.
    pub fn pile(&self, pile: PileRef) -> Option<&PileGeometry> {
        self.piles.iter().find(|geometry| geometry.pile == pile)
    }

    /// The card, or else the pile, under the cell (`column`, `row`).
    pub fn hit(&self, column: u16, row: u16) -> Option<Hit> {
        let at = Position::new(column, row);
        let on_card = self.piles.iter().rev().find_map(|pile| {
            let index = pile.cards.iter().rposition(|rect| rect.contains(at))?;
            Some(Hit {
                pile: pile.pile,
                card: Some(index),
            })
        });
        on_card.or_else(|| {
            self.piles
                .iter()
                .rev()
                .find(|pile| pile.area.contains(at))
                .map(|pile| Hit {
                    pile: pile.pile,
                    card: None,
                })
        })
    }

    fn clamp(&mut self, area: Rect) {
        for pile in &mut self.piles {
            pile.slot = pile.slot.intersection(area);
            pile.area = pile.area.intersection(area);
            for rect in &mut pile.cards {
                *rect = rect.intersection(area);
            }
        }
    }
}

/// Places the piles of `variant` (as listed by `Game::piles()`) inside `area`. Every rectangle
/// lies inside `area`.
pub fn compute(variant: Variant, piles: &[PileSnapshot], area: Rect) -> BoardGeometry {
    let mut geometry = match variant {
        Variant::Pyramid(_) => pyramid::compute(piles, area),
        _ => columns::compute(variant, piles, area),
    };
    geometry.clamp(area);
    geometry
}
