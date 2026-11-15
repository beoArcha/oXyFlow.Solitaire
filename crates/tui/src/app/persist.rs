//! Autosave: keeps the stored game, the history and the in-memory copy of the save in step with
//! the board. A write that fails is logged and alerted once per session; the game goes on.

use std::time::Duration;

use super::{App, Modal, Screen};
use crate::storage::{GameId, Outcome, SavedGame, StorageError};

const WRITE_FAILED_ALERT: &str = "common.error.storage";

/// Undo is always available until a "no undo" mode exists.
pub(super) const UNDO_ALLOWED: bool = true;

fn millis(elapsed: Duration) -> u64 {
    u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
}

fn move_count(moves: usize) -> u32 {
    u32::try_from(moves).unwrap_or(u32::MAX)
}

/// The game that could not be restored: its id, playing time and move count.
pub(super) type Unusable = (GameId, u64, u32);

impl App {
    /// Saves or finishes the game after the board reported a change of the session.
    pub(super) fn sync_board(&mut self) {
        if !self.board.take_dirty() {
            return;
        }
        if self.board.is_won() {
            self.finish_won();
        } else {
            self.save_board();
        }
    }

    /// Writes the running game with the playing time so far. Does nothing without a recorded
    /// game or for a solved one.
    pub(super) fn save_board(&mut self) {
        let (Some(id), Some(session)) = (self.board.game_id(), self.board.session()) else {
            return;
        };
        if self.board.is_won() {
            return;
        }
        let game = SavedGame {
            id,
            record: session.record(),
            elapsed_ms: millis(self.board.elapsed(self.now)),
        };
        let result = self.storage.save_game(&game);
        self.saved = Some(game);
        self.note_write(result, "the game could not be saved");
    }

    fn finish_won(&mut self) {
        let Some(id) = self.board.game_id() else {
            return;
        };
        if self.saved.take().is_none() {
            return;
        }
        let elapsed_ms = millis(self.board.elapsed(self.now));
        let moves = self
            .board
            .session()
            .map_or(0, |s| move_count(s.moves_made()));
        let finished = self
            .storage
            .finish_game(id, Outcome::Won, elapsed_ms, moves);
        self.note_write(finished, "the won game could not be recorded");
        let cleared = self.storage.clear_game();
        self.note_write(cleared, "the saved game could not be removed");
    }

    /// Closes the stored game as abandoned before another one is dealt over it.
    pub(super) fn abandon_saved(&mut self) {
        self.save_board();
        let Some(saved) = self.saved.take() else {
            return;
        };
        let moves = move_count(saved.record.cursor);
        let finished =
            self.storage
                .finish_game(saved.id, Outcome::Abandoned, saved.elapsed_ms, moves);
        self.note_write(finished, "the abandoned game could not be recorded");
        let cleared = self.storage.clear_game();
        self.note_write(cleared, "the saved game could not be removed");
    }

    /// Drops a save that cannot be used. The caller alerts the player; a failure here is only
    /// logged, once, because there is nothing left to protect.
    pub(super) fn discard_unusable_save(&mut self, game: Option<Unusable>) {
        self.saved = None;
        let finished = match game {
            Some((id, elapsed_ms, moves)) => {
                self.storage
                    .finish_game(id, Outcome::Abandoned, elapsed_ms, moves)
            }
            None => Ok(()),
        };
        let result = finished.and(self.storage.clear_game());
        if let Err(error) = result {
            tracing::warn!(%error, "the unusable saved game could not be removed");
        }
    }

    /// Reports a failed write once per session: one log record and one alert.
    pub(super) fn note_write(&mut self, result: Result<(), StorageError>, what: &'static str) {
        let Err(error) = result else {
            return;
        };
        if self.write_failed {
            return;
        }
        self.write_failed = true;
        tracing::warn!(%error, "{what}");
        if self.modal.is_none() {
            self.open_modal(Modal::alert(WRITE_FAILED_ALERT));
        }
    }

    /// Shows the board; a modal that is open keeps the clock stopped.
    pub(super) fn enter_board(&mut self) {
        self.goto(Screen::Board);
        if self.modal.is_some() {
            self.board.pause_clock(self.now);
        }
    }
}
