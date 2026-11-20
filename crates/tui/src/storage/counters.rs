//! The assist counters of one game (P4-06): how many moves the player made by hand and how many
//! the automatic moves made. They are information only and never block a move.

/// The share of automatic moves, in percent, above which a game counts as assisted.
pub const ASSISTED_ABOVE_PERCENT: u32 = 80;
/// Fewest moves decided by the player or the quick move before a game can count as assisted, so
/// that a game of two quick moves is not marked.
pub const ASSISTED_MIN_MOVES: u32 = 10;

/// Moves by their origin. The auto-finish is counted apart: the engine has already proven the win
/// there, so it is not part of the share.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AssistCounters {
    /// Moves the player made by hand.
    pub manual: u32,
    /// Moves made by the quick move key or a double click.
    pub quick: u32,
    /// Moves made by the auto-finish.
    pub finish: u32,
    /// Quick moves that found no target.
    pub failed: u32,
    /// The most quick moves made in a row.
    pub longest_streak: u32,
}

/// How one move came about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveSource {
    Manual,
    Quick,
    Finish,
}

impl AssistCounters {
    /// Counts one move; `streak` is the running count of quick moves in a row, which a manual move
    /// ends.
    pub fn count(&mut self, source: MoveSource, streak: &mut u32) {
        match source {
            MoveSource::Manual => {
                self.manual = self.manual.saturating_add(1);
                *streak = 0;
            }
            MoveSource::Quick => {
                self.quick = self.quick.saturating_add(1);
                *streak = streak.saturating_add(1);
                self.longest_streak = self.longest_streak.max(*streak);
            }
            MoveSource::Finish => self.finish = self.finish.saturating_add(1),
        }
    }

    /// Counts a quick move that found no target.
    pub fn count_failed(&mut self) {
        self.failed = self.failed.saturating_add(1);
    }

    /// Moves the player or the quick move decided, the base of the share.
    pub fn decided(&self) -> u32 {
        self.manual.saturating_add(self.quick)
    }

    /// Share of quick moves among the decided moves, rounded to a whole percent; `None` when no
    /// move was counted (a game from before the counters existed, or none made yet).
    pub fn share_percent(&self) -> Option<u32> {
        let decided = u64::from(self.decided());
        if decided == 0 {
            return None;
        }
        let share = (u64::from(self.quick) * 100 + decided / 2) / decided;
        Some(u32::try_from(share).unwrap_or(100))
    }

    /// Whether the quick moves are more than [`ASSISTED_ABOVE_PERCENT`] of at least
    /// [`ASSISTED_MIN_MOVES`] decided moves. The share is compared unrounded.
    pub fn is_assisted(&self) -> bool {
        let decided = u64::from(self.decided());
        decided >= u64::from(ASSISTED_MIN_MOVES)
            && u64::from(self.quick) * 100 > u64::from(ASSISTED_ABOVE_PERCENT) * decided
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn played(sources: &[MoveSource]) -> (AssistCounters, u32) {
        let mut counters = AssistCounters::default();
        let mut streak = 0;
        for source in sources {
            counters.count(*source, &mut streak);
        }
        (counters, streak)
    }

    #[test]
    fn each_source_has_its_own_counter() {
        let (counters, _) = played(&[
            MoveSource::Manual,
            MoveSource::Quick,
            MoveSource::Quick,
            MoveSource::Finish,
        ]);
        assert_eq!(
            (counters.manual, counters.quick, counters.finish),
            (1, 2, 1)
        );
        assert_eq!(counters.decided(), 3);
    }

    #[test]
    fn a_manual_move_ends_the_streak_and_the_longest_is_kept() {
        let mut sources = vec![MoveSource::Quick; 4];
        sources.push(MoveSource::Manual);
        sources.extend([MoveSource::Quick; 2]);
        let (counters, streak) = played(&sources);
        assert_eq!(counters.longest_streak, 4);
        assert_eq!(streak, 2);
    }

    #[test]
    fn the_finish_and_a_failed_attempt_do_not_end_the_streak() {
        let (mut counters, mut streak) = played(&[MoveSource::Quick, MoveSource::Quick]);
        counters.count(MoveSource::Finish, &mut streak);
        counters.count_failed();
        counters.count(MoveSource::Quick, &mut streak);
        assert_eq!((counters.longest_streak, counters.failed), (3, 1));
    }

    #[test]
    fn the_share_is_rounded_and_unknown_without_moves() {
        assert_eq!(AssistCounters::default().share_percent(), None);
        let (counters, _) = played(&[MoveSource::Quick, MoveSource::Manual, MoveSource::Manual]);
        assert_eq!(counters.share_percent(), Some(33));
    }

    #[test]
    fn a_game_is_assisted_only_above_the_threshold() {
        let with_share = |quick: u32, manual: u32| AssistCounters {
            quick,
            manual,
            ..AssistCounters::default()
        };
        let total = 100;
        let at = ASSISTED_ABOVE_PERCENT;
        assert!(!with_share(at, total - at).is_assisted());
        assert!(with_share(at + 1, total - at - 1).is_assisted());
        assert!(!with_share(ASSISTED_MIN_MOVES - 1, 0).is_assisted());
        assert!(with_share(ASSISTED_MIN_MOVES, 0).is_assisted());
    }

    #[test]
    fn the_unrounded_share_decides_not_the_rounded_one() {
        let counters = AssistCounters {
            quick: 800,
            manual: 201,
            ..AssistCounters::default()
        };
        assert_eq!(counters.share_percent(), Some(80));
        assert!(!counters.is_assisted());
    }
}
