//! Shared machinery of the forced-win detection ([`crate::Game::forced_finish`]). Every sequence a
//! game returns is first replayed here on a clone through the game's own `apply`, so a bug in a
//! finishing rule can only produce `None`, never an illegal or insufficient suggestion.

use crate::game::Game;
use crate::moves::Move;
use crate::pile::PileRef;

/// The longest finishing sequence accepted; far above what any game needs (at most 104 cards).
const MAX_FINISH_MOVES: usize = 256;

/// Builds a finishing sequence step by step: `next` proposes a move for the simulated position
/// until it is won. `None` when `game` is already won, when `next` has no move (the greedy rule
/// is stuck), when a proposed move is rejected, or when the sequence exceeds the bound.
pub(crate) fn greedy_finish<G: Game + Clone>(
    game: &G,
    next: impl Fn(&G) -> Option<Move>,
) -> Option<Vec<Move>> {
    if game.is_won() {
        return None;
    }
    let mut simulated = game.clone();
    let mut moves = Vec::new();
    while !simulated.is_won() {
        if moves.len() >= MAX_FINISH_MOVES {
            return None;
        }
        let mv = next(&simulated)?;
        simulated.apply(mv).ok()?;
        moves.push(mv);
    }
    Some(moves)
}

/// `moves` itself when it applies cleanly to a clone of `game` and the last move wins; otherwise
/// `None`. An already won game and an empty sequence give `None`.
pub(crate) fn verified_finish<G: Game + Clone>(game: &G, moves: Vec<Move>) -> Option<Vec<Move>> {
    if game.is_won() || moves.is_empty() || moves.len() > MAX_FINISH_MOVES {
        return None;
    }
    let mut simulated = game.clone();
    for &mv in &moves {
        simulated.apply(mv).ok()?;
    }
    simulated.is_won().then_some(moves)
}

/// The first move in `moves` that puts a card onto a foundation.
pub(crate) fn first_to_foundation(moves: &[Move]) -> Option<Move> {
    moves.iter().copied().find(|mv| {
        matches!(
            mv,
            Move::Transfer {
                to: PileRef::Foundation(_),
                ..
            }
        )
    })
}
