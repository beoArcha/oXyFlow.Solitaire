//! The steps of a FreeCell position: every legal move, each once (onto an empty column or into a
//! free cell only the first one, the others are interchangeable).

use super::position::{CELLS, COLUMNS, Position};
use crate::solver::face::{self, NONE};

/// One edge of the search.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::solver) enum Step {
    /// The top `count` cards of a column onto another column.
    Run { from: u8, to: u8, count: u8 },
    /// The top card of a column into the first free cell.
    ToCell { from: u8 },
    /// A cell card onto a column.
    FromCell { cell: u8, to: u8 },
    /// The top card of a column onto its foundation.
    Up { from: u8 },
    /// A cell card onto its foundation.
    UpCell { cell: u8 },
}

fn index(value: usize) -> u8 {
    u8::try_from(value).unwrap_or(u8::MAX)
}

impl Position {
    pub(super) fn direct(&self, out: &mut Vec<Step>) {
        for from in (0..COLUMNS).filter(|&from| self.len[from] > 0) {
            self.column_moves(from, out);
        }
        for cell in (0..CELLS).filter(|&cell| self.cells[cell] != NONE) {
            self.cell_moves(cell, out);
        }
    }

    fn column_moves(&self, from: usize, out: &mut Vec<Step>) {
        let Some(top) = self.top(from) else {
            return;
        };
        if self.foundation_accepts(top) {
            out.push(Step::Up { from: index(from) });
        }
        if self.free_cells() > 0 {
            out.push(Step::ToCell { from: index(from) });
        }
        let run = self.run_len(from);
        let onto_card = self.capacity(false);
        for to in (0..COLUMNS).filter(|&to| to != from) {
            let Some(target) = self.top(to) else {
                continue;
            };
            let count = (1..=run.min(onto_card)).find(|&count| {
                face::fits_alternating(target, self.cols[from][self.len[from] - count])
            });
            if let Some(count) = count {
                out.push(Step::Run {
                    from: index(from),
                    to: index(to),
                    count: index(count),
                });
            }
        }
        if let Some(empty) = self.first_empty_column() {
            for count in 1..=run.min(self.capacity(true)) {
                out.push(Step::Run {
                    from: index(from),
                    to: index(empty),
                    count: index(count),
                });
            }
        }
    }

    fn cell_moves(&self, cell: usize, out: &mut Vec<Step>) {
        let card = self.cells[cell];
        if self.foundation_accepts(card) {
            out.push(Step::UpCell { cell: index(cell) });
        }
        let first_empty = self.first_empty_column();
        for to in 0..COLUMNS {
            let empty = self.len[to] == 0;
            if self.accepts(to, card) && (!empty || first_empty == Some(to)) {
                out.push(Step::FromCell {
                    cell: index(cell),
                    to: index(to),
                });
            }
        }
    }

    /// The steps worth searching. Only moves that cannot matter are left out: a whole column to an
    /// empty column only swaps two columns.
    pub(super) fn useful(&self, out: &mut Vec<Step>) {
        self.direct(out);
        out.retain(|step| match *step {
            Step::Run { from, to, count } => {
                !(self.len[usize::from(to)] == 0
                    && usize::from(count) == self.len[usize::from(from)])
            }
            _ => true,
        });
    }
}
