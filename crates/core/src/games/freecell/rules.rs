//! Validation and execution of FreeCell transfers, including the supermove capacity.

use super::{CELLS, COLUMNS, FOUNDATIONS, FreeCell, index_ref};
use crate::card::Card;
use crate::moves::{Move, MoveError};
use crate::pile::PileRef;
use crate::rules;

/// The longest descending, alternating-color run at the top of a column, as a suffix of it.
fn movable_run(column: &[Card]) -> &[Card] {
    let joined = column
        .windows(2)
        .rev()
        .take_while(
            |pair| matches!(pair, [upper, lower] if rules::fits_alternating(*upper, *lower)),
        )
        .count();
    let start = column.len().saturating_sub(joined + 1);
    column.get(start..).unwrap_or_default()
}

fn too_many(count: usize) -> MoveError {
    MoveError::CountOutOfRange {
        count: u8::try_from(count).unwrap_or(u8::MAX),
    }
}

impl FreeCell {
    pub(super) fn execute(&mut self, mv: Move) -> Result<(), MoveError> {
        match mv {
            Move::Transfer { from, to, count } => self.transfer(from, to, count),
            Move::Stock | Move::Remove { .. } => Err(MoveError::Unsupported),
        }
    }

    fn exists(pile: PileRef) -> bool {
        match pile {
            PileRef::Foundation(i) => usize::from(i) < FOUNDATIONS,
            PileRef::Cell(i) => usize::from(i) < CELLS,
            PileRef::Tableau(i) => usize::from(i) < COLUMNS,
            PileRef::Stock | PileRef::Waste | PileRef::Pyramid { .. } => false,
        }
    }

    /// The cards of `pile` that may leave it: the top card of a cell, the longest valid run at the
    /// top of a column. Foundations never give cards back.
    pub(super) fn movable(&self, pile: PileRef) -> Result<&[Card], MoveError> {
        match pile {
            PileRef::Cell(i) => self
                .cells
                .get(usize::from(i))
                .map(Vec::as_slice)
                .ok_or(MoveError::NoSuchPile(pile)),
            PileRef::Tableau(i) => self
                .columns
                .get(usize::from(i))
                .map(Vec::as_slice)
                .map(movable_run)
                .ok_or(MoveError::NoSuchPile(pile)),
            PileRef::Foundation(_) => Err(MoveError::Unsupported),
            PileRef::Stock | PileRef::Waste | PileRef::Pyramid { .. } => {
                Err(MoveError::NoSuchPile(pile))
            }
        }
    }

    /// The largest number of cards that can move onto `to` at once: `(free cells + 1) × 2^(empty
    /// columns)`, where an empty `to` does not count as one of the empty columns.
    pub(super) fn move_capacity(&self, to: PileRef) -> usize {
        let free_cells = self.cells.iter().filter(|cell| cell.is_empty()).count();
        let empty_columns = self
            .columns
            .iter()
            .enumerate()
            .filter(|(i, column)| column.is_empty() && PileRef::Tableau(index_ref(*i)) != to)
            .count();
        let doubling = u32::try_from(empty_columns)
            .ok()
            .and_then(|shift| 1usize.checked_shl(shift))
            .unwrap_or(usize::MAX);
        (free_cells + 1).saturating_mul(doubling)
    }

    /// Why `count` cards of `from` cannot go onto `to`, or `Ok` when the move is legal.
    ///
    /// Errors, in order: a missing pile is `NoSuchPile`; a foundation as source is `Unsupported`;
    /// an empty source is `EmptySource`; a count of 0 or above the movable cards is
    /// `CountOutOfRange`; a sequence the target refuses is `TargetRejects`; a sequence longer than
    /// the capacity is `NotEnoughSpace`.
    pub(super) fn check_transfer(
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
        let movable = self.movable(from)?;
        if movable.is_empty() {
            return Err(MoveError::EmptySource(from));
        }
        let run = movable
            .len()
            .checked_sub(usize::from(count))
            .and_then(|at| movable.get(at..))
            .filter(|run| !run.is_empty())
            .ok_or(MoveError::CountOutOfRange { count })?;
        self.accepts(from, to, run)?;
        if run.len() > self.move_capacity(to) {
            return Err(MoveError::NotEnoughSpace);
        }
        Ok(())
    }

    fn accepts(&self, from: PileRef, to: PileRef, run: &[Card]) -> Result<(), MoveError> {
        let [first, ..] = *run else {
            return Err(MoveError::TargetRejects);
        };
        let fits = match to {
            _ if to == from => false,
            PileRef::Foundation(i) => {
                if run.len() != 1 {
                    return Err(too_many(run.len()));
                }
                self.foundations
                    .get(usize::from(i))
                    .is_some_and(|pile| rules::builds_foundation_up(pile, first))
            }
            PileRef::Cell(i) => {
                if run.len() != 1 {
                    return Err(too_many(run.len()));
                }
                !matches!(from, PileRef::Cell(_))
                    && self
                        .cells
                        .get(usize::from(i))
                        .is_some_and(|cell| cell.is_empty())
            }
            PileRef::Tableau(i) => {
                self.columns
                    .get(usize::from(i))
                    .is_some_and(|column| match column.last() {
                        Some(&top) => rules::fits_alternating(top, first),
                        None => true,
                    })
            }
            PileRef::Stock | PileRef::Waste | PileRef::Pyramid { .. } => false,
        };
        if fits {
            Ok(())
        } else {
            Err(MoveError::TargetRejects)
        }
    }

    fn open_mut(&mut self, pile: PileRef) -> Option<&mut Vec<Card>> {
        match pile {
            PileRef::Foundation(i) => self.foundations.get_mut(usize::from(i)),
            PileRef::Cell(i) => self.cells.get_mut(usize::from(i)),
            PileRef::Tableau(i) => self.columns.get_mut(usize::from(i)),
            PileRef::Stock | PileRef::Waste | PileRef::Pyramid { .. } => None,
        }
    }

    fn transfer(&mut self, from: PileRef, to: PileRef, count: u8) -> Result<(), MoveError> {
        self.check_transfer(from, to, count)?;
        let cards = self.open_mut(from).map_or_else(Vec::new, |open| {
            let at = open.len().saturating_sub(usize::from(count));
            open.split_off(at)
        });
        if let Some(open) = self.open_mut(to) {
            open.extend(cards);
        }
        Ok(())
    }
}
