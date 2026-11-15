//! Spider. Phase 1 stub: a real shuffled deal of 104 cards, every card in the stock.

use crate::card::Suit;
use crate::deck;
use crate::game::{Game, SuitCount};
use crate::games::stub::StockStub;
use crate::moves::{Move, MoveError};
use crate::pile::{PileRef, PileSnapshot};

const ROW: usize = 10;

pub(crate) struct Spider {
    table: StockStub,
}

impl Spider {
    pub(crate) fn deal(suits: SuitCount, seed: u64) -> Self {
        let cards = match suits {
            SuitCount::One => deck::build(&[Suit::Spades], 8),
            SuitCount::Two => deck::build(&[Suit::Spades, Suit::Hearts], 4),
            SuitCount::Four => deck::build(&Suit::ALL, 2),
        };
        Self {
            table: StockStub::new(cards, seed, PileRef::Tableau(0), ROW),
        }
    }
}

impl Game for Spider {
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
