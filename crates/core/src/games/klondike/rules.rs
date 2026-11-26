//! Validation and execution of Klondike moves, including the automatic flip of a face-down card.

use super::{COLUMNS, Column, FOUNDATIONS, Klondike};
use crate::card::{Card, Rank};
use crate::game::DrawCount;
use crate::moves::{Move, MoveError};
use crate::pile::PileRef;
use crate::rules;

fn top_card(cards: &[Card]) -> &[Card] {
    cards
        .get(cards.len().saturating_sub(1)..)
        .unwrap_or_default()
}

impl Klondike {
    pub(super) fn execute(&mut self, mv: Move) -> Result<(), MoveError> {
        match mv {
            Move::Stock => self.draw_or_recycle(),
            Move::Transfer { from, to, count } => self.transfer(from, to, count),
            Move::Remove { .. } => Err(MoveError::Unsupported),
        }
    }

    fn draw_size(&self) -> usize {
        match self.draw {
            DrawCount::One => 1,
            DrawCount::Three => 3,
        }
    }

    /// Moves up to one draw of cards from the stock to the waste. An empty stock takes the whole
    /// waste back, reversed, so the next pass deals the cards in the original order.
    pub(super) fn draw_or_recycle(&mut self) -> Result<(), MoveError> {
        if self.stock.is_empty() {
            if self.waste.is_empty() {
                return Err(MoveError::StockEmpty);
            }
            if !self.recycle_allowed() {
                return Err(MoveError::PassesExhausted);
            }
            self.stock = std::mem::take(&mut self.waste);
            self.stock.reverse();
            self.recycles = self.recycles.saturating_add(1);
            return Ok(());
        }
        let from = self.stock.len().saturating_sub(self.draw_size());
        let mut drawn = self.stock.split_off(from);
        drawn.reverse();
        self.waste.extend(drawn);
        Ok(())
    }

    fn exists(&self, pile: PileRef) -> bool {
        match pile {
            PileRef::Stock | PileRef::Waste => true,
            PileRef::Foundation(i) => usize::from(i) < FOUNDATIONS,
            PileRef::Tableau(i) => usize::from(i) < COLUMNS,
            PileRef::Cell(_) | PileRef::Pyramid { .. } => false,
        }
    }

    pub(super) fn column(&self, pile: PileRef) -> Option<&Column> {
        match pile {
            PileRef::Tableau(i) => self.columns.get(usize::from(i)),
            _ => None,
        }
    }

    /// The cards of `pile` that may leave it: the face-up run of a column, the top card of the
    /// waste or of a foundation. The stock gives its cards only through [`Move::Stock`].
    pub(super) fn movable(&self, pile: PileRef) -> Result<&[Card], MoveError> {
        match pile {
            PileRef::Waste => Ok(top_card(&self.waste)),
            PileRef::Foundation(i) => self
                .foundations
                .get(usize::from(i))
                .map(|cards| top_card(cards))
                .ok_or(MoveError::NoSuchPile(pile)),
            PileRef::Tableau(_) => self
                .column(pile)
                .map(|column| column.up.as_slice())
                .ok_or(MoveError::NoSuchPile(pile)),
            PileRef::Stock => Err(MoveError::Unsupported),
            PileRef::Cell(_) | PileRef::Pyramid { .. } => Err(MoveError::NoSuchPile(pile)),
        }
    }

    /// Why `count` cards of `from` cannot go onto `to`, or `Ok` when the move is legal.
    ///
    /// Errors, in order: a missing pile is `NoSuchPile`; the stock as a source is `Unsupported`;
    /// an empty source is `EmptySource`; a count of 0, above the movable cards, or above 1 for a
    /// foundation target is `CountOutOfRange`; every other refusal is `TargetRejects`.
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
        self.accepts(from, to, run)
    }

    fn accepts(&self, from: PileRef, to: PileRef, run: &[Card]) -> Result<(), MoveError> {
        let [first, ..] = *run else {
            return Err(MoveError::TargetRejects);
        };
        let fits = match to {
            _ if to == from => false,
            PileRef::Foundation(i) => {
                if run.len() != 1 {
                    return Err(MoveError::CountOutOfRange {
                        count: u8::try_from(run.len()).unwrap_or(u8::MAX),
                    });
                }
                let pile = self.foundations.get(usize::from(i)).map(Vec::as_slice);
                !matches!(from, PileRef::Foundation(_))
                    && pile.is_some_and(|pile| rules::builds_foundation_up(pile, first))
            }
            PileRef::Tableau(_) => self
                .column(to)
                .is_some_and(|column| match column.up.last() {
                    Some(&top) => rules::fits_alternating(top, first),
                    None => self.any_empty || first.rank == Rank::King,
                }),
            _ => false,
        };
        if fits {
            Ok(())
        } else {
            Err(MoveError::TargetRejects)
        }
    }

    fn open_mut(&mut self, pile: PileRef) -> Option<&mut Vec<Card>> {
        match pile {
            PileRef::Waste => Some(&mut self.waste),
            PileRef::Foundation(i) => self.foundations.get_mut(usize::from(i)),
            PileRef::Tableau(i) => self.columns.get_mut(usize::from(i)).map(|c| &mut c.up),
            _ => None,
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
        self.flip_exposed(from);
        Ok(())
    }

    fn flip_exposed(&mut self, pile: PileRef) {
        let PileRef::Tableau(i) = pile else {
            return;
        };
        let Some(column) = self.columns.get_mut(usize::from(i)) else {
            return;
        };
        if column.up.is_empty() {
            column.up.extend(column.down.pop());
        }
    }
}
