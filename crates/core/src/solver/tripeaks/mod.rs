//! TriPeaks for the solver. A position is the 28-bit mask of remaining peak cards, the stock
//! size, and the rank of the waste top. Every play and the stock turn are steps, plays first;
//! a rank-group count proves dead positions without losing completeness.

mod position;
mod steps;
#[cfg(test)]
mod tests;

pub(in crate::solver) use position::Position;

use crate::moves::Move;
use crate::pile::PileRef;
use crate::solver::search::Model;
use position::PLACES;
use steps::Step;

fn peak(at: usize) -> PileRef {
    let (row, index) = PLACES[at];
    PileRef::Pyramid { row, index }
}

impl Position {
    fn play_step(&mut self, step: Step, moves: &mut Vec<Move>) {
        match step {
            Step::Draw => {
                moves.push(Move::Stock);
                self.draw();
            }
            Step::Play(at) => {
                moves.push(Move::Transfer {
                    from: peak(usize::from(at)),
                    to: PileRef::Waste,
                    count: 1,
                });
                self.play(usize::from(at));
            }
        }
    }

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
