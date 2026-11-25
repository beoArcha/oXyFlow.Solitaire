//! Golf: seven columns of five face-up cards, a stock and one waste pile. The top card of a column
//! may be played onto the waste when its rank is one higher or one lower than the waste top; the
//! stock turns one card onto the waste, once, with no redeal. The game is won when every column
//! is empty. The deal turns the first stock card onto the waste to start it.
//!
//! The module follows the shared layout of the games: state and deal here, validation and
//! execution in `rules`, move generation and the hint in `moves`, the forced win in `finish`.
//!
//! Whether a king and an ace are adjacent is the single constant [`KING_ACE_ADJACENT`]; the
//! traditional rule, kept here, is that they are not, so only a queen goes onto a king and only
//! a two onto an ace.

mod finish;
mod moves;
mod rules;
#[cfg(test)]
mod tests;

use crate::card::Card;
use crate::deck;
use crate::game::{Game, StockStatus};
use crate::games::progress::{self, HintProfile};
use crate::games::{index_ref, snapshot};
use crate::moves::{Move, MoveError};
use crate::pile::{PileRef, PileSnapshot};

pub(crate) const COLUMNS: usize = 7;
pub(crate) const COLUMN_CARDS: usize = 5;
const RANKS: u8 = 13;

/// Whether a king and an ace count as adjacent ranks. Traditional Golf: they do not.
pub(crate) const KING_ACE_ADJACENT: bool = false;

/// Whether two ranks, given by [`crate::Rank::index`], are one apart under the Golf rule.
pub(crate) fn ranks_adjacent(a: u8, b: u8) -> bool {
    let apart = a.abs_diff(b);
    apart == 1 || (KING_ACE_ADJACENT && apart == RANKS - 1)
}

/// A Golf game. The top card of every pile is the last element of its vector.
#[derive(Debug, Clone)]
pub(crate) struct Golf {
    stock: Vec<Card>,
    waste: Vec<Card>,
    columns: [Vec<Card>; COLUMNS],
}

impl Golf {
    /// Deals a shuffled deck from `seed`: five cards to each column in turn, the first card of the
    /// rest to the waste and the other 16 to the stock.
    pub(crate) fn deal(seed: u64) -> Self {
        let mut cards = deck::standard();
        deck::shuffle_with_seed(&mut cards, seed);
        let mut rest = cards.into_iter();
        let columns = std::array::from_fn(|_| rest.by_ref().take(COLUMN_CARDS).collect());
        let waste = rest.by_ref().take(1).collect();
        Self {
            stock: rest.collect(),
            waste,
            columns,
        }
    }
}

/// The hint profile of this game, chosen by the hints-only measurement of T-48.
const HINT_PROFILE: HintProfile = HintProfile::Rules;

impl Game for Golf {
    fn apply(&mut self, mv: Move) -> Result<(), MoveError> {
        if self.is_won() {
            return Err(MoveError::Unsupported);
        }
        self.execute(mv)
    }

    fn legal_moves(&self) -> Vec<Move> {
        if self.is_won() {
            return Vec::new();
        }
        self.enumerate_moves()
    }

    fn hint(&self) -> Option<Move> {
        if self.is_won() {
            return None;
        }
        progress::hint(self, HINT_PROFILE, progress::PLAY_KNOWLEDGE, || {
            self.suggest()
        })
    }

    fn is_won(&self) -> bool {
        self.columns.iter().all(Vec::is_empty)
    }

    fn piles(&self) -> Vec<PileSnapshot> {
        let columns = self
            .columns
            .iter()
            .enumerate()
            .map(|(i, cards)| snapshot(PileRef::Tableau(index_ref(i)), &[], cards));
        [
            snapshot(PileRef::Stock, &self.stock, &[]),
            snapshot(PileRef::Waste, &[], &self.waste),
        ]
        .into_iter()
        .chain(columns)
        .collect()
    }

    fn stock_status(&self) -> Option<StockStatus> {
        Some(StockStatus {
            recycles: 0,
            recycles_left: Some(0),
            deals_left: None,
        })
    }

    fn forced_finish(&self) -> Option<Vec<Move>> {
        self.finishing_moves()
    }
}
