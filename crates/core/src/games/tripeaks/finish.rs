//! The forced win of TriPeaks.

use super::TriPeaks;
use super::moves::{play, positions};
use crate::game::Game;
use crate::games::finish::verified_finish;
use crate::moves::Move;

/// Positions the search may visit before it gives up.
const NODE_BUDGET: u32 = 5_000;

impl TriPeaks {
    /// A depth-first search over peak plays only that never turns a stock card. Stops after
    /// [`NODE_BUDGET`] positions and returns `None`.
    pub(super) fn finishing_moves(&self) -> Option<Vec<Move>> {
        let mut budget = NODE_BUDGET;
        let mut path = Vec::new();
        if !self.search(&mut budget, &mut path) {
            return None;
        }
        verified_finish(self, path)
    }

    fn search(&self, budget: &mut u32, path: &mut Vec<Move>) -> bool {
        if self.is_won() {
            return true;
        }
        for from in positions().filter(|&from| self.can_play(from)) {
            if *budget == 0 {
                return false;
            }
            *budget -= 1;
            let mut next = self.clone();
            let mv = play(from);
            if next.execute(mv).is_err() {
                continue;
            }
            path.push(mv);
            if next.search(budget, path) {
                return true;
            }
            path.pop();
        }
        false
    }
}
