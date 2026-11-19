//! The persistence contract. The application (`oxysol`) implements it over SQLite; the TUI only
//! calls it and decides how to react to a failure (alert, defaults) and logs it there.

use oxysol_core::{SessionRecord, Variant};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::settings::Settings;
use crate::solver::Proof;

pub mod memory;

/// Identifier of one dealt game in the history.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GameId(pub i64);

/// How a recorded game ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Won,
    Abandoned,
}

/// The counters kept with one history row (P4-05). A later counter is a new field here, so the
/// [`Storage`] trait keeps its shape.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GameCounters {
    /// Restarts of a game played without undo. A game with undo never counts one, so it stays 0.
    pub restarts: u32,
}

/// One dealt game as recorded in the history (R-18).
///
/// `moves` counts the moves in the line of play when the game ended: each executed move is one,
/// an undo takes its move back and a redo puts it again (so it is the history cursor). Rows
/// written before the count existed have `None`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameRecord {
    pub id: GameId,
    pub variant: Variant,
    pub seed: u64,
    /// UTC, Unix milliseconds.
    pub started_at_ms: i64,
    /// UTC, Unix milliseconds; `None` while the game is open.
    pub finished_at_ms: Option<i64>,
    pub outcome: Option<Outcome>,
    pub elapsed_ms: Option<u64>,
    pub moves: Option<u32>,
    /// Whether undo and redo were available in this game.
    pub undo_allowed: bool,
    /// What the solver said about the deal; `None` while it has not been checked.
    pub proof: Option<Proof>,
    pub counters: GameCounters,
}

const fn undo_allowed_by_default() -> bool {
    true
}

/// The game that resumes on the next start. `undo_allowed` is the undo mode the game was dealt
/// with; a save written before the mode existed loads as `true`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedGame {
    pub id: GameId,
    pub record: SessionRecord,
    pub elapsed_ms: u64,
    #[serde(default = "undo_allowed_by_default")]
    pub undo_allowed: bool,
}

/// Why a storage operation failed. A missing value is `Ok(None)`, never an error.
///
/// The backend's own error type cannot appear here (the TUI must not depend on `rusqlite`), so an
/// implementation maps it to `Backend` with the message, or to `Corrupt` / `Incompatible`.
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("storage I/O failed")]
    Io(#[from] std::io::Error),
    #[error("stored data is unreadable: {reason}")]
    Corrupt { reason: String },
    #[error("stored data has schema version {found}, this build supports up to {supported}")]
    Incompatible { found: u32, supported: u32 },
    #[error("storage backend failed: {message}")]
    Backend { message: String },
}

/// Settings, the saved game and the game history behind one interface.
pub trait Storage {
    /// The stored settings, `None` on a first run.
    fn load_settings(&self) -> Result<Option<Settings>, StorageError>;
    /// Replaces the stored settings.
    fn save_settings(&mut self, settings: &Settings) -> Result<(), StorageError>;
    /// The stored game to resume, if any.
    fn load_game(&self) -> Result<Option<SavedGame>, StorageError>;
    /// Replaces the stored game.
    fn save_game(&mut self, game: &SavedGame) -> Result<(), StorageError>;
    /// Removes the stored game.
    fn clear_game(&mut self) -> Result<(), StorageError>;
    /// Records a new deal in the history and returns its id. `undo_allowed` is the undo mode of
    /// the game.
    fn start_game(
        &mut self,
        variant: Variant,
        seed: u64,
        undo_allowed: bool,
    ) -> Result<GameId, StorageError>;
    /// Closes a history row with its outcome, playing time and move count.
    fn finish_game(
        &mut self,
        id: GameId,
        outcome: Outcome,
        elapsed_ms: u64,
        moves: u32,
    ) -> Result<(), StorageError>;
    /// Adds one to the restart count of a history row, when the player restarts a game that is
    /// played without undo. It is written at once, so a quit or a crash cannot lose it.
    fn record_restart(&mut self, id: GameId) -> Result<(), StorageError>;
    /// Records what the solver found out about the deal of a history row.
    fn record_proof(&mut self, id: GameId, proof: Proof) -> Result<(), StorageError>;
    /// Every recorded game in start order, oldest first.
    fn history(&self) -> Result<Vec<GameRecord>, StorageError>;
}

#[cfg(test)]
mod tests {
    use oxysol_core::{Session, Variant};

    use super::*;

    #[test]
    fn a_save_written_before_the_undo_mode_existed_loads_with_undo_allowed() {
        let saved = SavedGame {
            id: GameId(1),
            record: Session::new(Variant::FreeCell, 3).record(),
            elapsed_ms: 10,
            undo_allowed: false,
        };
        let text = toml::to_string(&saved).unwrap_or_default();
        assert!(text.contains("undo_allowed = false"), "{text}");
        let old: String = text
            .lines()
            .filter(|line| !line.starts_with("undo_allowed"))
            .collect::<Vec<_>>()
            .join("\n");
        let loaded = toml::from_str::<SavedGame>(&old);
        assert!(
            matches!(loaded, Ok(ref game) if game.undo_allowed),
            "{loaded:?}"
        );
        let current = toml::from_str::<SavedGame>(&text);
        assert!(matches!(current, Ok(ref game) if *game == saved));
    }
}
