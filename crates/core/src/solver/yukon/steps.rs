//! The steps of a Yukon position: every group move, the plays of the talon after the stock draws
//! they need (Australian Patience), the moves back from a foundation, and the ordered list the
//! search walks. Nothing that can matter is left out, so an exhausted search is a proof.

use super::position::{COLUMNS, Position, SUITS};
use crate::solver::face::Face;
use std::cmp::Reverse;

/// Where a card from the waste goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::solver) enum Dest {
    Foundation,
    Column(u8),
}

/// One edge of the search.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::solver) enum Step {
    /// The top `count` cards of a column, in any order, onto another column.
    Group { from: u8, to: u8, count: u8 },
    /// The top card of a column onto its foundation.
    Up { from: u8 },
    /// `draws` stock moves, then the waste top to `to`. `draws` is 0 for the current waste top.
    Talon { draws: u8, to: Dest },
    /// The top card of the foundation of `suit` back onto a column.
    Down { suit: u8, to: u8 },
}

fn index(value: usize) -> u8 {
    u8::try_from(value).unwrap_or(u8::MAX)
}

impl Position {
    /// Every legal move except the stock draws, each once (onto an empty column, only the first).
    pub(super) fn direct(&self, out: &mut Vec<Step>) {
        for from in 0..COLUMNS {
            self.column_moves(from, out);
        }
        if let Some(card) = self.talon.waste_top() {
            self.placements(card, 0, out);
        }
        for suit in 0..SUITS {
            self.retreats(suit, out);
        }
    }

    fn first_empty(&self) -> Option<usize> {
        (0..COLUMNS).find(|&to| self.len[to] == 0)
    }

    fn targets(&self, card: Face, from: Option<usize>) -> impl Iterator<Item = usize> + '_ {
        let first_empty = self.first_empty();
        (0..COLUMNS).filter(move |&to| {
            Some(to) != from
                && self.accepts(to, card)
                && (self.len[to] > 0 || first_empty == Some(to))
        })
    }

    fn column_moves(&self, from: usize, out: &mut Vec<Step>) {
        if self
            .top(from)
            .is_some_and(|top| self.foundation_accepts(top))
        {
            out.push(Step::Up { from: index(from) });
        }
        for count in 1..=self.face_up(from) {
            let card = self.cols[from][self.len[from] - count];
            for to in self.targets(card, Some(from)) {
                out.push(Step::Group {
                    from: index(from),
                    to: index(to),
                    count: index(count),
                });
            }
        }
    }

    /// The plays of `card`, the waste top after `draws` stock moves.
    fn placements(&self, card: Face, draws: u8, out: &mut Vec<Step>) {
        if self.foundation_accepts(card) {
            out.push(Step::Talon {
                draws,
                to: Dest::Foundation,
            });
        }
        for to in self.targets(card, None) {
            out.push(Step::Talon {
                draws,
                to: Dest::Column(index(to)),
            });
        }
    }

    fn retreats(&self, suit: usize, out: &mut Vec<Step>) {
        let Some(card) = self.foundation_top(suit) else {
            return;
        };
        for to in self.targets(card, None) {
            out.push(Step::Down {
                suit: index(suit),
                to: index(to),
            });
        }
    }

    /// The plays of every card the stock can still bring to the top of the waste. Drawing without
    /// playing is never a step of its own: it can always be postponed until the play it prepares.
    fn talon_plays(&self, out: &mut Vec<Step>) {
        for draws in 1..=self.talon.stock_len {
            if let Some(card) = self.talon.top_after(draws) {
                self.placements(card, index(draws), out);
            }
        }
    }

    /// The steps worth searching, best first. Only a move that cannot matter is left out: a whole
    /// column without face-down cards to an empty column only swaps two columns.
    pub(super) fn ordered(&self, out: &mut Vec<Step>) {
        let mut all = Vec::new();
        self.direct(&mut all);
        self.talon_plays(&mut all);
        let mut scored: Vec<(u32, Step)> = all
            .into_iter()
            .filter_map(|step| self.score_step(step).map(|score| (score, step)))
            .collect();
        scored.sort_by_key(|&(score, _)| Reverse(score));
        out.extend(scored.into_iter().map(|(_, step)| step));
    }

    fn score_step(&self, step: Step) -> Option<u32> {
        match step {
            Step::Group { from, to, count } => {
                self.score_group(usize::from(from), usize::from(to), usize::from(count))
            }
            Step::Up { from } => {
                let from = usize::from(from);
                let reveals = self.face_up(from) == 1 && self.hidden[from] > 0;
                Some(if reveals { 900 } else { 800 })
            }
            Step::Talon { draws, to } => {
                let base: u32 = match to {
                    Dest::Foundation => 750,
                    Dest::Column(column) if self.len[usize::from(column)] == 0 => 650,
                    Dest::Column(_) => 600,
                };
                Some(base.saturating_sub(20 * u32::from(draws)))
            }
            Step::Down { .. } => Some(50),
        }
    }

    /// Whether the group of the top `count` cards of `from` lies on a face-up card it fits.
    fn rests_on_fit(&self, from: usize, count: usize) -> bool {
        let at = self.len[from] - count;
        count < self.face_up(from) && self.fits(self.cols[from][at - 1], self.cols[from][at])
    }

    fn score_group(&self, from: usize, to: usize, count: usize) -> Option<u32> {
        let whole = count == self.face_up(from);
        let to_empty = self.len[to] == 0;
        if whole && to_empty && self.hidden[from] == 0 {
            return None;
        }
        if whole && self.hidden[from] > 0 {
            return Some(1000 + 30 * u32::try_from(self.hidden[from]).unwrap_or(0));
        }
        if whole {
            return Some(450);
        }
        let exposed = self.cols[from][self.len[from] - count - 1];
        if self.foundation_accepts(exposed) {
            return Some(700);
        }
        if self.rests_on_fit(from, count) {
            return Some(100);
        }
        Some(if to_empty { 400 } else { 300 })
    }
}
