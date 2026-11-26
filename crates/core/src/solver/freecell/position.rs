//! The FreeCell position of the solver: eight columns, four cells, the foundation heights and where
//! the engine keeps each foundation.

use crate::pile::{PileRef, PileSnapshot};
use crate::solver::face::{self, Face, NONE, fill, rank, suit};
use crate::solver::hash::{PositionHash, column_hash, feed_unordered};

pub(super) const COLUMNS: usize = 8;
pub(super) const CELLS: usize = 4;
pub(super) const SUITS: usize = 4;
pub(super) const DECK: u8 = 52;
const COLUMN_CAPACITY: usize = 24;

#[derive(Clone)]
pub(in crate::solver) struct Position {
    pub(super) cols: [[Face; COLUMN_CAPACITY]; COLUMNS],
    pub(super) len: [usize; COLUMNS],
    pub(super) cells: [Face; CELLS],
    pub(super) found: [u8; SUITS],
    slot: [u8; SUITS],
}

impl Position {
    /// Reads the snapshot of a FreeCell game. `None` when it does not look like one.
    pub(in crate::solver) fn from_piles(piles: &[PileSnapshot]) -> Option<Self> {
        let mut position = Self {
            cols: [[NONE; COLUMN_CAPACITY]; COLUMNS],
            len: [0; COLUMNS],
            cells: [NONE; CELLS],
            found: [0; SUITS],
            slot: [NONE; SUITS],
        };
        for i in 0..COLUMNS {
            let cards = face::cards_of(piles, PileRef::Tableau(u8::try_from(i).ok()?));
            position.len[i] = usize::from(fill(&mut position.cols[i], cards)?);
        }
        for i in 0..CELLS {
            let cards = face::cards_of(piles, PileRef::Cell(u8::try_from(i).ok()?));
            position.cells[i] = cards.first().map_or(NONE, |cell| face::face(cell.card));
        }
        for i in 0..SUITS {
            let cards = face::cards_of(piles, PileRef::Foundation(u8::try_from(i).ok()?));
            let Some(last) = cards.last() else {
                continue;
            };
            let suit = usize::from(suit(face::face(last.card)));
            position.found[suit] = u8::try_from(cards.len()).ok()?;
            position.slot[suit] = u8::try_from(i).ok()?;
        }
        Some(position)
    }

    pub(super) fn top(&self, column: usize) -> Option<Face> {
        self.len[column]
            .checked_sub(1)
            .map(|top| self.cols[column][top])
    }

    pub(super) fn pop(&mut self, column: usize) -> Face {
        self.len[column] -= 1;
        self.cols[column][self.len[column]]
    }

    pub(super) fn push(&mut self, column: usize, card: Face) {
        self.cols[column][self.len[column]] = card;
        self.len[column] += 1;
    }

    pub(super) fn free_cells(&self) -> usize {
        self.cells.iter().filter(|&&cell| cell == NONE).count()
    }

    pub(super) fn first_free_cell(&self) -> Option<usize> {
        self.cells.iter().position(|&cell| cell == NONE)
    }

    pub(super) fn first_empty_column(&self) -> Option<usize> {
        self.len.iter().position(|&len| len == 0)
    }

    pub(super) fn empty_columns(&self) -> usize {
        self.len.iter().filter(|&&len| len == 0).count()
    }

    /// The length of the longest descending alternating run at the top of `column`.
    pub(super) fn run_len(&self, column: usize) -> usize {
        let cards = &self.cols[column][..self.len[column]];
        let linked = cards
            .windows(2)
            .rev()
            .take_while(|pair| face::fits_alternating(pair[0], pair[1]))
            .count();
        if cards.is_empty() { 0 } else { linked + 1 }
    }

    /// The most cards that move onto a column at once: `(free cells + 1) * 2^(empty columns)`, an
    /// empty target column not counted as one of them.
    pub(super) fn capacity(&self, target_is_empty: bool) -> usize {
        let empties = self.empty_columns() - usize::from(target_is_empty);
        (self.free_cells() + 1) << empties.min(16)
    }

    pub(super) fn accepts(&self, column: usize, card: Face) -> bool {
        self.top(column)
            .is_none_or(|top| face::fits_alternating(top, card))
    }

    pub(super) fn foundation_accepts(&self, card: Face) -> bool {
        self.found[usize::from(suit(card))] == rank(card)
    }

    /// The engine foundation that takes `card`: its own one, or the first empty one for an ace.
    pub(super) fn foundation_index(&self, card: Face) -> u8 {
        let own = usize::from(suit(card));
        if self.found[own] > 0 {
            return self.slot[own];
        }
        (0..SUITS)
            .filter_map(|i| u8::try_from(i).ok())
            .find(|&i| (0..SUITS).all(|s| self.found[s] == 0 || self.slot[s] != i))
            .unwrap_or(0)
    }

    pub(super) fn place_home(&mut self, card: Face) {
        let own = usize::from(suit(card));
        if self.found[own] == 0 {
            self.slot[own] = self.foundation_index(card);
        }
        self.found[own] += 1;
    }

    pub(super) fn is_won(&self) -> bool {
        self.found.iter().copied().sum::<u8>() == DECK
    }

    pub(super) fn key(&self) -> u128 {
        let mut columns: [u64; COLUMNS] =
            std::array::from_fn(|i| column_hash(0, &self.cols[i][..self.len[i]]));
        let mut hash = PositionHash::new();
        feed_unordered(&mut hash, &mut columns);
        let mut cells = self.cells;
        cells.sort_unstable();
        hash.feed_bytes(&cells);
        hash.feed_bytes(&self.found);
        hash.key()
    }
}
