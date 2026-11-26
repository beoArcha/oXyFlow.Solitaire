//! A game in progress with history. Undo, redo, restart and restore all rebuild the game by
//! replaying the move log from the seed (decision T-03); no move is ever inverted.

use crate::game::{Game, GameOptions, Variant, deal_with};
use crate::moves::{Move, MoveError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

type Dealer = fn(Variant, u64, GameOptions) -> Box<dyn Game>;

/// Everything needed to rebuild a session: the deal, its options, the move log and the history cursor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRecord {
    pub variant: Variant,
    pub seed: u64,
    #[serde(default, skip_serializing_if = "is_default_options")]
    pub options: GameOptions,
    pub moves: Vec<Move>,
    /// Number of moves currently applied; moves after it can be redone.
    pub cursor: usize,
}

fn is_default_options(options: &GameOptions) -> bool {
    options.is_default()
}

/// Why a [`SessionRecord`] cannot be restored.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RestoreError {
    #[error("history cursor {cursor} is beyond the {len} recorded moves")]
    CursorBeyondMoves { cursor: usize, len: usize },
    #[error("recorded move {index} was rejected while replaying")]
    MoveRejected {
        index: usize,
        #[source]
        source: MoveError,
    },
}

/// A game plus its move log. The game is always the deal with `moves[..cursor]` applied.
pub struct Session {
    variant: Variant,
    seed: u64,
    options: GameOptions,
    dealer: Dealer,
    game: Box<dyn Game>,
    moves: Vec<Move>,
    cursor: usize,
}

fn replay(
    dealer: Dealer,
    variant: Variant,
    seed: u64,
    options: GameOptions,
    moves: &[Move],
) -> Result<Box<dyn Game>, RestoreError> {
    let mut game = dealer(variant, seed, options);
    for (index, &mv) in moves.iter().enumerate() {
        game.apply(mv)
            .map_err(|source| RestoreError::MoveRejected { index, source })?;
    }
    Ok(game)
}

impl Session {
    /// Starts a fresh deal of `variant` from `seed` with default options.
    pub fn new(variant: Variant, seed: u64) -> Self {
        Self::with_options(variant, seed, GameOptions::DEFAULT)
    }

    /// Starts a fresh deal of `variant` from `seed` with `options` (normalized to `variant`).
    pub fn with_options(variant: Variant, seed: u64, options: GameOptions) -> Self {
        Self::with_dealer(variant, seed, options.for_variant(variant), deal_with)
    }

    pub(crate) fn with_dealer(
        variant: Variant,
        seed: u64,
        options: GameOptions,
        dealer: Dealer,
    ) -> Self {
        let options = options.for_variant(variant);
        Self {
            variant,
            seed,
            options,
            dealer,
            game: dealer(variant, seed, options),
            moves: Vec::new(),
            cursor: 0,
        }
    }

    /// Rebuilds a session by replaying `record`. The whole log is validated, including the
    /// part after the cursor that can still be redone.
    pub fn restore(record: SessionRecord) -> Result<Self, RestoreError> {
        Self::restore_with(record, deal_with)
    }

    pub(crate) fn restore_with(
        record: SessionRecord,
        dealer: Dealer,
    ) -> Result<Self, RestoreError> {
        let SessionRecord {
            variant,
            seed,
            options,
            moves,
            cursor,
        } = record;
        let options = options.for_variant(variant);
        if cursor > moves.len() {
            return Err(RestoreError::CursorBeyondMoves {
                cursor,
                len: moves.len(),
            });
        }
        let applied = moves.get(..cursor).unwrap_or_default();
        let game = replay(dealer, variant, seed, options, applied)?;
        if cursor < moves.len() {
            replay(dealer, variant, seed, options, &moves)?;
        }
        Ok(Self {
            variant,
            seed,
            options,
            dealer,
            game,
            moves,
            cursor,
        })
    }

    /// The serializable state of this session.
    pub fn record(&self) -> SessionRecord {
        SessionRecord {
            variant: self.variant,
            seed: self.seed,
            options: self.options,
            moves: self.moves.clone(),
            cursor: self.cursor,
        }
    }

    /// Applies `mv` and drops any moves that could have been redone. A rejected move changes
    /// neither the game nor the history.
    pub fn apply(&mut self, mv: Move) -> Result<(), MoveError> {
        self.game.apply(mv)?;
        self.moves.truncate(self.cursor);
        self.moves.push(mv);
        self.cursor = self.moves.len();
        Ok(())
    }

    /// Steps one move back. `false` at the start of the history.
    pub fn undo(&mut self) -> bool {
        let Some(target) = self.cursor.checked_sub(1) else {
            return false;
        };
        let applied = self.moves.get(..target).unwrap_or_default();
        let Ok(game) = replay(self.dealer, self.variant, self.seed, self.options, applied) else {
            return false;
        };
        self.game = game;
        self.cursor = target;
        true
    }

    /// Re-applies the next undone move. `false` at the end of the history.
    pub fn redo(&mut self) -> bool {
        let Some(&mv) = self.moves.get(self.cursor) else {
            return false;
        };
        if self.game.apply(mv).is_err() {
            return false;
        }
        self.cursor += 1;
        true
    }

    /// Whether [`Session::undo`] has a move to step back over.
    pub fn can_undo(&self) -> bool {
        self.cursor > 0
    }

    /// Whether [`Session::redo`] has an undone move to re-apply.
    pub fn can_redo(&self) -> bool {
        self.cursor < self.moves.len()
    }

    /// How many moves are applied now: every executed move counts once, an undo takes one back
    /// and a redo puts it again, so it equals the history cursor.
    pub fn moves_made(&self) -> usize {
        self.cursor
    }

    /// Back to the dealt position of the same seed with an empty history.
    pub fn restart(&mut self) {
        self.game = (self.dealer)(self.variant, self.seed, self.options);
        self.moves.clear();
        self.cursor = 0;
    }

    /// The current game, read-only; every change goes through [`Session::apply`].
    pub fn game(&self) -> &dyn Game {
        self.game.as_ref()
    }

    /// The moves that finish the current game when it is a certain win, see
    /// [`Game::forced_finish`]. Nothing is applied: the caller plays them through
    /// [`Session::apply`], so each one is recorded in the history.
    pub fn forced_finish(&self) -> Option<Vec<Move>> {
        self.game.forced_finish()
    }

    /// The variant being played.
    pub fn variant(&self) -> Variant {
        self.variant
    }

    /// The seed of the deal.
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// The options of the deal (normalized to the variant).
    pub fn options(&self) -> GameOptions {
        self.options
    }
}

#[cfg(test)]
mod tests;
