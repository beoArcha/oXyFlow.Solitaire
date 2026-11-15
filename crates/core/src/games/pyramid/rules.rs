//! Validation and execution of Pyramid moves: the stock and the removal of a king or of a pair of
//! available cards summing to 13.

use super::{Pyramid, ROWS, TARGET_SUM, value};
use crate::card::{Card, Rank};
use crate::moves::{Move, MoveError};
use crate::pile::PileRef;

impl Pyramid {
    pub(super) fn execute(&mut self, mv: Move) -> Result<(), MoveError> {
        match mv {
            Move::Stock => self.draw_or_recycle(),
            Move::Remove { first, second } => self.remove(first, second),
            Move::Transfer { .. } => Err(MoveError::Unsupported),
        }
    }

    /// Whether [`Move::Stock`] is accepted: the stock has a card, or the waste can go back to it.
    pub(super) fn draw_is_legal(&self) -> bool {
        !self.stock.is_empty() || self.recycle_is_allowed()
    }

    fn recycle_is_allowed(&self) -> bool {
        !self.waste.is_empty() && self.recycles_left > 0
    }

    /// Moves the top stock card to the waste. An empty stock takes the waste back, reversed, so
    /// the next pass deals the cards in the original order; after the last pass it is `StockEmpty`.
    fn draw_or_recycle(&mut self) -> Result<(), MoveError> {
        if let Some(card) = self.stock.pop() {
            self.waste.push(card);
            return Ok(());
        }
        if !self.recycle_is_allowed() {
            return Err(MoveError::StockEmpty);
        }
        self.stock = std::mem::take(&mut self.waste);
        self.stock.reverse();
        self.recycles_left = self.recycles_left.saturating_sub(1);
        Ok(())
    }

    /// Whether `pile` names an existing place: the stock, the waste, the foundation, or a position
    /// of the pyramid. Other piles belong to other games.
    pub(super) fn exists(&self, pile: PileRef) -> bool {
        match pile {
            PileRef::Stock | PileRef::Waste => true,
            PileRef::Foundation(i) => i == 0,
            PileRef::Pyramid { row, index } => {
                usize::from(row) < ROWS && usize::from(index) <= usize::from(row)
            }
            PileRef::Tableau(_) | PileRef::Cell(_) => false,
        }
    }

    /// The card that `pile` offers to a removal: the stock or waste top, or an uncovered pyramid
    /// card. The foundation is never a source.
    pub(super) fn playable(&self, pile: PileRef) -> Result<Card, MoveError> {
        if !self.exists(pile) {
            return Err(MoveError::NoSuchPile(pile));
        }
        match pile {
            PileRef::Stock => self
                .stock
                .last()
                .copied()
                .ok_or(MoveError::EmptySource(pile)),
            PileRef::Waste => self
                .waste
                .last()
                .copied()
                .ok_or(MoveError::EmptySource(pile)),
            PileRef::Pyramid { row, index } => {
                let (row, index) = (usize::from(row), usize::from(index));
                let card = self.slot(row, index).ok_or(MoveError::EmptySource(pile))?;
                if self.covered(row, index) {
                    Err(MoveError::Blocked)
                } else {
                    Ok(card)
                }
            }
            PileRef::Foundation(_) | PileRef::Tableau(_) | PileRef::Cell(_) => {
                Err(MoveError::Unsupported)
            }
        }
    }

    /// Removes a king alone, or two cards summing to 13, onto the foundation. Both cards are
    /// checked before anything changes; a card never pairs with itself because 13 is odd.
    fn remove(&mut self, first: PileRef, second: Option<PileRef>) -> Result<(), MoveError> {
        let taken = self.removal(first, second)?;
        for (pile, card) in taken {
            self.discard(pile);
            self.foundation.push(card);
        }
        Ok(())
    }

    fn removal(
        &self,
        first: PileRef,
        second: Option<PileRef>,
    ) -> Result<Vec<(PileRef, Card)>, MoveError> {
        let a = self.playable(first)?;
        let Some(second) = second else {
            return if a.rank == Rank::King {
                Ok(vec![(first, a)])
            } else {
                Err(MoveError::NotAPair)
            };
        };
        let b = self.playable(second)?;
        if value(a) + value(b) == TARGET_SUM {
            Ok(vec![(first, a), (second, b)])
        } else {
            Err(MoveError::NotAPair)
        }
    }

    fn discard(&mut self, pile: PileRef) {
        match pile {
            PileRef::Stock => {
                self.stock.pop();
            }
            PileRef::Waste => {
                self.waste.pop();
            }
            PileRef::Pyramid { row, index } => {
                let slot = self
                    .rows
                    .get_mut(usize::from(row))
                    .and_then(|slots| slots.get_mut(usize::from(index)));
                if let Some(slot) = slot {
                    *slot = None;
                }
            }
            PileRef::Foundation(_) | PileRef::Tableau(_) | PileRef::Cell(_) => {}
        }
    }
}
