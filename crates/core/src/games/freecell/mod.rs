//! FreeCell: eight tableau columns, all cards face up, four free cells and four foundations. The
//! module follows the shared layout of the games: state and deal here, validation and execution in
//! `rules`, move generation and the hint in `moves`.

mod moves;
mod rules;
#[cfg(test)]
mod tests;

use crate::card::Card;
use crate::deck;
use crate::game::Game;
use crate::games::{index_ref, snapshot};
use crate::moves::{Move, MoveError};
use crate::pile::{PileRef, PileSnapshot};

const COLUMNS: usize = 8;
const CELLS: usize = 4;
const FOUNDATIONS: usize = 4;
const DECK_SIZE: usize = 52;
const DEEP_COLUMNS: usize = 4;
const DEEP_COLUMN_SIZE: usize = 7;
const SHALLOW_COLUMN_SIZE: usize = 6;

/// A FreeCell game. The top card of every pile is the last element of its vector. A cell holds at
/// most one card; the rules keep it that way.
#[derive(Debug, Clone)]
pub(crate) struct FreeCell {
    cells: [Vec<Card>; CELLS],
    foundations: [Vec<Card>; FOUNDATIONS],
    columns: [Vec<Card>; COLUMNS],
}

impl FreeCell {
    fn empty() -> Self {
        Self {
            cells: Default::default(),
            foundations: Default::default(),
            columns: Default::default(),
        }
    }

    /// Deals a shuffled deck from `seed`: the first four columns hold seven cards, the other four
    /// six, all face up.
    pub(crate) fn deal(seed: u64) -> Self {
        let mut cards = deck::standard();
        deck::shuffle_with_seed(&mut cards, seed);
        let mut game = Self::empty();
        let mut rest = cards.into_iter();
        for (index, column) in game.columns.iter_mut().enumerate() {
            let size = if index < DEEP_COLUMNS {
                DEEP_COLUMN_SIZE
            } else {
                SHALLOW_COLUMN_SIZE
            };
            column.extend(rest.by_ref().take(size));
        }
        game
    }
}

impl Game for FreeCell {
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
        self.suggest()
    }

    fn is_won(&self) -> bool {
        self.foundations.iter().map(Vec::len).sum::<usize>() == DECK_SIZE
    }

    fn piles(&self) -> Vec<PileSnapshot> {
        let foundations = self
            .foundations
            .iter()
            .enumerate()
            .map(|(i, cards)| snapshot(PileRef::Foundation(index_ref(i)), &[], cards));
        let cells = self
            .cells
            .iter()
            .enumerate()
            .map(|(i, cards)| snapshot(PileRef::Cell(index_ref(i)), &[], cards));
        let columns = self
            .columns
            .iter()
            .enumerate()
            .map(|(i, cards)| snapshot(PileRef::Tableau(index_ref(i)), &[], cards));
        foundations.chain(cells).chain(columns).collect()
    }
}
