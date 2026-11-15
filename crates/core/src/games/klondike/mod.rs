//! Klondike. Phase 1 stub: a real shuffled deal, every card in the stock, drawn to the waste.

use crate::deck;
use crate::game::{DrawCount, Game};
use crate::games::stub::StockStub;
use crate::moves::{Move, MoveError};
use crate::pile::{PileRef, PileSnapshot};

pub(crate) struct Klondike {
    table: StockStub,
}

impl Klondike {
    pub(crate) fn deal(draw: DrawCount, seed: u64) -> Self {
        let per_draw = match draw {
            DrawCount::One => 1,
            DrawCount::Three => 3,
        };
        Self {
            table: StockStub::new(deck::standard(), seed, PileRef::Waste, per_draw),
        }
    }
}

impl Game for Klondike {
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
