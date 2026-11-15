//! FreeCell. Phase 1 stub: a real shuffled deal, every card in the stock, drawn to the first cell.

use crate::deck;
use crate::game::Game;
use crate::games::stub::StockStub;
use crate::moves::{Move, MoveError};
use crate::pile::{PileRef, PileSnapshot};

pub(crate) struct FreeCell {
    table: StockStub,
}

impl FreeCell {
    pub(crate) fn deal(seed: u64) -> Self {
        Self {
            table: StockStub::new(deck::standard(), seed, PileRef::Cell(0), 1),
        }
    }
}

impl Game for FreeCell {
    fn apply(&mut self, mv: Move) -> Result<(), MoveError> {
        self.table.apply(mv)
    }

    fn legal_moves(&self) -> Vec<Move> {
        self.table.legal_moves()
    }

    fn hint(&self) -> Option<Move> {
        self.table.hint()
    }

    fn is_won(&self) -> bool {
        self.table.is_won()
    }

    fn piles(&self) -> Vec<PileSnapshot> {
        self.table.piles()
    }
}
