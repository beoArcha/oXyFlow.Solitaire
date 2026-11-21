//! Golf for the solver (T-45). A position is just the column heights, the stock size and the rank
//! of the waste top, so the state space is small and the depth-first search often exhausts it:
//! a deal is then proven unwinnable. Every play and the stock turn are steps, plays first; no
//! move is safe to make at once, because each play changes the waste top.

mod position;
mod steps;
#[cfg(test)]
mod tests;

pub(in crate::solver) use position::Position;

use crate::moves::Move;
use crate::pile::PileRef;
use crate::solver::search::Model;
use steps::Step;

fn tableau(column: usize) -> PileRef {
    PileRef::Tableau(u8::try_from(column).unwrap_or(u8::MAX))
}

impl Position {
    fn play_step(&mut self, step: Step, moves: &mut Vec<Move>) {
        match step {
            Step::Draw => {
                moves.push(Move::Stock);
                self.draw();
            }
            Step::Play(column) => {
                moves.push(Move::Transfer {
                    from: tableau(column),
                    to: PileRef::Waste,
                    count: 1,
                });
                self.play(column);
            }
        }
    }

    /// The engine move of a step, for the tests.
    #[cfg(test)]
    fn engine_move(&self, step: Step) -> Option<Move> {
        let mut moves = Vec::new();
        self.clone().play_step(step, &mut moves);
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
        self.play_step(step, moves);
    }

    fn settle(&mut self, _moves: &mut Vec<Move>) {}
}
