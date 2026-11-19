//! Checks of a solver model against the real engine along random playouts.

use crate::game::{Game, Variant, deal};
use crate::moves::Move;
use crate::rng::SplitMix64;
use crate::solver::search::Model;
use std::collections::HashSet;

const SEEDS: u64 = 25;
const STEPS: usize = 140;
const STEPS_CHECKED: usize = 10;

/// Plays random legal moves on `SEEDS` deals and calls `check` with the game, the seed and the
/// moves played so far before each move.
pub(in crate::solver) fn walk(variant: Variant, mut check: impl FnMut(&dyn Game, u64, &[Move])) {
    for seed in 0..SEEDS {
        let mut game = deal(variant, seed);
        let mut history = Vec::new();
        let mut rng = SplitMix64::new(seed);
        for _ in 0..STEPS {
            check(game.as_ref(), seed, &history);
            let moves = game.legal_moves();
            let bound = u64::try_from(moves.len()).unwrap();
            let Some(&mv) = moves.get(usize::try_from(rng.below(bound)).unwrap()) else {
                break;
            };
            game.apply(mv).unwrap();
            history.push(mv);
        }
    }
}

pub(in crate::solver) fn same_moves(model: Vec<Move>, engine: Vec<Move>, context: &str) {
    let model: HashSet<Move> = model.into_iter().collect();
    let engine: HashSet<Move> = engine.into_iter().collect();
    assert_eq!(model, engine, "{context}");
}

pub(in crate::solver) fn no_finish(_: &[Move]) -> Option<Vec<Move>> {
    None
}

/// Along random playouts of `variant`: every step of the model, applied to the model, gives the
/// moves that are legal on the engine and lead to the position the model computed.
pub(in crate::solver) fn check_steps<M: Model>(
    variant: Variant,
    make: impl Fn(&dyn Game) -> Option<M>,
) {
    walk(variant, |game, seed, history| {
        let position = make(game).unwrap();
        let mut steps = Vec::new();
        position.steps(&mut steps);
        for &step in steps.iter().take(STEPS_CHECKED) {
            let mut child = position.clone();
            let mut moves = Vec::new();
            child.apply(step, &mut moves);
            let mut fresh = deal(variant, seed);
            for &mv in history.iter().chain(&moves) {
                fresh.apply(mv).unwrap();
            }
            assert_eq!(child.is_won(), fresh.is_won(), "{variant:?} seed {seed}");
            assert_eq!(
                child.key(),
                make(fresh.as_ref()).unwrap().key(),
                "{variant:?} seed {seed} after {} moves",
                history.len()
            );
        }
    });
}
