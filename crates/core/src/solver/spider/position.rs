//! The Spider position of the solver: ten columns with their face-down counts, the stock in the
//! order it will be dealt, and the number of complete runs already collected.

use crate::pile::{PileRef, PileSnapshot};
use crate::solver::face::{self, Face, NONE, fill, hidden_count, rank, suit};
use crate::solver::hash::{PositionHash, column_hash};

pub(super) const COLUMNS: usize = 10;
pub(super) const RUN: usize = 13;
const RUNS_TO_WIN: u8 = 8;
const COLUMN_CAPACITY: usize = 104;
const STOCK_CAPACITY: usize = 50;

#[derive(Clone)]
pub(in crate::solver) struct Position {
    pub(super) cols: [[Face; COLUMN_CAPACITY]; COLUMNS],
    pub(super) len: [usize; COLUMNS],
    pub(super) hidden: [usize; COLUMNS],
    stock: [Face; STOCK_CAPACITY],
    pub(super) stock_len: usize,
    pub(super) removed: u8,
}

fn same_suit_below(upper: Face, lower: Face) -> bool {
    rank(upper) == rank(lower) + 1 && suit(upper) == suit(lower)
}

impl Position {
    /// Reads the snapshot of a Spider game. `None` when it does not look like one.
    pub(in crate::solver) fn from_piles(piles: &[PileSnapshot]) -> Option<Self> {
        let mut position = Self {
            cols: [[NONE; COLUMN_CAPACITY]; COLUMNS],
            len: [0; COLUMNS],
            hidden: [0; COLUMNS],
            stock: [NONE; STOCK_CAPACITY],
            stock_len: 0,
            removed: 0,
        };
        let stock = face::cards_of(piles, PileRef::Stock);
        position.stock_len = usize::from(fill(&mut position.stock, stock)?);
        for i in 0..COLUMNS {
            let cards = face::cards_of(piles, PileRef::Tableau(u8::try_from(i).ok()?));
            position.len[i] = usize::from(fill(&mut position.cols[i], cards)?);
            position.hidden[i] = usize::from(hidden_count(cards)?);
        }
        let collected = piles
            .iter()
            .filter(|snapshot| matches!(snapshot.pile, PileRef::Foundation(_)))
            .filter(|snapshot| !snapshot.cards.is_empty())
            .count();
        position.removed = u8::try_from(collected).ok()?;
        Some(position)
    }

    pub(super) fn top(&self, column: usize) -> Option<Face> {
        self.len[column]
            .checked_sub(1)
            .map(|top| self.cols[column][top])
    }

    pub(super) fn face_up(&self, column: usize) -> usize {
        self.len[column] - self.hidden[column]
    }

    /// The length of the same-suit descending tail of `column`: the cards that move together.
    pub(super) fn run_len(&self, column: usize) -> usize {
        let up = &self.cols[column][self.hidden[column]..self.len[column]];
        let linked = up
            .windows(2)
            .rev()
            .take_while(|pair| same_suit_below(pair[0], pair[1]))
            .count();
        if up.is_empty() { 0 } else { linked + 1 }
    }

    pub(super) fn accepts(&self, column: usize, card: Face) -> bool {
        self.top(column)
            .is_none_or(|top| rank(top) == rank(card) + 1)
    }

    pub(super) fn can_deal(&self) -> bool {
        self.stock_len > 0 && self.len.iter().all(|&len| len > 0)
    }

    pub(super) fn first_empty_column(&self) -> Option<usize> {
        self.len.iter().position(|&len| len == 0)
    }

    fn flip(&mut self, column: usize) {
        if self.len[column] == self.hidden[column] && self.hidden[column] > 0 {
            self.hidden[column] -= 1;
        }
    }

    /// Moves a complete king-to-ace run off the top of `column`, then turns over what it covered.
    fn collect(&mut self, column: usize) {
        if self.face_up(column) < RUN {
            return;
        }
        let start = self.len[column] - RUN;
        let run = &self.cols[column][start..self.len[column]];
        let complete = rank(run[0]) == face::KING
            && run.windows(2).all(|pair| same_suit_below(pair[0], pair[1]));
        if complete {
            self.len[column] = start;
            self.removed += 1;
            self.flip(column);
        }
    }

    pub(super) fn move_run(&mut self, from: usize, to: usize, count: usize) {
        let start = self.len[from] - count;
        for offset in 0..count {
            self.cols[to][self.len[to] + offset] = self.cols[from][start + offset];
        }
        self.len[to] += count;
        self.len[from] = start;
        self.flip(from);
        self.collect(to);
    }

    pub(super) fn deal_row(&mut self) {
        for column in 0..COLUMNS {
            self.stock_len -= 1;
            self.cols[column][self.len[column]] = self.stock[self.stock_len];
            self.len[column] += 1;
        }
        for column in 0..COLUMNS {
            self.collect(column);
        }
    }

    pub(super) fn is_won(&self) -> bool {
        self.removed == RUNS_TO_WIN
    }

    pub(super) fn key(&self) -> u128 {
        let mut columns = [0u64; COLUMNS];
        for (i, slot) in columns.iter_mut().enumerate() {
            *slot = column_hash(
                u8::try_from(self.hidden[i]).unwrap_or(0),
                &self.cols[i][..self.len[i]],
            );
        }
        columns.sort_unstable();
        let mut hash = PositionHash::new();
        for value in columns {
            hash.feed(value);
        }
        hash.feed(u64::try_from(self.stock_len).unwrap_or(0));
        hash.feed(u64::from(self.removed));
        hash.key()
    }
}
