//! The Golf position of the solver. Columns only ever lose their top card, the stock only its top
//! card, and only the rank of the waste top matters, so a position is the column heights, the
//! stock size and that rank.

use crate::games::golf::{COLUMN_CARDS, COLUMNS, ranks_adjacent};
use crate::pile::{PileRef, PileSnapshot};
use crate::solver::face::{self, NONE, cards_of, rank};
use crate::solver::hash::PositionHash;

const STOCK_CAPACITY: usize = 24;

#[derive(Clone)]
pub(in crate::solver) struct Position {
    columns: [[u8; COLUMN_CARDS]; COLUMNS],
    heights: [usize; COLUMNS],
    stock: [u8; STOCK_CAPACITY],
    stock_len: usize,
    waste: u8,
}

fn ranks_of(piles: &[PileSnapshot], pile: PileRef) -> impl Iterator<Item = u8> + '_ {
    cards_of(piles, pile)
        .iter()
        .map(|table_card| rank(face::face(table_card.card)))
}

impl Position {
    /// Reads the snapshot of a Golf game. `None` when it does not look like one.
    pub(in crate::solver) fn from_piles(piles: &[PileSnapshot]) -> Option<Self> {
        let mut position = Self {
            columns: [[NONE; COLUMN_CARDS]; COLUMNS],
            heights: [0; COLUMNS],
            stock: [NONE; STOCK_CAPACITY],
            stock_len: 0,
            waste: ranks_of(piles, PileRef::Waste).last().unwrap_or(NONE),
        };
        for column in 0..COLUMNS {
            let pile = PileRef::Tableau(u8::try_from(column).ok()?);
            for (slot, value) in position.columns[column]
                .iter_mut()
                .zip(ranks_of(piles, pile))
            {
                *slot = value;
                position.heights[column] += 1;
            }
            if cards_of(piles, pile).len() != position.heights[column] {
                return None;
            }
        }
        let stock: Vec<u8> = ranks_of(piles, PileRef::Stock).collect();
        position
            .stock
            .get_mut(..stock.len())?
            .copy_from_slice(&stock);
        position.stock_len = stock.len();
        Some(position)
    }

    /// The rank `depth` cards below the top of `column`, 0 being the top card.
    fn from_top(&self, column: usize, depth: usize) -> Option<u8> {
        let index = self.heights.get(column)?.checked_sub(depth + 1)?;
        self.columns.get(column)?.get(index).copied()
    }

    pub(super) fn top(&self, column: usize) -> Option<u8> {
        self.from_top(column, 0)
    }

    /// The card under the top card of `column`.
    pub(super) fn below(&self, column: usize) -> Option<u8> {
        self.from_top(column, 1)
    }

    pub(super) fn can_play(&self, column: usize) -> bool {
        self.top(column)
            .is_some_and(|top| self.waste != NONE && ranks_adjacent(top, self.waste))
    }

    pub(super) fn height(&self, column: usize) -> usize {
        self.heights.get(column).copied().unwrap_or(0)
    }

    pub(super) fn can_draw(&self) -> bool {
        self.stock_len > 0
    }

    /// Plays the top card of `column` onto the waste.
    pub(super) fn play(&mut self, column: usize) {
        if let Some(top) = self.top(column) {
            self.waste = top;
            self.heights[column] -= 1;
        }
    }

    /// Turns the top stock card onto the waste.
    pub(super) fn draw(&mut self) {
        if let Some(next) = self.stock_len.checked_sub(1) {
            self.stock_len = next;
            self.waste = self.stock[next];
        }
    }

    pub(super) fn is_won(&self) -> bool {
        self.heights.iter().all(|&height| height == 0)
    }

    pub(super) fn key(&self) -> u128 {
        let heights = self
            .heights
            .iter()
            .fold(0u64, |packed, &height| (packed << 3) | height as u64);
        let mut hash = PositionHash::new();
        hash.feed(heights);
        hash.feed(((self.stock_len as u64) << 8) | u64::from(self.waste));
        hash.key()
    }
}
