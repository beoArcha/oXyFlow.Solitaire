//! Pyramid for the solver (Relaxed and Classic). The pyramid is a bit mask and the talon a short
//! known sequence, so the state space is small and the depth-first search often exhausts it: most
//! Classic deals are provably unwinnable. Lone kings are removed at once (nothing can need a
//! king); every other removal and the stock move are steps, removals first.

mod position;
mod steps;
#[cfg(test)]
mod tests;

pub(in crate::solver) use position::{Position, Slot};

use crate::moves::Move;
use crate::pile::PileRef;
use crate::solver::search::Model;
use position::{PLACES, value};
use steps::Step;

fn pile(slot: Slot) -> PileRef {
    match slot {
        Slot::Stock => PileRef::Stock,
        Slot::Waste => PileRef::Waste,
        Slot::Pyramid(at) => {
            let (row, index) = PLACES.get(usize::from(at)).copied().unwrap_or((0, 0));
            PileRef::Pyramid { row, index }
        }
    }
}

impl Position {
    fn remove(&mut self, first: Slot, second: Option<Slot>, moves: &mut Vec<Move>) {
        moves.push(Move::Remove {
            first: pile(first),
            second: second.map(pile),
        });
        self.take(first);
        if let Some(second) = second {
            self.take(second);
        }
    }

    fn play(&mut self, step: Step, moves: &mut Vec<Move>) {
        match step {
            Step::Draw => {
                moves.push(Move::Stock);
                self.draw();
            }
            Step::Remove { first, second } => self.remove(first, second, moves),
        }
    }

    /// The engine move of a step, for the tests.
    #[cfg(test)]
    fn engine_move(&self, step: Step) -> Option<Move> {
        let mut moves = Vec::new();
        self.clone().play(step, &mut moves);
        moves.pop()
    }

    fn talon_values(&self) -> impl Iterator<Item = u8> + '_ {
        self.talon_faces().map(value)
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
        self.settle(moves);
    }

    fn settle(&mut self, moves: &mut Vec<Move>) {
        loop {
            let king = self
                .available()
                .items()
                .iter()
                .find(|&&(_, value)| value == position::KING_VALUE)
                .map(|&(slot, _)| slot);
            let Some(slot) = king else {
                return;
            };
            self.remove(slot, None, moves);
        }
    }
}
