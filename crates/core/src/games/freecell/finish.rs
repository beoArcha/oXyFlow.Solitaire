//! The forced win of FreeCell.

use super::FreeCell;
use crate::games::finish::{first_to_foundation, greedy_finish};
use crate::moves::Move;

impl FreeCell {
    /// Moves cards from the cells and the column tops to the foundations as long as one fits. Such
    /// moves never block each other, so the order does not matter: the sequence wins exactly when
    /// this closure empties the table. A position that needs any other move (a card to a cell or
    /// another column to reach a buried card) gets `None`, even if it is winnable.
    pub(super) fn finishing_moves(&self) -> Option<Vec<Move>> {
        greedy_finish(self, |game| first_to_foundation(&game.enumerate_moves()))
    }
}
