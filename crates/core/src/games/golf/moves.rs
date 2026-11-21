//! Move generation and the hint of Golf.

use super::{COLUMNS, Golf, index_ref};
use crate::moves::Move;
use crate::pile::PileRef;

pub(super) fn columns() -> impl Iterator<Item = PileRef> {
    (0..COLUMNS).map(|i| PileRef::Tableau(index_ref(i)))
}

pub(super) fn play(from: PileRef) -> Move {
    Move::Transfer {
        from,
        to: PileRef::Waste,
        count: 1,
    }
}

impl Golf {
    /// Every move [`Golf::execute`] accepts: the stock turn, then each column whose top card
    /// goes onto the waste.
    pub(super) fn enumerate_moves(&self) -> Vec<Move> {
        let stock = (!self.stock.is_empty()).then_some(Move::Stock);
        let plays = columns().filter(|&from| self.can_play(from)).map(play);
        stock.into_iter().chain(plays).collect()
    }

    /// The play from the column with the most cards left (the lowest column on a tie), because
    /// a tall column is the hardest to clear; the stock turn only when no column card fits.
    /// `None` when neither exists.
    pub(super) fn suggest(&self) -> Option<Move> {
        let tallest = columns().filter(|&from| self.can_play(from)).fold(
            None::<(usize, PileRef)>,
            |best, from| {
                let len = self.column(from).map_or(0, Vec::len);
                match best {
                    Some((most, _)) if most >= len => best,
                    _ => Some((len, from)),
                }
            },
        );
        tallest
            .map(|(_, from)| play(from))
            .or_else(|| (!self.stock.is_empty()).then_some(Move::Stock))
    }
}
