//! Validation and execution of Spider moves, including the automatic collection of complete runs
//! and the flip of the face-down card that a removal exposes.

use super::{COLUMNS, Column, FOUNDATIONS, RUN_SIZE, Spider};
use crate::card::{Card, Rank};
use crate::moves::{Move, MoveError};
use crate::pile::PileRef;
use crate::rules;

/// Length of the longest same-suit descending tail of `up`: the cards that may move together.
pub(super) fn run_len(up: &[Card]) -> usize {
    let linked = up
        .windows(2)
        .rev()
        .take_while(|pair| rules::fits_same_suit(pair[0], pair[1]))
        .count();
    if up.is_empty() { 0 } else { linked + 1 }
}

fn ends_with_complete_run(up: &[Card]) -> bool {
    let Some(start) = up.len().checked_sub(RUN_SIZE) else {
        return false;
    };
    let run = up.get(start..).unwrap_or_default();
    run.first().is_some_and(|card| card.rank == Rank::King) && rules::descending_same_suit(run)
}

impl Spider {
    pub(super) fn execute(&mut self, mv: Move) -> Result<(), MoveError> {
        match mv {
            Move::Stock => self.deal_row(),
            Move::Transfer { from, to, count } => self.transfer(from, to, count),
            Move::Remove { .. } => Err(MoveError::Unsupported),
        }
    }

    /// Whether [`Move::Stock`] is allowed: the stock has cards and, unless `deal_empty` is on,
    /// every column holds one. Errors: an empty stock is `StockEmpty`, an empty column is
    /// `EmptyColumnBlocksDeal`.
    pub(super) fn check_deal(&self) -> Result<(), MoveError> {
        if self.stock.is_empty() {
            return Err(MoveError::StockEmpty);
        }
        if !self.deal_empty && self.columns.iter().any(|column| column.up.is_empty()) {
            return Err(MoveError::EmptyColumnBlocksDeal);
        }
        Ok(())
    }

    fn deal_row(&mut self) -> Result<(), MoveError> {
        self.check_deal()?;
        for column in &mut self.columns {
            column.up.extend(self.stock.pop());
        }
        for index in 0..COLUMNS {
            self.collect_run(index);
        }
        Ok(())
    }

    fn exists(&self, pile: PileRef) -> bool {
        match pile {
            PileRef::Stock => true,
            PileRef::Foundation(i) => usize::from(i) < FOUNDATIONS,
            PileRef::Tableau(i) => usize::from(i) < COLUMNS,
            PileRef::Waste | PileRef::Cell(_) | PileRef::Pyramid { .. } => false,
        }
    }

    pub(super) fn column(&self, pile: PileRef) -> Option<&Column> {
        match pile {
            PileRef::Tableau(i) => self.columns.get(usize::from(i)),
            _ => None,
        }
    }

    /// The face-up cards of a source column.
    ///
    /// Errors: a missing pile is `NoSuchPile`; the stock and the foundations never give cards away,
    /// so they are `Unsupported`; an empty column is `EmptySource`.
    pub(super) fn open_cards(&self, pile: PileRef) -> Result<&[Card], MoveError> {
        if !self.exists(pile) {
            return Err(MoveError::NoSuchPile(pile));
        }
        let column = self.column(pile).ok_or(MoveError::Unsupported)?;
        if column.up.is_empty() {
            return Err(MoveError::EmptySource(pile));
        }
        Ok(&column.up)
    }

    /// Why `count` cards of `from` cannot go onto `to`, or `Ok` when the move is legal.
    ///
    /// Errors, in order: a missing pile is `NoSuchPile`; a source that is not a column is
    /// `Unsupported`; an empty source is `EmptySource`; a count of 0 or above the face-up cards is
    /// `CountOutOfRange`; cards that are not a same-suit descending run are `InvalidSequence`;
    /// every other refusal is `TargetRejects`.
    pub(super) fn check_transfer(
        &self,
        from: PileRef,
        to: PileRef,
        count: u8,
    ) -> Result<(), MoveError> {
        for pile in [from, to] {
            if !self.exists(pile) {
                return Err(MoveError::NoSuchPile(pile));
            }
        }
        let open = self.open_cards(from)?;
        let run = open
            .len()
            .checked_sub(usize::from(count))
            .and_then(|at| open.get(at..))
            .filter(|run| !run.is_empty())
            .ok_or(MoveError::CountOutOfRange { count })?;
        if !rules::descending_same_suit(run) {
            return Err(MoveError::InvalidSequence);
        }
        self.accepts(from, to, run)
    }

    fn accepts(&self, from: PileRef, to: PileRef, run: &[Card]) -> Result<(), MoveError> {
        let [first, ..] = *run else {
            return Err(MoveError::TargetRejects);
        };
        let fits = to != from
            && self
                .column(to)
                .is_some_and(|column| match column.up.last() {
                    Some(&top) => rules::fits_any_suit(top, first),
                    None => true,
                });
        if fits {
            Ok(())
        } else {
            Err(MoveError::TargetRejects)
        }
    }

    fn transfer(&mut self, from: PileRef, to: PileRef, count: u8) -> Result<(), MoveError> {
        self.check_transfer(from, to, count)?;
        let (PileRef::Tableau(source), PileRef::Tableau(target)) = (from, to) else {
            return Err(MoveError::Unsupported);
        };
        let (source, target) = (usize::from(source), usize::from(target));
        let cards = self
            .columns
            .get_mut(source)
            .map_or_else(Vec::new, |column| {
                let at = column.up.len().saturating_sub(usize::from(count));
                column.up.split_off(at)
            });
        if let Some(column) = self.columns.get_mut(target) {
            column.up.extend(cards);
        }
        self.flip_exposed(source);
        self.collect_run(target);
        Ok(())
    }

    fn flip_exposed(&mut self, index: usize) {
        let Some(column) = self.columns.get_mut(index) else {
            return;
        };
        if column.up.is_empty() {
            column.up.extend(column.down.pop());
        }
    }

    /// Moves a complete king-to-ace run on top of column `index` to the first free foundation,
    /// then turns over the card it uncovered.
    fn collect_run(&mut self, index: usize) {
        let Some(column) = self.columns.get(index) else {
            return;
        };
        if !ends_with_complete_run(&column.up) {
            return;
        }
        let Some(foundation) = self.foundations.iter_mut().find(|cards| cards.is_empty()) else {
            return;
        };
        if let Some(column) = self.columns.get_mut(index) {
            let at = column.up.len().saturating_sub(RUN_SIZE);
            *foundation = column.up.split_off(at);
        }
        self.flip_exposed(index);
    }
}
