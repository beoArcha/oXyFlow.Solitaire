//! Pyramid: twenty-eight cards in seven rows, a stock drawn to the waste and a single foundation.
//! Pairs of available cards summing to 13 and available kings leave the table. The two rule sets
//! differ only in the number of passes through the stock and in the win condition. The module
//! follows the shared layout of the games: state and deal here, validation and execution in
//! `rules`, move generation and the hint in `moves`.

mod moves;
mod rules;
#[cfg(test)]
mod tests;

use crate::card::{Card, Rank};
use crate::deck;
use crate::game::{Game, PyramidRules};
use crate::moves::{Move, MoveError};
use crate::pile::{PileRef, PileSnapshot, TableCard};

const ROWS: usize = 7;
const DECK_SIZE: usize = 52;
const TARGET_SUM: u8 = 13;
const RELAXED_PASSES: u8 = 3;
const CLASSIC_PASSES: u8 = 1;

/// A Pyramid game. `rows[r][i]` is the pyramid card at row *r*, index *i*; `None` once removed.
/// The top card of the stock and of the waste is the last element of its vector.
#[derive(Debug, Clone)]
pub(crate) struct Pyramid {
    rules: PyramidRules,
    rows: [Vec<Option<Card>>; ROWS],
    stock: Vec<Card>,
    waste: Vec<Card>,
    foundation: Vec<Card>,
    recycles_left: u8,
}

impl Pyramid {
    /// Deals a shuffled deck from `seed`: row *r* takes the next *r*+1 cards, the remaining 24
    /// form the stock.
    pub(crate) fn deal(rules: PyramidRules, seed: u64) -> Self {
        let mut cards = deck::standard();
        deck::shuffle_with_seed(&mut cards, seed);
        let mut rest = cards.into_iter();
        let rows = std::array::from_fn(|row| rest.by_ref().take(row + 1).map(Some).collect());
        Self {
            rules,
            rows,
            stock: rest.collect(),
            waste: Vec::new(),
            foundation: Vec::new(),
            recycles_left: passes(rules).saturating_sub(1),
        }
    }

    fn pyramid_is_empty(&self) -> bool {
        self.rows.iter().flatten().all(Option::is_none)
    }

    /// The card still at the position, if any.
    fn slot(&self, row: usize, index: usize) -> Option<Card> {
        self.rows
            .get(row)
            .and_then(|slots| slots.get(index))
            .copied()
            .flatten()
    }

    /// A card covers `(row, index)` while either card of the row below is still in place.
    fn covered(&self, row: usize, index: usize) -> bool {
        self.rows.get(row + 1).is_some_and(|below| {
            [index, index + 1]
                .iter()
                .any(|&i| below.get(i).is_some_and(|slot| slot.is_some()))
        })
    }
}

/// Passes through the stock allowed by a rule set, the first traversal included.
fn passes(rules: PyramidRules) -> u8 {
    match rules {
        PyramidRules::Relaxed => RELAXED_PASSES,
        PyramidRules::Classic => CLASSIC_PASSES,
    }
}

/// The value of a card for pairing: ace 1, two to ten at face value, jack 11, queen 12, king 13.
pub(super) fn value(card: Card) -> u8 {
    match card.rank {
        Rank::Ace => 1,
        Rank::Jack => 11,
        Rank::Queen => 12,
        Rank::King => 13,
        rank => rank.index() + 1,
    }
}

fn small(n: usize) -> u8 {
    u8::try_from(n).unwrap_or(u8::MAX)
}

fn face_up(card: Card) -> TableCard {
    TableCard {
        card,
        face_up: true,
    }
}

impl Game for Pyramid {
    fn apply(&mut self, mv: Move) -> Result<(), MoveError> {
        self.execute(mv)
    }

    fn legal_moves(&self) -> Vec<Move> {
        self.enumerate_moves()
    }

    fn hint(&self) -> Option<Move> {
        self.suggest()
    }

    fn is_won(&self) -> bool {
        match self.rules {
            PyramidRules::Relaxed => self.pyramid_is_empty(),
            PyramidRules::Classic => self.foundation.len() == DECK_SIZE,
        }
    }

    /// Stock, waste, the foundation, then all 28 pyramid positions row by row. A removed position
    /// is listed with no cards; the stock lists only its top card as face up.
    fn piles(&self) -> Vec<PileSnapshot> {
        let stock_top = self.stock.len().saturating_sub(1);
        let stock = self
            .stock
            .iter()
            .enumerate()
            .map(|(i, &card)| TableCard {
                card,
                face_up: i == stock_top,
            })
            .collect();
        let mut piles = vec![
            PileSnapshot {
                pile: PileRef::Stock,
                cards: stock,
            },
            PileSnapshot {
                pile: PileRef::Waste,
                cards: self.waste.iter().copied().map(face_up).collect(),
            },
            PileSnapshot {
                pile: PileRef::Foundation(0),
                cards: self.foundation.iter().copied().map(face_up).collect(),
            },
        ];
        for (row, slots) in self.rows.iter().enumerate() {
            for (index, slot) in slots.iter().enumerate() {
                piles.push(PileSnapshot {
                    pile: PileRef::Pyramid {
                        row: small(row),
                        index: small(index),
                    },
                    cards: slot.iter().copied().map(face_up).collect(),
                });
            }
        }
        piles
    }
}
