//! The steps of a Golf position: the stock turn and the play of every column top that is adjacent
//! to the waste top, plays first.

use super::position::Position;
use crate::games::golf::{COLUMNS, ranks_adjacent};
use std::cmp::Reverse;

/// One edge of the search.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::solver) enum Step {
    /// Turn the top stock card onto the waste.
    Draw,
    /// Play the top card of a column onto the waste.
    Play(usize),
}

impl Position {
    /// Every legal move: the stock turn, then each column play.
    pub(super) fn direct(&self, out: &mut Vec<Step>) {
        if self.can_draw() {
            out.push(Step::Draw);
        }
        out.extend(
            (0..COLUMNS)
                .filter(|&column| self.can_play(column))
                .map(Step::Play),
        );
    }

    /// How promising playing `column` is: the number of column tops, once its top card has left,
    /// that are adjacent to that card (what the new waste top lets us play next), then the cards
    /// still in the column.
    fn score(&self, column: usize) -> (usize, usize) {
        let Some(played) = self.top(column) else {
            return (0, 0);
        };
        let next_plays = (0..COLUMNS)
            .filter_map(|other| {
                let top = if other == column {
                    self.below(other)
                } else {
                    self.top(other)
                };
                top.filter(|&top| ranks_adjacent(top, played))
            })
            .count();
        (next_plays, self.height(column))
    }

    fn score_step(&self, step: Step) -> Option<(usize, usize)> {
        match step {
            Step::Draw => None,
            Step::Play(column) => Some(self.score(column)),
        }
    }

    /// The steps worth searching, best first: plays by [`Position::score`], the stock turn last.
    /// Nothing is dropped, so an exhausted search proves the position unwinnable.
    pub(super) fn ordered(&self, out: &mut Vec<Step>) {
        let mut all = Vec::new();
        self.direct(&mut all);
        all.sort_by_key(|&step| Reverse(self.score_step(step)));
        out.extend(all);
    }
}
