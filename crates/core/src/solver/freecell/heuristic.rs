//! The estimate of the work left in a FreeCell position (lower is closer to a win).

use super::position::{COLUMNS, Position, SUITS};
use crate::solver::face::{NONE, rank};

const BASE: i32 = 2000;
const PER_CARD_HOME: i32 = 20;
const PER_BURIER: i32 = 6;
const PER_BLOCKER: i32 = 3;
const PER_FULL_CELL: i32 = 4;
const PER_EMPTY_COLUMN: i32 = 8;

impl Position {
    /// Whether every column falls in rank from bottom to top. Then the lowest card left is always
    /// on top of a column or in a cell, so moving cards home one by one wins.
    pub(super) fn columns_descend(&self) -> bool {
        (0..COLUMNS).all(|column| {
            self.cols[column][..self.len[column]]
                .windows(2)
                .all(|pair| rank(pair[0]) > rank(pair[1]))
        })
    }

    /// The cards that sit above the next card each foundation needs.
    fn burying_cards(&self) -> usize {
        (0..SUITS)
            .filter_map(|suit| {
                let wanted = u8::try_from(suit).ok()? * 13 + self.found[suit];
                (self.found[suit] < 13).then_some(wanted)
            })
            .map(|wanted| {
                (0..COLUMNS)
                    .find_map(|column| {
                        let cards = &self.cols[column][..self.len[column]];
                        cards
                            .iter()
                            .position(|&card| card == wanted)
                            .map(|at| cards.len() - 1 - at)
                    })
                    .unwrap_or(0)
            })
            .sum()
    }

    /// The cards that sit above a lower card of their own column.
    fn blocking_cards(&self) -> usize {
        (0..COLUMNS)
            .map(|column| {
                let mut lowest = u8::MAX;
                let mut blockers = 0;
                for &card in &self.cols[column][..self.len[column]] {
                    if rank(card) > lowest {
                        blockers += 1;
                    }
                    lowest = lowest.min(rank(card));
                }
                blockers
            })
            .sum()
    }

    pub(super) fn work_left(&self) -> u32 {
        let home = i32::from(self.found.iter().sum::<u8>());
        let full_cells = i32::try_from(self.cells.iter().filter(|&&c| c != NONE).count()).unwrap_or(0);
        let empty = i32::try_from(self.empty_columns()).unwrap_or(0);
        let burying = i32::try_from(self.burying_cards()).unwrap_or(0);
        let blocking = i32::try_from(self.blocking_cards()).unwrap_or(0);
        let score = BASE - PER_CARD_HOME * home
            + PER_BURIER * burying
            + PER_BLOCKER * blocking
            + PER_FULL_CELL * full_cells
            - PER_EMPTY_COLUMN * empty;
        u32::try_from(score.max(0)).unwrap_or(0)
    }
}
