//! Pyramid. Phase 1 stub: a real shuffled deal, every card in the stock, drawn to the waste.

use crate::deck;
use crate::game::{Game, PyramidRules};
use crate::games::stub::StockStub;
use crate::moves::{Move, MoveError};
use crate::pile::{PileRef, PileSnapshot};

pub(crate) struct Pyramid {
    table: StockStub,
}

impl Pyramid {
    pub(crate) fn deal(_rules: PyramidRules, seed: u64) -> Self {
        Self {
            table: StockStub::new(deck::standard(), seed, PileRef::Waste, 1),
        }
    }
}

impl Game for Pyramid {
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
