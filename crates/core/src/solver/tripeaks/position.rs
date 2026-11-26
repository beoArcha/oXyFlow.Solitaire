//! The TriPeaks position of the solver: which of the 28 peak cards remain (a bit mask), the
//! stock ranks and length, and the waste top rank.

use crate::games::tripeaks::{POSITIONS, ROW_LENS, ROWS, children_of, ranks_adjacent};
use crate::pile::{PileRef, PileSnapshot};
use crate::solver::face::{self, NONE, cards_of, rank};
use crate::solver::hash::PositionHash;

const STOCK_CAPACITY: usize = 24;
const RANKS: usize = 13;
const ALL_RANKS: u16 = (1 << RANKS) - 1;

const fn row_start(row: usize) -> usize {
    let mut start = 0;
    let mut r = 0;
    while r < row {
        start += ROW_LENS[r];
        r += 1;
    }
    start
}

const fn layout() -> [(u8, u8); POSITIONS] {
    let mut places = [(0, 0); POSITIONS];
    let mut at = 0;
    let mut row = 0;
    while row < ROWS {
        let mut index = 0;
        while index < ROW_LENS[row] {
            places[at] = (row as u8, index as u8);
            at += 1;
            index += 1;
        }
        row += 1;
    }
    places
}

const fn covers() -> [u32; POSITIONS] {
    let mut table = [0u32; POSITIONS];
    let places = layout();
    let mut at = 0;
    while at < POSITIONS {
        let (row, index) = (places[at].0 as usize, places[at].1 as usize);
        if let Some([left, right]) = children_of(row, index) {
            let base = row_start(row + 1);
            table[at] = (1 << (base + left)) | (1 << (base + right));
        }
        at += 1;
    }
    table
}

pub(super) const PLACES: [(u8, u8); POSITIONS] = layout();
pub(super) const COVERS: [u32; POSITIONS] = covers();

#[derive(Clone)]
pub(in crate::solver) struct Position {
    pub(super) ranks: [u8; POSITIONS],
    pub(super) mask: u32,
    stock: [u8; STOCK_CAPACITY],
    pub(super) stock_len: usize,
    pub(super) waste: u8,
}

fn ranks_of(piles: &[PileSnapshot], pile: PileRef) -> impl Iterator<Item = u8> + '_ {
    cards_of(piles, pile)
        .iter()
        .map(|table_card| rank(face::face(table_card.card)))
}

impl Position {
    /// Reads the snapshot of a TriPeaks game. `None` when it does not look like one.
    pub(in crate::solver) fn from_piles(piles: &[PileSnapshot]) -> Option<Self> {
        let mut position = Self {
            ranks: [NONE; POSITIONS],
            mask: 0,
            stock: [NONE; STOCK_CAPACITY],
            stock_len: 0,
            waste: ranks_of(piles, PileRef::Waste).last().unwrap_or(NONE),
        };
        let stock: Vec<u8> = ranks_of(piles, PileRef::Stock).collect();
        position
            .stock
            .get_mut(..stock.len())?
            .copy_from_slice(&stock);
        position.stock_len = stock.len();
        for (at, &(row, index)) in PLACES.iter().enumerate() {
            let cards = face::cards_of(piles, PileRef::Pyramid { row, index });
            if let Some(card) = cards.first() {
                position.ranks[at] = rank(face::face(card.card));
                position.mask |= 1 << at;
            }
        }
        Some(position)
    }

    pub(super) fn is_free_under(mask: u32, at: usize) -> bool {
        mask & (1 << at) != 0 && mask & COVERS[at] == 0
    }

    pub(super) fn is_free(&self, at: usize) -> bool {
        Self::is_free_under(self.mask, at)
    }

    pub(super) fn can_play(&self, at: usize) -> bool {
        self.is_free(at) && self.waste != NONE && ranks_adjacent(self.ranks[at], self.waste)
    }

    pub(super) fn can_draw(&self) -> bool {
        self.stock_len > 0
    }

    pub(super) fn play(&mut self, at: usize) {
        self.mask &= !(1 << at);
        self.waste = self.ranks[at];
    }

    pub(super) fn draw(&mut self) {
        if let Some(next) = self.stock_len.checked_sub(1) {
            self.stock_len = next;
            self.waste = self.stock[next];
        }
    }

    pub(super) fn is_won(&self) -> bool {
        self.mask == 0
    }

    /// True when the remaining peak ranks form more circular rank groups than chains can start.
    pub(super) fn is_dead(&self) -> bool {
        if self.mask == 0 {
            return false;
        }
        let mut rank_bits = 0u16;
        for at in 0..POSITIONS {
            if self.mask & (1 << at) != 0 {
                rank_bits |= 1 << self.ranks[at];
            }
        }
        let components = if rank_bits == ALL_RANKS {
            1
        } else {
            (0..RANKS)
                .filter(|&r| {
                    let prev = (r + RANKS - 1) % RANKS;
                    rank_bits & (1 << r) != 0 && rank_bits & (1 << prev) == 0
                })
                .count()
        };
        let can_start_now = usize::from((0..POSITIONS).any(|at| self.can_play(at)));
        components > self.stock_len + can_start_now
    }

    pub(super) fn key(&self) -> u128 {
        let packed =
            (u64::from(self.mask) << 16) | ((self.stock_len as u64) << 8) | u64::from(self.waste);
        let mut hash = PositionHash::new();
        hash.feed(packed);
        hash.key()
    }
}
