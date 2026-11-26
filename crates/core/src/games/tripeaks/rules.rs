//! Validation and execution of TriPeaks moves: the stock turn and the play of an uncovered
//! peak card onto the waste.

use super::{ROW_LENS, TriPeaks, ranks_adjacent, slot_exists};
use crate::card::Card;
use crate::moves::{Move, MoveError};
use crate::pile::PileRef;

impl TriPeaks {
    pub(super) fn execute(&mut self, mv: Move) -> Result<(), MoveError> {
        match mv {
            Move::Stock => self.draw(),
            Move::Transfer { from, to, count } => self.play(from, to, count),
            Move::Remove { .. } => Err(MoveError::Unsupported),
        }
    }

    fn draw(&mut self) -> Result<(), MoveError> {
        let card = self.stock.pop().ok_or(MoveError::StockEmpty)?;
        self.waste.push(card);
        Ok(())
    }

    fn exists(pile: PileRef) -> bool {
        match pile {
            PileRef::Stock | PileRef::Waste => true,
            PileRef::Pyramid { row, index } => slot_exists(usize::from(row), usize::from(index)),
            PileRef::Foundation(_) | PileRef::Tableau(_) | PileRef::Cell(_) => false,
        }
    }

    fn playable(&self, from: PileRef) -> Result<Card, MoveError> {
        match from {
            PileRef::Pyramid { row, index } => {
                let (row, index) = (usize::from(row), usize::from(index));
                let slot = self.slot(row, index).ok_or(MoveError::EmptySource(from))?;
                if self.covered(row, index) {
                    return Err(MoveError::Blocked);
                }
                if !slot.face_up {
                    return Err(MoveError::FaceDown);
                }
                Ok(slot.card)
            }
            _ => Err(MoveError::Unsupported),
        }
    }

    pub(super) fn can_play(&self, from: PileRef) -> bool {
        self.check_play(from, PileRef::Waste, 1).is_ok()
    }

    /// Why `count` cards from `from` cannot go onto `to`, or `Ok` when the move is legal.
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
        if !matches!(from, PileRef::Pyramid { .. }) {
            return Err(MoveError::Unsupported);
        }
        if to != PileRef::Waste {
            return Err(MoveError::TargetRejects);
        }
        let card = self.playable(from)?;
        if count != 1 {
            return Err(MoveError::CountOutOfRange { count });
        }
        let waste = self.waste.last().ok_or(MoveError::TargetRejects)?;
        ranks_adjacent(card.rank.index(), waste.rank.index())
            .then_some(())
            .ok_or(MoveError::TargetRejects)
    }

    fn play(&mut self, from: PileRef, to: PileRef, count: u8) -> Result<(), MoveError> {
        self.check_play(from, to, count)?;
        let PileRef::Pyramid { row, index } = from else {
            return Err(MoveError::Unsupported);
        };
        let (row, index) = (usize::from(row), usize::from(index));
        let slot = self
            .rows
            .get_mut(row)
            .and_then(|slots| slots.get_mut(index))
            .and_then(Option::take)
            .ok_or(MoveError::EmptySource(from))?;
        self.waste.push(slot.card);
        self.flip_uncovered(row);
        Ok(())
    }

    fn flip_uncovered(&mut self, removed_row: usize) {
        let Some(parent_row) = removed_row.checked_sub(1) else {
            return;
        };
        for index in 0..ROW_LENS[parent_row] {
            if !self.covered(parent_row, index)
                && let Some(slot) = self.rows[parent_row]
                    .get_mut(index)
                    .and_then(Option::as_mut)
            {
                slot.face_up = true;
            }
        }
    }
}
