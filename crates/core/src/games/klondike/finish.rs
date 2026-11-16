//! The forced win of Klondike.

use super::Klondike;
use crate::games::finish::{first_to_foundation, greedy_finish};
use crate::moves::Move;

impl Klondike {
    /// Only when the stock and the waste are empty and no card is face down. Then every column is
    /// a descending run whose top is its lowest card, so the lowest card left on the table is
    /// always on top of a column and its foundation is ready: moving any card that fits a
    /// foundation, again and again, wins.
    pub(super) fn finishing_moves(&self) -> Option<Vec<Move>> {
        let all_open = self.stock.is_empty()
            && self.waste.is_empty()
            && self.columns.iter().all(|column| column.down.is_empty());
        if !all_open {
            return None;
        }
        greedy_finish(self, |game| first_to_foundation(&game.enumerate_moves()))
    }
}
