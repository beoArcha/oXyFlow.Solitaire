//! A bounded solver: for a deal it answers "winnable" with the winning moves, "unwinnable", or
//! "unknown within the budget". The budget is a count of generated positions, never time, so the
//! core stays clock-free and the answer for one input never changes.
//!
//! Every game has its own compact model of the rules ([`search::Model`]) built from the dealt
//! game's piles. The search walks the models, so it is fast; the engine is used only to check the
//! result: a winning move list is replayed on a fresh deal and must end in `is_won()`, otherwise
//! the answer is `Unknown`. The solver sees the whole deal, face-down cards and stock order
//! included, so it decides whether the deal as dealt can be won (the "thoroughly solvable"
//! notion of the Klondike literature), not whether a player without that knowledge could win it.

mod face;
mod freecell;
mod hash;
mod klondike;
mod pyramid;
mod search;
mod spider;
#[cfg(test)]
mod tests;

use crate::game::{Variant, deal};
use crate::moves::Move;
use search::{Outcome, Searched, Strategy};

/// What the solver found out about a deal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// The deal can be won; `moves` win it when applied in order to a fresh deal.
    Winnable { moves: Vec<Move> },
    /// The search space was exhausted within the budget and holds no win.
    Unwinnable,
    /// The budget ran out first, or the answer could not be verified.
    Unknown,
}

/// How much work the solver may do: the number of positions it may generate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SolveBudget {
    pub nodes: u64,
}

impl SolveBudget {
    pub const fn nodes(nodes: u64) -> Self {
        Self { nodes }
    }

    /// The budget that answers most deals of `variant` in about half a second in a release build.
    pub const fn default_for(variant: Variant) -> Self {
        Self::nodes(match variant {
            Variant::Klondike(_) => DEFAULT_KLONDIKE_NODES,
            Variant::FreeCell => DEFAULT_FREECELL_NODES,
            Variant::Spider(_) => DEFAULT_SPIDER_NODES,
            Variant::Pyramid(_) => DEFAULT_PYRAMID_NODES,
        })
    }
}

pub const DEFAULT_KLONDIKE_NODES: u64 = 300_000;
pub const DEFAULT_FREECELL_NODES: u64 = 100_000;
pub const DEFAULT_SPIDER_NODES: u64 = 100_000;
pub const DEFAULT_PYRAMID_NODES: u64 = 300_000;

/// A verdict together with the positions it cost.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub verdict: Verdict,
    pub nodes: u64,
}

/// Solves the deal of `variant` and `seed` within `budget`.
pub fn solve(variant: Variant, seed: u64, budget: SolveBudget) -> Verdict {
    solve_report(variant, seed, budget).verdict
}

/// Like [`solve`], and also reports how many positions were generated (at most `budget.nodes`
/// unless the deal is won at once).
pub fn solve_report(variant: Variant, seed: u64, budget: SolveBudget) -> Report {
    let searched = search_deal(variant, seed, budget);
    let nodes = searched.nodes;
    let verdict = match searched.outcome {
        Outcome::Won(moves) if replays_to_win(variant, seed, &moves) => {
            Verdict::Winnable { moves }
        }
        Outcome::Exhausted => Verdict::Unwinnable,
        Outcome::Won(_) | Outcome::OutOfBudget => Verdict::Unknown,
    };
    Report { verdict, nodes }
}

fn unknown() -> Searched {
    Searched {
        outcome: Outcome::OutOfBudget,
        nodes: 0,
    }
}

/// The unverified search of a deal; `Unknown` work when the dealt piles are not understood.
fn search_deal(variant: Variant, seed: u64, budget: SolveBudget) -> Searched {
    let game = deal(variant, seed);
    let piles = game.piles();
    let finish = |moves: &[Move]| finishing_tail(variant, seed, moves);
    match variant {
        Variant::Klondike(draw) => match klondike::Position::from_piles(draw, &piles) {
            Some(start) => search::run(Strategy::DepthFirst, &start, budget.nodes, &finish),
            None => unknown(),
        },
        Variant::FreeCell => match freecell::Position::from_piles(&piles) {
            Some(start) => search::run(Strategy::BestFirst, &start, budget.nodes, &finish),
            None => unknown(),
        },
        Variant::Spider(_) => match spider::Position::from_piles(&piles) {
            Some(start) => search::run(Strategy::DepthFirst, &start, budget.nodes, &finish),
            None => unknown(),
        },
        Variant::Pyramid(rules) => {
            let recycles = game.stock_status().and_then(|status| status.recycles_left);
            match pyramid::Position::from_piles(rules, &piles, recycles.unwrap_or(0)) {
                Some(start) => search::run(Strategy::DepthFirst, &start, budget.nodes, &finish),
                None => unknown(),
            }
        }
    }
}

/// The engine's own finishing moves after `moves` on a fresh deal, when it knows a certain win.
fn finishing_tail(variant: Variant, seed: u64, moves: &[Move]) -> Option<Vec<Move>> {
    let mut game = deal(variant, seed);
    for &mv in moves {
        game.apply(mv).ok()?;
    }
    game.forced_finish()
}

/// Whether `moves` are all legal on a fresh deal and the last one wins.
pub(crate) fn replays_to_win(variant: Variant, seed: u64, moves: &[Move]) -> bool {
    let mut game = deal(variant, seed);
    moves.iter().all(|&mv| game.apply(mv).is_ok()) && game.is_won()
}
