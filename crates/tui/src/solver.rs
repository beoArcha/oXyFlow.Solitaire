//! The winnability service the application asks for deals proven winnable. The TUI never solves
//! anything itself: a [`Solver`] runs queries off the UI thread (the binary wires a worker thread,
//! tests a stub) and the application collects the [`Answer`]s on every input and tick.

use oxysol_core::{PyramidRules, SuitCount, Variant};
use serde::{Deserialize, Serialize};

/// What is known about one deal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Proof {
    Winnable,
    Unwinnable,
    Unknown,
}

/// What the board knows about the running deal, for the header.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DealCheck {
    /// A random deal nobody asked about: the header says nothing.
    #[default]
    Unchecked,
    /// The solver is working on it.
    Checking,
    Known(Proof),
}

impl DealCheck {
    /// Localization key of the header text, `None` when there is nothing to say.
    pub const fn label_key(self) -> Option<&'static str> {
        match self {
            DealCheck::Unchecked => None,
            DealCheck::Checking => Some("board.header.winnable_checking"),
            DealCheck::Known(Proof::Winnable) => Some("board.header.winnable_yes"),
            DealCheck::Known(Proof::Unknown) => Some("board.header.winnable_unknown"),
            DealCheck::Known(Proof::Unwinnable) => Some("board.header.winnable_no"),
        }
    }
}

/// One deal to check. `id` is chosen by the application and comes back in the [`Answer`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Query {
    pub id: u64,
    pub variant: Variant,
    pub seed: u64,
}

/// The verdict for the query with the same `id`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Answer {
    pub id: u64,
    pub proof: Proof,
}

/// Checks deals in the background. Implementations must never block the caller.
pub trait Solver {
    /// Queues a query; its answer arrives through [`Solver::poll`] in submission order.
    fn submit(&mut self, query: Query);
    /// Drops every query that has not started. An answer already in flight may still arrive and
    /// is ignored by the application.
    fn cancel(&mut self);
    /// The answers that arrived since the last call, oldest first.
    fn poll(&mut self) -> Vec<Answer>;
}

/// A solver that answers every query with [`Proof::Unknown`] at once: the fallback when no
/// worker is available.
#[derive(Debug, Default)]
pub struct NullSolver {
    answers: Vec<Answer>,
}

impl Solver for NullSolver {
    fn submit(&mut self, query: Query) {
        self.answers.push(Answer {
            id: query.id,
            proof: Proof::Unknown,
        });
    }

    fn cancel(&mut self) {
        self.answers.clear();
    }

    fn poll(&mut self) -> Vec<Answer> {
        std::mem::take(&mut self.answers)
    }
}

/// Variants of which the solver proves only a small share of the deals (decision T-29: Spider with
/// 2 and 4 suits, Pyramid Classic), so a search for a winnable deal can take long.
pub const fn slow_to_prove(variant: Variant) -> bool {
    matches!(
        variant,
        Variant::Spider(SuitCount::Two | SuitCount::Four) | Variant::Pyramid(PyramidRules::Classic)
    )
}

#[cfg(test)]
mod tests {
    use oxysol_core::DrawCount;

    use super::*;

    #[test]
    fn only_the_rarely_proven_variants_are_slow() {
        let slow: Vec<Variant> = Variant::ALL
            .into_iter()
            .filter(|variant| slow_to_prove(*variant))
            .collect();
        assert_eq!(
            slow,
            vec![
                Variant::Spider(SuitCount::Two),
                Variant::Spider(SuitCount::Four),
                Variant::Pyramid(PyramidRules::Classic),
            ]
        );
        assert!(!slow_to_prove(Variant::Klondike(DrawCount::Three)));
    }

    #[test]
    fn the_null_solver_answers_unknown_and_forgets_on_cancel() {
        let mut solver = NullSolver::default();
        let query = Query {
            id: 5,
            variant: Variant::FreeCell,
            seed: 1,
        };
        solver.submit(query);
        assert_eq!(
            solver.poll(),
            vec![Answer {
                id: 5,
                proof: Proof::Unknown
            }]
        );
        solver.submit(query);
        solver.cancel();
        assert!(solver.poll().is_empty());
    }
}
