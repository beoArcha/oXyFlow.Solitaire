//! The steps of a TriPeaks position: the stock turn and the play of every free peak card that is
//! adjacent to the waste top, plays first.

use std::cmp::Reverse;

use super::position::{COVERS, PLACES, Position};
use crate::games::tripeaks::{POSITIONS, ranks_adjacent};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::solver) enum Step {
    Draw,
    Play(u8),
}

impl Position {
    /// Every legal move: the stock turn, then each playable peak card.
    pub(super) fn direct(&self, out: &mut Vec<Step>) {
        if self.can_draw() {
            out.push(Step::Draw);
        }
        out.extend(
            (0..POSITIONS)
                .filter(|&at| self.can_play(at))
                .map(|at| Step::Play(u8::try_from(at).unwrap_or(0))),
        );
    }

    fn score(&self, at: usize) -> (usize, usize, u8) {
        let played = self.ranks[at];
        let next_mask = self.mask & !(1 << at);
        let uncovers = (0..POSITIONS)
            .filter(|&p| {
                self.mask & (1 << p) != 0
                    && self.mask & COVERS[p] != 0
                    && next_mask & COVERS[p] == 0
            })
            .count();
        let next_plays = (0..POSITIONS)
            .filter(|&other| {
                Position::is_free_under(next_mask, other)
                    && ranks_adjacent(self.ranks[other], played)
            })
            .count();
        let row = PLACES[at].0;
        (next_plays, uncovers, row)
    }

    fn score_step(&self, step: Step) -> Option<(usize, usize, u8)> {
        match step {
            Step::Draw => None,
            Step::Play(at) => Some(self.score(usize::from(at))),
        }
    }

    /// The steps worth searching, best first; empty on a dead position.
    pub(super) fn ordered(&self, out: &mut Vec<Step>) {
        if self.is_dead() {
            return;
        }
        let mut all = Vec::new();
        self.direct(&mut all);
        all.sort_by_key(|&step| Reverse(self.score_step(step)));
        out.extend(all);
    }
}
