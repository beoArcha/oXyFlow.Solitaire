//! What the main menu and the win dialog read from the saved game and the history: the summary that
//! the Resume row names, and the personal bests of a won game.

use std::time::Duration;

use oxysol_core::Variant;

use super::{App, Modal};
use crate::screens::SavedSummary;
use crate::storage::{GameId, GameRecord, Outcome};
use crate::widgets::overlay::PersonalBest;

/// The game that was won, as the personal bests compare it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WonGame {
    pub id: GameId,
    pub variant: Variant,
    pub undo_allowed: bool,
    pub elapsed_ms: u64,
    pub moves: u32,
}

/// Compares a won game with the earlier wins of its variant in its undo mode, the same rows the
/// statistics summarize as one line. A first win has nothing to beat, so it is no personal best.
pub fn personal_best(records: &[GameRecord], game: WonGame) -> PersonalBest {
    let earlier = || {
        records.iter().filter(|record| {
            record.id != game.id
                && record.variant == game.variant
                && record.undo_allowed == game.undo_allowed
                && record.outcome == Some(Outcome::Won)
        })
    };
    let best_time = earlier().filter_map(|record| record.elapsed_ms).min();
    let fewest_moves = earlier().filter_map(|record| record.moves).min();
    PersonalBest {
        time: best_time.is_some_and(|best| game.elapsed_ms < best),
        moves: fewest_moves.is_some_and(|best| game.moves < best),
    }
}

impl App {
    /// The saved game as the Resume row names it; `None` without a save.
    pub(super) fn saved_summary(&self) -> Option<SavedSummary> {
        self.saved.as_ref().map(|saved| SavedSummary {
            variant: saved.record.variant,
            elapsed: Duration::from_millis(saved.elapsed_ms),
            moves: saved.record.cursor,
        })
    }

    /// Fills the personal bests into a win dialog, read from the history. Other dialogs pass through.
    pub(super) fn with_personal_best(&self, modal: Modal) -> Modal {
        match modal {
            Modal::Win { focus, .. } => Modal::Win {
                focus,
                best: self.personal_best(),
            },
            other => other,
        }
    }

    /// The personal bests of the running game. Without a game, or when the history cannot be read,
    /// there are none; the read failure is logged here.
    fn personal_best(&self) -> PersonalBest {
        let (Some(id), Some(session)) = (self.board.game_id(), self.board.session()) else {
            return PersonalBest::default();
        };
        let game = WonGame {
            id,
            variant: session.variant(),
            undo_allowed: self.board.undo_allowed(),
            elapsed_ms: u64::try_from(self.board.elapsed(self.now).as_millis()).unwrap_or(u64::MAX),
            moves: u32::try_from(session.moves_made()).unwrap_or(u32::MAX),
        };
        match self.storage.history() {
            Ok(records) => personal_best(&records, game),
            Err(error) => {
                tracing::warn!(%error, "the personal bests could not be read from the history");
                PersonalBest::default()
            }
        }
    }
}
