//! The steps of a Spider position: the row deal and every movable run onto every column that takes
//! it (onto an empty column only the first one), ordered by what the move achieves.

use super::position::{Position, RUN};
use crate::solver::face::{Face, suit};
use std::cmp::Reverse;

/// One edge of the search.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::solver) enum Step {
    /// The top `count` cards of a column onto another column.
    Run { from: u8, to: u8, count: u8 },
    /// One card from the stock onto every column.
    Deal,
}

fn index(value: usize) -> u8 {
    u8::try_from(value).unwrap_or(u8::MAX)
}

impl Position {
    pub(super) fn direct(&self, out: &mut Vec<Step>) {
        if self.can_deal() {
            out.push(Step::Deal);
        }
        for from in (0..super::position::COLUMNS).filter(|&from| self.len[from] > 0) {
            self.run_moves(from, out);
        }
    }

    fn run_moves(&self, from: usize, out: &mut Vec<Step>) {
        let first_empty = self.first_empty_column();
        for count in 1..=self.run_len(from) {
            let card = self.cols[from][self.len[from] - count];
            for to in (0..super::position::COLUMNS).filter(|&to| to != from) {
                let empty = self.len[to] == 0;
                if self.accepts(to, card) && (!empty || first_empty == Some(to)) {
                    out.push(Step::Run {
                        from: index(from),
                        to: index(to),
                        count: index(count),
                    });
                }
            }
        }
    }

    /// The steps worth searching, best first. Only moves that cannot matter are left out: a whole
    /// column with nothing face down to an empty column only swaps two columns.
    pub(super) fn ordered(&self, out: &mut Vec<Step>) {
        let mut all = Vec::new();
        self.direct(&mut all);
        let mut scored: Vec<(u32, Step)> = all
            .into_iter()
            .filter_map(|step| self.score_step(step).map(|score| (score, step)))
            .collect();
        scored.sort_by_key(|&(score, _)| Reverse(score));
        out.extend(scored.into_iter().map(|(_, step)| step));
    }

    fn score_step(&self, step: Step) -> Option<u32> {
        match step {
            Step::Deal => Some(10),
            Step::Run { from, to, count } => {
                self.score_run(usize::from(from), usize::from(to), usize::from(count))
            }
        }
    }

    fn score_run(&self, from: usize, to: usize, count: usize) -> Option<u32> {
        let start = self.len[from] - count;
        let card: Face = self.cols[from][start];
        let to_empty = self.len[to] == 0;
        if to_empty && count == self.len[from] && self.hidden[from] == 0 {
            return None;
        }
        let joins = self.top(to).is_some_and(|top| suit(top) == suit(card));
        let chain = if joins { self.run_len(to) + count } else { count };
        let reveals = start == self.hidden[from] && self.hidden[from] > 0;
        let breaks = start > self.hidden[from]
            && self.cols[from][start - 1] / 13 == card / 13
            && self.cols[from][start - 1] % 13 == card % 13 + 1;
        let base: u32 = if joins && chain >= RUN {
            3000
        } else if joins {
            1000 + 20 * u32::try_from(chain).unwrap_or(0)
        } else if reveals {
            900 + u32::try_from(self.hidden[from]).unwrap_or(0)
        } else if to_empty {
            80
        } else {
            100
        };
        Some(base.saturating_sub(if breaks { 400 } else { 0 }))
    }
}
