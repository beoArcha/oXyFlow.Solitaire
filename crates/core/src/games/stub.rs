//! Placeholder table shared by the game stubs of phase 1: a shuffled stock whose only legal move
//! is [`Move::Stock`]. Each game replaces it with its real layout and rules in phase 2.

use crate::card::Card;
use crate::deck;
use crate::game::Game;
use crate::moves::{Move, MoveError};
use crate::pile::{PileRef, PileSnapshot, TableCard};

pub(crate) struct StockStub {
    stock: Vec<Card>,
    drawn: Vec<Card>,
    target: PileRef,
    per_draw: usize,
}

impl StockStub {
    pub(crate) fn new(mut cards: Vec<Card>, seed: u64, target: PileRef, per_draw: usize) -> Self {
        deck::shuffle_with_seed(&mut cards, seed);
        Self {
            stock: cards,
            drawn: Vec::new(),
            target,
            per_draw,
        }
    }
}

fn view(pile: PileRef, cards: &[Card], face_up: bool) -> PileSnapshot {
    PileSnapshot {
        pile,
        cards: cards
            .iter()
            .map(|&card| TableCard { card, face_up })
            .collect(),
    }
}

impl Game for StockStub {
    fn apply(&mut self, mv: Move) -> Result<(), MoveError> {
        if mv != Move::Stock || self.stock.is_empty() {
            return Err(MoveError::Unsupported);
        }
        let take = self.per_draw.min(self.stock.len());
        let from = self.stock.len().saturating_sub(take);
        self.drawn.extend(self.stock.drain(from..));
        Ok(())
    }

    fn legal_moves(&self) -> Vec<Move> {
        if self.stock.is_empty() {
            Vec::new()
        } else {
            vec![Move::Stock]
        }
    }

    fn hint(&self) -> Option<Move> {
        self.legal_moves().into_iter().next()
    }

    fn is_won(&self) -> bool {
        false
    }

    fn piles(&self) -> Vec<PileSnapshot> {
        vec![
            view(PileRef::Stock, &self.stock, false),
            view(self.target, &self.drawn, true),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::Suit;

    fn stub(per_draw: usize) -> StockStub {
        StockStub::new(deck::build(&[Suit::Spades], 1), 1, PileRef::Waste, per_draw)
    }

    #[test]
    fn draws_until_the_stock_is_empty() {
        let mut table = stub(5);
        let mut draws = 0;
        while table.apply(Move::Stock).is_ok() {
            draws += 1;
        }
        assert_eq!(draws, 3);
        assert!(table.legal_moves().is_empty());
        assert_eq!(table.hint(), None);
        assert_eq!(table.piles()[0].cards.len(), 0);
        assert_eq!(table.piles()[1].cards.len(), 13);
    }

    #[test]
    fn rejects_everything_else_without_changing_state() {
        let mut table = stub(1);
        let before = table.piles();
        let other = Move::Transfer {
            from: PileRef::Waste,
            to: PileRef::Tableau(0),
            count: 1,
        };
        assert_eq!(table.apply(other), Err(MoveError::Unsupported));
        assert_eq!(table.piles(), before);
        assert_eq!(table.hint(), Some(Move::Stock));
        assert!(!table.is_won());
    }
}
