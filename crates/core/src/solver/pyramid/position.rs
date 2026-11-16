//! The Pyramid position of the solver: which of the 28 pyramid cards remain (a bit mask), the
//! stock and the waste with their order, and the recycles left.

use crate::game::PyramidRules;
use crate::pile::{PileRef, PileSnapshot};
use crate::solver::face::{self, Face, NONE, fill, rank};
use crate::solver::hash::PositionHash;

pub(super) const POSITIONS: usize = 28;
const ROWS: usize = 7;
const TALON_CAPACITY: usize = 24;
pub(super) const TARGET: u8 = 13;
pub(super) const KING_VALUE: u8 = 13;

const fn row_start(row: usize) -> usize {
    row * (row + 1) / 2
}

/// `(row, index)` of every pyramid position, row by row.
const fn layout() -> [(u8, u8); POSITIONS] {
    let mut places = [(0, 0); POSITIONS];
    let mut at = 0;
    let mut row = 0;
    while row < ROWS {
        let mut index = 0;
        while index <= row {
            places[at] = (row as u8, index as u8);
            at += 1;
            index += 1;
        }
        row += 1;
    }
    places
}

/// The two positions that cover each pyramid position, as a bit mask.
const fn covers() -> [u32; POSITIONS] {
    let mut table = [0u32; POSITIONS];
    let places = layout();
    let mut at = 0;
    while at < POSITIONS {
        let (row, index) = (places[at].0 as usize, places[at].1 as usize);
        if row + 1 < ROWS {
            let below = row_start(row + 1) + index;
            table[at] = (1 << below) | (1 << (below + 1));
        }
        at += 1;
    }
    table
}

pub(super) const PLACES: [(u8, u8); POSITIONS] = layout();
pub(super) const COVERS: [u32; POSITIONS] = covers();

/// A card that may leave the table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::solver) enum Slot {
    Stock,
    Waste,
    Pyramid(u8),
}

#[derive(Clone)]
pub(in crate::solver) struct Position {
    pub(super) classic: bool,
    pub(super) values: [u8; POSITIONS],
    pub(super) mask: u32,
    stock: [Face; TALON_CAPACITY],
    pub(super) stock_len: usize,
    waste: [Face; TALON_CAPACITY],
    pub(super) waste_len: usize,
    pub(super) recycles_left: u32,
}

pub(super) fn value(face: Face) -> u8 {
    rank(face) + 1
}

impl Position {
    /// Reads the snapshot of a Pyramid game. `None` when it does not look like one.
    pub(in crate::solver) fn from_piles(
        rules: PyramidRules,
        piles: &[PileSnapshot],
        recycles_left: u32,
    ) -> Option<Self> {
        let mut position = Self {
            classic: rules == PyramidRules::Classic,
            values: [0; POSITIONS],
            mask: 0,
            stock: [NONE; TALON_CAPACITY],
            stock_len: 0,
            waste: [NONE; TALON_CAPACITY],
            waste_len: 0,
            recycles_left,
        };
        position.stock_len = usize::from(fill(
            &mut position.stock,
            face::cards_of(piles, PileRef::Stock),
        )?);
        position.waste_len = usize::from(fill(
            &mut position.waste,
            face::cards_of(piles, PileRef::Waste),
        )?);
        for (at, &(row, index)) in PLACES.iter().enumerate() {
            let cards = face::cards_of(piles, PileRef::Pyramid { row, index });
            if let Some(card) = cards.first() {
                position.values[at] = value(face::face(card.card));
                position.mask |= 1 << at;
            }
        }
        Some(position)
    }

    pub(super) fn stock_top(&self) -> Option<u8> {
        self.stock_len
            .checked_sub(1)
            .map(|top| value(self.stock[top]))
    }

    pub(super) fn waste_top(&self) -> Option<u8> {
        self.waste_len
            .checked_sub(1)
            .map(|top| value(self.waste[top]))
    }

    pub(super) fn talon_faces(&self) -> impl Iterator<Item = Face> + '_ {
        self.stock[..self.stock_len]
            .iter()
            .chain(&self.waste[..self.waste_len])
            .copied()
    }

    pub(super) fn is_free(&self, at: usize) -> bool {
        self.mask & (1 << at) != 0 && self.mask & COVERS[at] == 0
    }

    pub(super) fn can_draw(&self) -> bool {
        self.stock_len > 0 || (self.waste_len > 0 && self.recycles_left > 0)
    }

    /// One `Move::Stock`: the stock top to the waste, or the waste back to the stock.
    pub(super) fn draw(&mut self) {
        if self.stock_len > 0 {
            self.stock_len -= 1;
            self.waste[self.waste_len] = self.stock[self.stock_len];
            self.waste_len += 1;
            return;
        }
        for i in 0..self.waste_len {
            self.stock[i] = self.waste[self.waste_len - 1 - i];
        }
        self.stock_len = self.waste_len;
        self.waste_len = 0;
        self.recycles_left = self.recycles_left.saturating_sub(1);
    }

    pub(super) fn take(&mut self, slot: Slot) {
        match slot {
            Slot::Stock => self.stock_len = self.stock_len.saturating_sub(1),
            Slot::Waste => self.waste_len = self.waste_len.saturating_sub(1),
            Slot::Pyramid(at) => self.mask &= !(1 << at),
        }
    }

    pub(super) fn is_won(&self) -> bool {
        self.mask == 0 && (!self.classic || (self.stock_len == 0 && self.waste_len == 0))
    }

    pub(super) fn key(&self) -> u128 {
        let mut hash = PositionHash::new();
        hash.feed(u64::from(self.mask));
        hash.feed(u64::from(self.recycles_left));
        hash.feed_bytes(&self.stock[..self.stock_len]);
        hash.feed_bytes(&self.waste[..self.waste_len]);
        hash.key()
    }
}
