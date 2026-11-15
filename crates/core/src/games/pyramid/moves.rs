//! Move generation and the hint of Pyramid.

use super::{Pyramid, ROWS, TARGET_SUM, small, value};
use crate::card::{Card, Rank};
use crate::moves::Move;
use crate::pile::PileRef;

/// Hint priorities, lowest first: a king, a pair of two pyramid cards, a pyramid card with a stock
/// or waste card, a stock and waste pair, and finally the stock move.
fn hint_rank(mv: Move) -> u8 {
    match mv {
        Move::Remove { second: None, .. } => 0,
        Move::Remove {
            first,
            second: Some(second),
        } => match (is_pyramid(first), is_pyramid(second)) {
            (true, true) => 1,
            (false, false) => 3,
            _ => 2,
        },
        Move::Stock | Move::Transfer { .. } => 4,
    }
}

fn is_pyramid(pile: PileRef) -> bool {
    matches!(pile, PileRef::Pyramid { .. })
}

impl Pyramid {
    /// The cards that may leave the table, in the canonical order: stock top, waste top, then the
    /// uncovered pyramid cards row by row.
    fn available(&self) -> Vec<(PileRef, Card)> {
        let stock = self.stock.last().map(|&card| (PileRef::Stock, card));
        let waste = self.waste.last().map(|&card| (PileRef::Waste, card));
        let pyramid = (0..ROWS)
            .flat_map(|row| (0..=row).map(move |index| (row, index)))
            .filter(|&(row, index)| !self.covered(row, index))
            .filter_map(|(row, index)| {
                let pile = PileRef::Pyramid {
                    row: small(row),
                    index: small(index),
                };
                self.slot(row, index).map(|card| (pile, card))
            });
        stock.into_iter().chain(waste).chain(pyramid).collect()
    }

    /// Every move [`Pyramid::execute`] accepts, with each pair listed once: the first card is the
    /// one earlier in [`Pyramid::available`]. `apply` also accepts the reversed order.
    pub(super) fn enumerate_moves(&self) -> Vec<Move> {
        let available = self.available();
        let mut moves = Vec::new();
        if self.draw_is_legal() {
            moves.push(Move::Stock);
        }
        for (i, &(first, a)) in available.iter().enumerate() {
            if a.rank == Rank::King {
                moves.push(Move::Remove {
                    first,
                    second: None,
                });
            }
            for &(second, b) in available.iter().skip(i + 1) {
                if value(a) + value(b) == TARGET_SUM {
                    moves.push(Move::Remove {
                        first,
                        second: Some(second),
                    });
                }
            }
        }
        moves
    }

    /// The best legal move by [`hint_rank`]; ties keep the order of [`Pyramid::enumerate_moves`].
    pub(super) fn suggest(&self) -> Option<Move> {
        self.enumerate_moves()
            .into_iter()
            .min_by_key(|&mv| hint_rank(mv))
    }
}
