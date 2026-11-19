//! FreeCell for the solver. Every deal is fully known, so the search is a plain best-first walk
//! over positions with a "work left" estimate (cards still to place, cards that bury the next
//! card of each foundation, cards that sit above a lower card). Cell order and column order do not
//! matter for the visited set. Aces and every card that nothing can still be built on go to the
//! foundations at once.

mod heuristic;
mod position;
mod steps;
#[cfg(test)]
mod tests;

pub(in crate::solver) use position::Position;

use crate::moves::Move;
use crate::pile::PileRef;
use crate::solver::face::{Face, NONE, is_red, rank};
use crate::solver::search::Model;
use position::{CELLS, COLUMNS, SUITS};
use steps::Step;

fn tableau(column: usize) -> PileRef {
    PileRef::Tableau(u8::try_from(column).unwrap_or(u8::MAX))
}

fn cell(index: usize) -> PileRef {
    PileRef::Cell(u8::try_from(index).unwrap_or(u8::MAX))
}

fn transfer(from: PileRef, to: PileRef, count: usize) -> Move {
    Move::Transfer {
        from,
        to,
        count: u8::try_from(count).unwrap_or(u8::MAX),
    }
}

impl Position {
    /// A card may go to its foundation for good when nothing of the other color can still need it
    /// as a base.
    fn is_safe(&self, card: Face) -> bool {
        let needed = rank(card);
        let red = is_red(card);
        needed <= 1
            || (0..SUITS)
                .filter(|&s| (s == 1 || s == 2) != red)
                .all(|s| self.found[s] >= needed)
    }

    fn foundation_move(&self, from: PileRef, card: Face) -> Move {
        transfer(from, PileRef::Foundation(self.foundation_index(card)), 1)
    }

    fn settle_once(&mut self, moves: &mut Vec<Move>) -> bool {
        for column in 0..COLUMNS {
            let Some(card) = self.top(column) else {
                continue;
            };
            if self.foundation_accepts(card) && self.is_safe(card) {
                moves.push(self.foundation_move(tableau(column), card));
                self.pop(column);
                self.place_home(card);
                return true;
            }
        }
        for index in 0..CELLS {
            let card = self.cells[index];
            if card != NONE && self.foundation_accepts(card) && self.is_safe(card) {
                moves.push(self.foundation_move(cell(index), card));
                self.cells[index] = NONE;
                self.place_home(card);
                return true;
            }
        }
        false
    }

    fn play(&mut self, step: Step, moves: &mut Vec<Move>) {
        match step {
            Step::Run { from, to, count } => {
                let (from, to, count) = (usize::from(from), usize::from(to), usize::from(count));
                moves.push(transfer(tableau(from), tableau(to), count));
                let start = self.len[from] - count;
                for offset in 0..count {
                    self.push(to, self.cols[from][start + offset]);
                }
                self.len[from] = start;
            }
            Step::ToCell { from } => {
                let from = usize::from(from);
                if let Some(free) = self.first_free_cell() {
                    moves.push(transfer(tableau(from), cell(free), 1));
                    self.cells[free] = self.pop(from);
                }
            }
            Step::FromCell { cell: index, to } => {
                let (index, to) = (usize::from(index), usize::from(to));
                moves.push(transfer(cell(index), tableau(to), 1));
                self.push(to, self.cells[index]);
                self.cells[index] = NONE;
            }
            Step::Up { from } => {
                let from = usize::from(from);
                if let Some(card) = self.top(from) {
                    moves.push(self.foundation_move(tableau(from), card));
                    self.pop(from);
                    self.place_home(card);
                }
            }
            Step::UpCell { cell: index } => {
                let index = usize::from(index);
                let card = self.cells[index];
                moves.push(self.foundation_move(cell(index), card));
                self.cells[index] = NONE;
                self.place_home(card);
            }
        }
    }

    /// The engine move of a step, for the tests.
    #[cfg(test)]
    fn engine_move(&self, step: Step) -> Option<Move> {
        let mut moves = Vec::new();
        self.clone().play(step, &mut moves);
        moves.pop()
    }
}

impl Model for Position {
    type Step = Step;

    fn is_won(&self) -> bool {
        Position::is_won(self)
    }

    fn key(&self) -> u128 {
        Position::key(self)
    }

    fn steps(&self, out: &mut Vec<Step>) {
        self.useful(out);
    }

    fn apply(&mut self, step: Step, moves: &mut Vec<Move>) {
        self.play(step, moves);
        self.settle(moves);
    }

    fn settle(&mut self, moves: &mut Vec<Move>) {
        while self.settle_once(moves) {}
    }

    fn may_finish(&self) -> bool {
        self.columns_descend()
    }

    fn score(&self) -> u32 {
        self.work_left()
    }
}
