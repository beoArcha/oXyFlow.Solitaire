//! TriPeaks: 28 cards dealt into three overlapping peaks (rows of 3, 6, 9, and 10), a stock,
//! and one waste pile. Only the bottom row starts face up; a covered card turns face up when both
//! cards covering it are gone. An uncovered peak card may be played onto the waste when its rank
//! is one higher or one lower than the waste top, wrapping between king and ace. The stock turns
//! one card onto the waste with no redeal. The game is won when all 28 peak cards are played.

mod finish;
mod moves;
mod rules;
#[cfg(test)]
mod tests;

use crate::card::Card;
use crate::deck;
use crate::game::{Game, StockStatus};
use crate::games::index_ref;
use crate::games::progress::{self, HintProfile};
use crate::moves::{Move, MoveError};
use crate::pile::{PileRef, PileSnapshot, TableCard};

pub(crate) const ROWS: usize = 4;
pub(crate) const ROW_LENS: [usize; ROWS] = [3, 6, 9, 10];
pub(crate) const POSITIONS: usize = 28;
const RANKS: u8 = 13;

/// Whether a king and an ace count as adjacent ranks. Traditional TriPeaks: they do.
pub(crate) const KING_ACE_ADJACENT: bool = true;

/// Whether two ranks, given by [`crate::Rank::index`], are one apart under the TriPeaks rule.
pub(crate) fn ranks_adjacent(a: u8, b: u8) -> bool {
    let apart = a.abs_diff(b);
    apart == 1 || (KING_ACE_ADJACENT && apart == RANKS - 1)
}

/// Whether `(row, index)` is a valid peak slot.
pub(crate) fn slot_exists(row: usize, index: usize) -> bool {
    ROW_LENS.get(row).is_some_and(|&len| index < len)
}

/// The two indices in `row + 1` that cover `(row, index)`, or `None` for the bottom row.
pub(crate) const fn children_of(row: usize, index: usize) -> Option<[usize; 2]> {
    match row {
        0 => Some([2 * index, 2 * index + 1]),
        1 => {
            let left = index + index / 2;
            Some([left, left + 1])
        }
        2 => Some([index, index + 1]),
        _ => None,
    }
}

/// A TriPeaks game. `rows[r][i]` holds the card at row *r*, index *i*, or `None` once played.
/// The top card of the stock and of the waste is the last element of its vector.
#[derive(Debug, Clone)]
pub(crate) struct TriPeaks {
    rows: [Vec<Option<TableCard>>; ROWS],
    stock: Vec<Card>,
    waste: Vec<Card>,
}

impl TriPeaks {
    /// Deals a shuffled deck from `seed`: 28 cards to the three peaks (only row 3 face up), the
    /// next card to the waste, and the remaining 23 to the stock.
    pub(crate) fn deal(seed: u64) -> Self {
        let mut cards = deck::standard();
        deck::shuffle_with_seed(&mut cards, seed);
        let mut rest = cards.into_iter();
        let rows = std::array::from_fn(|row| {
            let face_up = row + 1 == ROWS;
            rest.by_ref()
                .take(ROW_LENS[row])
                .map(|card| Some(TableCard { card, face_up }))
                .collect()
        });
        let waste = rest.by_ref().take(1).collect();
        Self {
            rows,
            stock: rest.collect(),
            waste,
        }
    }

    fn peaks_empty(&self) -> bool {
        self.rows.iter().flatten().all(Option::is_none)
    }

    fn slot(&self, row: usize, index: usize) -> Option<TableCard> {
        self.rows
            .get(row)
            .and_then(|slots| slots.get(index))
            .copied()
            .flatten()
    }

    fn covered(&self, row: usize, index: usize) -> bool {
        let Some(children) = children_of(row, index) else {
            return false;
        };
        let Some(below) = self.rows.get(row + 1) else {
            return false;
        };
        children
            .iter()
            .any(|&child| below.get(child).is_some_and(Option::is_some))
    }
}

fn face_down(card: Card) -> TableCard {
    TableCard {
        card,
        face_up: false,
    }
}

fn face_up(card: Card) -> TableCard {
    TableCard {
        card,
        face_up: true,
    }
}

const HINT_PROFILE: HintProfile = HintProfile::Rules;

impl Game for TriPeaks {
    fn apply(&mut self, mv: Move) -> Result<(), MoveError> {
        if self.is_won() {
            return Err(MoveError::Unsupported);
        }
        self.execute(mv)
    }

    fn legal_moves(&self) -> Vec<Move> {
        if self.is_won() {
            return Vec::new();
        }
        self.enumerate_moves()
    }

    fn hint(&self) -> Option<Move> {
        if self.is_won() {
            return None;
        }
        progress::hint(self, HINT_PROFILE, progress::PLAY_KNOWLEDGE, || {
            self.suggest()
        })
    }

    fn is_won(&self) -> bool {
        self.peaks_empty()
    }

    /// Stock, waste, then all 28 peak slots row by row from top to bottom.
    fn piles(&self) -> Vec<PileSnapshot> {
        let mut piles = vec![
            PileSnapshot {
                pile: PileRef::Stock,
                cards: self.stock.iter().copied().map(face_down).collect(),
            },
            PileSnapshot {
                pile: PileRef::Waste,
                cards: self.waste.iter().copied().map(face_up).collect(),
            },
        ];
        for (row, slots) in self.rows.iter().enumerate() {
            for (index, slot) in slots.iter().enumerate() {
                piles.push(PileSnapshot {
                    pile: PileRef::Pyramid {
                        row: index_ref(row),
                        index: index_ref(index),
                    },
                    cards: slot.iter().copied().collect(),
                });
            }
        }
        piles
    }

    fn stock_status(&self) -> Option<StockStatus> {
        Some(StockStatus {
            recycles: 0,
            recycles_left: Some(0),
            deals_left: None,
        })
    }

    fn forced_finish(&self) -> Option<Vec<Move>> {
        self.finishing_moves()
    }

    fn is_covered(&self, pile: PileRef) -> bool {
        match pile {
            PileRef::Pyramid { row, index } => {
                let (row, index) = (usize::from(row), usize::from(index));
                self.slot(row, index).is_some() && self.covered(row, index)
            }
            _ => false,
        }
    }
}
