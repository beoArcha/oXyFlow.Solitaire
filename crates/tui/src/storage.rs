//! The persistence contract. The application (`oxysol`) implements it over SQLite; the TUI only
//! calls it and decides how to react to a failure (alert, defaults) and logs it there.

use oxysol_core::{SessionRecord, Variant};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::settings::Settings;

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
    /// Whether undo was available in this game; always `true` until a "no undo" mode exists.
    pub undo_allowed: bool,
}

/// The game that resumes on the next start.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedGame {
    pub id: GameId,
    pub record: SessionRecord,
    pub elapsed_ms: u64,
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
    /// Every recorded game in start order, oldest first.
    fn history(&self) -> Result<Vec<GameRecord>, StorageError>;
}
