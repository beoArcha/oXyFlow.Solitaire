//! The generic searches over a game model. A model is a compact copy of one game's rules; a
//! search walks positions, never touches the engine, and hands back the engine moves of the path.

mod best_first;
mod depth_first;

use crate::moves::Move;

/// One game as the solver sees it. Positions are small, `Clone` and hashed canonically.
pub(super) trait Model: Clone {
    /// One edge of the search; may stand for several engine moves (a stock draw run and a play).
    type Step: Copy;

    fn is_won(&self) -> bool;

    /// The canonical key: equal for positions that differ only by equivalent permutations.
    fn key(&self) -> u128;

    /// The steps worth trying, best first. Complete: no winning line is ever left out, only
    /// reordered, so an exhausted search proves the position unwinnable.
    fn steps(&self, out: &mut Vec<Self::Step>);

    /// Performs `step` and every safe automatic move after it; appends the engine moves.
    fn apply(&mut self, step: Self::Step, moves: &mut Vec<Move>);

    /// The safe automatic moves of a position that did not come out of [`Model::apply`].
    fn settle(&mut self, moves: &mut Vec<Move>);

    /// A cheap test that the engine's `forced_finish` will certainly finish this position.
    fn may_finish(&self) -> bool {
        false
    }

    /// An estimate of the work left, lower is closer to a win. Used by the best-first search.
    fn score(&self) -> u32 {
        0
    }
}

/// How a search ended.
pub(super) enum Outcome {
    Won(Vec<Move>),
    Exhausted,
    OutOfBudget,
}

/// The outcome and the number of positions generated.
pub(super) struct Searched {
    pub(super) outcome: Outcome,
    pub(super) nodes: u64,
}

/// Which order the search expands positions in.
#[derive(Clone, Copy)]
pub(super) enum Strategy {
    DepthFirst,
    BestFirst,
}

/// Moves that finish the game after `moves`, when the engine says the rest is a certain win.
pub(super) type Finisher<'a> = &'a dyn Fn(&[Move]) -> Option<Vec<Move>>;

/// Searches from `start` with at most `budget` positions generated (the first one included).
pub(super) fn run<M: Model>(
    strategy: Strategy,
    start: &M,
    budget: u64,
    finish: Finisher,
) -> Searched {
    let mut root = start.clone();
    let mut prefix = Vec::new();
    root.settle(&mut prefix);
    if let Some(moves) = solution(&root, &prefix, &[], &root, finish) {
        return Searched {
            outcome: Outcome::Won(moves),
            nodes: 1,
        };
    }
    match strategy {
        Strategy::DepthFirst => depth_first::search(&root, &prefix, budget, finish),
        Strategy::BestFirst => best_first::search(&root, &prefix, budget, finish),
    }
}

/// The whole winning move list when `at`, reached from `root` by `steps`, is won or a forced win.
fn solution<M: Model>(
    root: &M,
    prefix: &[Move],
    steps: &[M::Step],
    at: &M,
    finish: Finisher,
) -> Option<Vec<Move>> {
    let won = at.is_won();
    if !won && !at.may_finish() {
        return None;
    }
    let mut moves = prefix.to_vec();
    let mut state = root.clone();
    for &step in steps {
        state.apply(step, &mut moves);
    }
    if won {
        return Some(moves);
    }
    moves.extend(finish(&moves)?);
    Some(moves)
}
