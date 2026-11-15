//! Board geometry: where every pile and every card sits on the terminal grid. One
//! [`BoardGeometry`] feeds both drawing and mouse hit-testing, so what is drawn is what is clicked.
//!
//! Two layouts exist: columns (Klondike, Spider, FreeCell) and the pyramid. The card size is the
//! largest of [`SIZES`] that fits the window; gaps and margins grow with it.

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
use ratatui::layout::{Constraint, Layout, Position, Rect};

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

/// The parts of the board screen. `status` is the announcer line; `now` and `possible` are the two
/// rows of the action line. `guide` is the side panel, present only when it was asked for and the
/// terminal is wide enough to keep [`BOARD_MIN_WIDTH`] columns for the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chrome {
    pub header: Rect,
    pub body: Rect,
    pub guide: Option<Rect>,
    pub status: Rect,
    pub now: Rect,
    pub possible: Rect,
    pub footer: Rect,
}

/// Columns of the guide panel, its left edge included.
pub const GUIDE_WIDTH: u16 = 30;
/// Columns the board keeps when the guide panel is shown; narrower terminals hide the panel.
pub const BOARD_MIN_WIDTH: u16 = 100;

/// Splits the screen into header, board, status line, the two rows of the action line and footer,
/// without a guide panel. `body` is the area to pass to [`compute`].
pub fn chrome(area: Rect) -> Chrome {
    chrome_with(area, false)
}

/// Like [`chrome`], and with a guide panel at the right of the board when `guide` is set and
/// `area` is at least `BOARD_MIN_WIDTH + GUIDE_WIDTH` columns wide.
pub fn chrome_with(area: Rect, guide: bool) -> Chrome {
    let [header, middle, status, now, possible, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(area);
    let room = area.width >= BOARD_MIN_WIDTH + GUIDE_WIDTH;
    let (body, guide) = if guide && room {
        let [body, side] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(GUIDE_WIDTH)])
                .areas(middle);
        (body, Some(side))
    } else {
        (middle, None)
    };
    Chrome {
        header,
        body,
        guide,
        status,
        now,
        possible,
        footer,
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
