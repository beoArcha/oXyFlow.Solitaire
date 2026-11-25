//! Spider for the solver (1, 2 and 4 suits). A hard game for a bounded search: the stock deals ten
//! cards at a time and runs are only safe in one suit, so with two and four suits most deals end as
//! `Unknown` within the budget. The search is depth-first with moves ordered by what they achieve
//! (complete a run, join a suit, turn a card over); positions that differ only by column order
//! share one key. Nothing is pruned that a win could need, so an exhausted search is a proof.

mod heuristic;
mod multi;
mod multi_moves;
mod position;
mod steps;
#[cfg(test)]
mod tests;

pub(in crate::solver) use multi::search as search_multi;
pub(in crate::solver) use position::Position;

use crate::moves::Move;
use crate::pile::PileRef;
use crate::solver::search::Model;
use position::COLUMNS;
use steps::Step;

fn tableau(column: usize) -> PileRef {
    PileRef::Tableau(u8::try_from(column).unwrap_or(u8::MAX))
}

impl Position {
    fn play(&mut self, step: Step, moves: &mut Vec<Move>) {
        match step {
            Step::Deal => {
                moves.push(Move::Stock);
                self.deal_row();
            }
            Step::Run { from, to, count } => {
                let (from, to, count) = (usize::from(from), usize::from(to), usize::from(count));
                moves.push(Move::Transfer {
                    from: tableau(from),
                    to: tableau(to),
                    count: u8::try_from(count).unwrap_or(u8::MAX),
                });
                self.move_run(from, to, count);
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
        self.ordered(out);
    }

    fn apply(&mut self, step: Step, moves: &mut Vec<Move>) {
        self.play(step, moves);
    }

    fn settle(&mut self, _moves: &mut Vec<Move>) {}

    fn may_finish(&self) -> bool {
        self.stock_len == 0
            && (0..COLUMNS)
                .all(|column| self.hidden[column] == 0 && self.run_len(column) == self.len[column])
    }
}
