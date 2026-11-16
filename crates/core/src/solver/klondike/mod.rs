//! Klondike for the solver (draw 1 and draw 3). The solver knows the stock order, so it solves the
//! deal as dealt: "thoroughly solvable" in the literature sense. Stock draws are not steps of their
//! own; a card of the talon is played together with the draws that bring it to the waste top.
//! Aces and twos, and every card whose opposite-color lower cards are already home, go to the
//! foundations at once (the talon only in draw 1: in draw 3 a play shifts the later triples).

mod position;
mod steps;
#[cfg(test)]
mod tests;

pub(in crate::solver) use position::Position;

use crate::moves::Move;
use crate::pile::PileRef;
use crate::solver::face::{Face, rank};
use crate::solver::search::Model;
use position::{COLUMNS, SUITS};
use steps::{Dest, Step};

fn tableau(column: usize) -> PileRef {
    PileRef::Tableau(u8::try_from(column).unwrap_or(u8::MAX))
}

fn transfer(from: PileRef, to: PileRef, count: usize) -> Move {
    Move::Transfer {
        from,
        to,
        count: u8::try_from(count).unwrap_or(u8::MAX),
    }
}

impl Position {
    /// A card may go to its foundation for good when no lower card of the other color can still
    /// need it as a base.
    fn is_safe(&self, card: Face) -> bool {
        let needed = rank(card);
        let red = crate::solver::face::is_red(card);
        needed <= 1
            || (0..SUITS)
                .filter(|&s| (s == 1 || s == 2) != red)
                .all(|s| self.found[s] >= needed)
    }

    fn foundation_move(&self, from: PileRef, card: Face) -> Move {
        let to = PileRef::Foundation(self.foundation_index(card));
        transfer(from, to, 1)
    }

    fn settle_once(&mut self, moves: &mut Vec<Move>) -> bool {
        if self.draw == 1
            && let Some(card) = self.talon.waste_top()
            && self.foundation_accepts(card)
            && self.is_safe(card)
        {
            moves.push(self.foundation_move(PileRef::Waste, card));
            self.talon.pop_waste();
            self.to_foundation(card);
            return true;
        }
        for column in 0..COLUMNS {
            let Some(card) = self.top(column) else {
                continue;
            };
            if self.foundation_accepts(card) && self.is_safe(card) {
                moves.push(self.foundation_move(tableau(column), card));
                self.pop(column);
                self.to_foundation(card);
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
                self.move_run(from, to, count);
            }
            Step::Up { from } => {
                let from = usize::from(from);
                if let Some(card) = self.top(from) {
                    moves.push(self.foundation_move(tableau(from), card));
                    self.pop(from);
                    self.to_foundation(card);
                }
            }
            Step::Talon { draws, to } => self.play_talon(draws, to, moves),
            Step::Down { suit, to } => {
                let suit = usize::from(suit);
                if let Some(card) = self.foundation_top(suit) {
                    let from = PileRef::Foundation(self.foundation_slot(suit));
                    moves.push(transfer(from, tableau(usize::from(to)), 1));
                    self.from_foundation(suit);
                    self.push(usize::from(to), card);
                }
            }
        }
    }

    fn play_talon(&mut self, draws: u8, to: Dest, moves: &mut Vec<Move>) {
        for _ in 0..draws {
            moves.push(Move::Stock);
            self.talon.draw(self.draw);
        }
        let Some(card) = self.talon.waste_top() else {
            return;
        };
        match to {
            Dest::Foundation => {
                moves.push(self.foundation_move(PileRef::Waste, card));
                self.to_foundation(card);
            }
            Dest::Column(column) => {
                moves.push(transfer(PileRef::Waste, tableau(usize::from(column)), 1));
                self.push(usize::from(column), card);
            }
        }
        self.talon.pop_waste();
    }

    /// The engine move of a direct step; talon plays carry their draws separately.
    #[cfg(test)]
    fn engine_move(&self, step: Step) -> Option<Move> {
        let mut moves = Vec::new();
        let mut copy = self.clone();
        copy.play(step, &mut moves);
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
        self.settle(moves);
    }

    fn settle(&mut self, moves: &mut Vec<Move>) {
        while self.settle_once(moves) {}
    }

    fn may_finish(&self) -> bool {
        self.talon.is_empty() && self.hidden.iter().all(|&hidden| hidden == 0)
    }
}
