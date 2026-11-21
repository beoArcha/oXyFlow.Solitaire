//! Validation and execution of Golf moves: the stock turn and the play of a column card onto the
//! waste.

use super::{COLUMNS, Golf, ranks_adjacent};
use crate::card::Card;
use crate::moves::{Move, MoveError};
use crate::pile::PileRef;

impl Golf {
    pub(super) fn execute(&mut self, mv: Move) -> Result<(), MoveError> {
        match mv {
            Move::Stock => self.draw(),
            Move::Transfer { from, to, count } => self.play(from, to, count),
            Move::Remove { .. } => Err(MoveError::Unsupported),
        }
    }

    /// Turns the top stock card onto the waste. The waste is never turned back: one pass only.
    fn draw(&mut self) -> Result<(), MoveError> {
        let card = self.stock.pop().ok_or(MoveError::StockEmpty)?;
        self.waste.push(card);
        Ok(())
    }

    fn exists(pile: PileRef) -> bool {
        match pile {
            PileRef::Stock | PileRef::Waste => true,
            PileRef::Tableau(i) => usize::from(i) < COLUMNS,
            PileRef::Foundation(_) | PileRef::Cell(_) | PileRef::Pyramid { .. } => false,
        }
    }

    pub(super) fn column(&self, pile: PileRef) -> Option<&Vec<Card>> {
        match pile {
            PileRef::Tableau(i) => self.columns.get(usize::from(i)),
            _ => None,
        }
    }

    /// The top card of the column `from` when it may be played onto the waste: the waste top is
    /// one rank above or below it.
    fn playable(&self, from: PileRef) -> Option<Card> {
        let card = self.column(from)?.last().copied()?;
        let waste = self.waste.last()?;
        ranks_adjacent(card.rank.index(), waste.rank.index()).then_some(card)
    }

    pub(super) fn can_play(&self, from: PileRef) -> bool {
        self.playable(from).is_some()
    }

    /// Why the top `count` cards of `from` cannot go onto `to`, or `Ok` when the move is legal.
    ///
    /// Errors, in order: a missing pile is `NoSuchPile`; a source that is not a column is
    /// `Unsupported`; a target other than the waste is `TargetRejects`; an empty column is
    /// `EmptySource`; a count other than 1 is `CountOutOfRange`; a card that is not adjacent to
    /// the waste top is `TargetRejects`.
    pub(super) fn check_play(
        &self,
        from: PileRef,
        to: PileRef,
        count: u8,
    ) -> Result<(), MoveError> {
        for pile in [from, to] {
            if !Self::exists(pile) {
                return Err(MoveError::NoSuchPile(pile));
            }
        }
        let column = self.column(from).ok_or(MoveError::Unsupported)?;
        if to != PileRef::Waste {
            return Err(MoveError::TargetRejects);
        }
        if column.is_empty() {
            return Err(MoveError::EmptySource(from));
        }
        if count != 1 {
            return Err(MoveError::CountOutOfRange { count });
        }
        self.playable(from)
            .map(drop)
            .ok_or(MoveError::TargetRejects)
    }

    fn play(&mut self, from: PileRef, to: PileRef, count: u8) -> Result<(), MoveError> {
        self.check_play(from, to, count)?;
        let PileRef::Tableau(i) = from else {
            return Err(MoveError::Unsupported);
        };
        let card = self
            .columns
            .get_mut(usize::from(i))
            .and_then(Vec::pop)
            .ok_or(MoveError::EmptySource(from))?;
        self.waste.push(card);
        Ok(())
    }
}
