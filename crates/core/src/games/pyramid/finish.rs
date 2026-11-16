//! The forced win of Pyramid.

use super::Pyramid;
use crate::game::Game;
use crate::games::finish::verified_finish;
use crate::moves::Move;

/// Positions the search may visit before it gives up.
const NODE_BUDGET: u32 = 5_000;

impl Pyramid {
    /// A depth-first search over removals only, in the order of [`Pyramid::enumerate_moves`], that
    /// never draws or recycles: so it finishes what the stock top, the waste top and the pyramid
    /// can clear by themselves, and nothing that needs another card out of the stock. The search
    /// stops after [`NODE_BUDGET`] positions and then answers `None`, so a winnable position can be
    /// missed but a sequence that is returned always wins. A fresh deal is not a forced win.
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
        for mv in self.enumerate_moves() {
            if mv == Move::Stock {
                continue;
            }
            if *budget == 0 {
                return false;
            }
            *budget -= 1;
            let mut next = self.clone();
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
