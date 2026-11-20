//! The assist counters of the running game (P4-06): which moves the player made and which the
//! automatic moves made. Information only; nothing here ever refuses a move.

use super::BoardScreen;
use crate::storage::AssistCounters;
use crate::storage::counters::MoveSource;

/// The counters and the running streak of quick moves, which is not stored.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct AssistTally {
    counters: AssistCounters,
    streak: u32,
}

impl AssistTally {
    pub(super) fn from_stored(counters: AssistCounters) -> Self {
        Self {
            counters,
            streak: 0,
        }
    }

    fn count(&mut self, source: MoveSource) {
        self.counters.count(source, &mut self.streak);
    }

    fn count_failed(&mut self) {
        self.counters.count_failed();
    }
}

impl BoardScreen {
    /// The assist counters of the running game, which the application stores with its history row.
    pub fn assist(&self) -> AssistCounters {
        self.assist.counters
    }

    /// Sets the counters of a game taken over from the saved one; the streak starts again.
    pub fn set_assist(&mut self, counters: AssistCounters) {
        self.assist = AssistTally::from_stored(counters);
    }

    pub(super) fn count_move(&mut self, source: MoveSource) {
        self.assist.count(source);
    }

    pub(super) fn count_failed_quick(&mut self) {
        self.assist.count_failed();
        self.dirty = true;
    }
}
