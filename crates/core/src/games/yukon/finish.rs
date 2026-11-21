//! The forced win of Yukon.

use super::Yukon;
use crate::games::finish::{first_to_foundation, greedy_finish};
use crate::moves::Move;

impl Yukon {
    /// Only when no card is face down: the player then sees the whole table, and the sequence
    /// never turns over a card nobody has seen. Column tops go to the foundations as long as one
    /// fits; such moves never block each other, so the sequence wins exactly when this closure
    /// empties the table. A position that needs any column move gets `None`, even if it is
    /// winnable.
    pub(super) fn finishing_moves(&self) -> Option<Vec<Move>> {
        let all_open = self.columns.iter().all(|column| column.down.is_empty());
        if !all_open {
            return None;
        }
        greedy_finish(self, |game| first_to_foundation(&game.enumerate_moves()))
    }
}
