//! Validation and execution of Yukon moves, including the automatic flip of a face-down card.
//! Which card fits onto a column comes from the rule set's [`super::setup::BuildRule`].

use super::{COLUMNS, Column, FOUNDATIONS, Yukon};
use crate::card::{Card, Rank};
use crate::moves::{Move, MoveError};
use crate::pile::PileRef;
use crate::rules;

fn top_card(cards: &[Card]) -> &[Card] {
    cards
        .get(cards.len().saturating_sub(1)..)
        .unwrap_or_default()
}

impl Yukon {
    pub(super) fn execute(&mut self, mv: Move) -> Result<(), MoveError> {
        match mv {
            Move::Transfer { from, to, count } => self.transfer(from, to, count),
            Move::Stock | Move::Remove { .. } => Err(MoveError::Unsupported),
        }
    }

    fn exists(pile: PileRef) -> bool {
        match pile {
            PileRef::Foundation(i) => usize::from(i) < FOUNDATIONS,
            PileRef::Tableau(i) => usize::from(i) < COLUMNS,
            PileRef::Stock | PileRef::Waste | PileRef::Cell(_) | PileRef::Pyramid { .. } => false,
        }
    }

    pub(super) fn column(&self, pile: PileRef) -> Option<&Column> {
        match pile {
            PileRef::Tableau(i) => self.columns.get(usize::from(i)),
            _ => None,
        }
    }

    /// The cards of `pile` that may leave it: every face-up card of a column (a group moves with
    /// all the cards above its bottom card), the top card of a foundation.
    pub(super) fn movable(&self, pile: PileRef) -> Result<&[Card], MoveError> {
        match pile {
            PileRef::Foundation(i) => self
                .foundations
                .get(usize::from(i))
                .map(|cards| top_card(cards))
                .ok_or(MoveError::NoSuchPile(pile)),
            PileRef::Tableau(_) => self
                .column(pile)
                .map(|column| column.up.as_slice())
                .ok_or(MoveError::NoSuchPile(pile)),
            _ => Err(MoveError::NoSuchPile(pile)),
        }
    }

    /// Why the top `count` cards of `from` cannot go onto `to`, or `Ok` when the move is legal.
    ///
    /// Errors, in order: a missing pile is `NoSuchPile`; an empty source is `EmptySource`; a count
    /// of 0 or above the face-up cards is `CountOutOfRange`, and so is more than one card for a
    /// foundation; every other refusal is `TargetRejects`.
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
        let group = movable
            .len()
            .checked_sub(usize::from(count))
            .and_then(|at| movable.get(at..))
            .filter(|group| !group.is_empty())
            .ok_or(MoveError::CountOutOfRange { count })?;
        self.accepts(from, to, group)
    }

    fn accepts(&self, from: PileRef, to: PileRef, group: &[Card]) -> Result<(), MoveError> {
        let [bottom, ..] = *group else {
            return Err(MoveError::TargetRejects);
        };
        let fits = match to {
            _ if to == from => false,
            PileRef::Foundation(i) => {
                if group.len() != 1 {
                    return Err(MoveError::CountOutOfRange {
                        count: u8::try_from(group.len()).unwrap_or(u8::MAX),
                    });
                }
                let pile = self.foundations.get(usize::from(i)).map(Vec::as_slice);
                !matches!(from, PileRef::Foundation(_))
                    && pile.is_some_and(|pile| rules::builds_foundation_up(pile, bottom))
            }
            PileRef::Tableau(_) => self
                .column(to)
                .is_some_and(|column| match column.up.last() {
                    Some(&top) => self.rules.build().fits(top, bottom),
                    None => bottom.rank == Rank::King,
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
