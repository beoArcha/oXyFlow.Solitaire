//! The `Storage` implementation. Each operation is a private function returning the internal
//! `Fault`; the trait methods convert it to `StorageError` at the boundary.

use oxysol_core::{SessionRecord, Variant};
use oxysol_tui::{GameId, GameRecord, Outcome, Proof, SavedGame, Settings, Storage, StorageError};
use rusqlite::{OptionalExtension, params};

use super::convert::{
    count_to_column, millis_from_column, millis_to_column, now_millis, outcome_to_column,
    proof_to_column, seed_to_column, variant_to_column,
};
use super::error::Fault;
use super::{SqliteStorage, history};

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

impl SqliteStorage {
    fn read_settings(&self) -> Result<Option<Settings>, Fault> {
        let json: Option<String> = self
            .conn
            .query_row("SELECT json FROM settings WHERE id = 1", [], |row| {
                row.get(0)
            })
            .optional()?;
        json.map(|text| serde_json::from_str(&text))
            .transpose()
            .map_err(Fault::from)
    }

    fn write_settings(&mut self, settings: &Settings) -> Result<(), Fault> {
        let json = serde_json::to_string(settings)?;
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT OR REPLACE INTO settings (id, json) VALUES (1, ?1)",
            params![json],
        )?;
        tx.commit()?;
        Ok(())
    }

    fn read_game(&self) -> Result<Option<SavedGame>, Fault> {
        let row: Option<(i64, String, i64, Option<i64>)> = self
            .conn
            .query_row(
                "SELECT s.game_id, s.record, s.elapsed_ms, g.undo_allowed
                 FROM saved_game s LEFT JOIN games g ON g.id = s.game_id
                 WHERE s.id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()?;
        let Some((id, record, elapsed_ms, undo_allowed)) = row else {
            return Ok(None);
        };
        let record: SessionRecord = serde_json::from_str(&record)?;
        Ok(Some(SavedGame {
            id: GameId(id),
            record,
            elapsed_ms: millis_from_column(elapsed_ms)?,
            undo_allowed: undo_allowed.is_none_or(|flag| flag != 0),
        }))
    }

    /// The undo mode of a saved game is the one of its `games` row, written when the game was
    /// dealt, so the saved record needs no column of its own.
    fn write_game(&mut self, game: &SavedGame) -> Result<(), Fault> {
        let record = serde_json::to_string(&game.record)?;
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT OR REPLACE INTO saved_game (id, game_id, record, elapsed_ms, app_version)
             VALUES (1, ?1, ?2, ?3, ?4)",
            params![
                game.id.0,
                record,
                millis_to_column(game.elapsed_ms),
                APP_VERSION
            ],
        )?;
        tx.commit()?;
        Ok(())
    }

    fn remove_game(&mut self) -> Result<(), Fault> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM saved_game WHERE id = 1", [])?;
        tx.commit()?;
        Ok(())
    }

    fn insert_game(
        &mut self,
        variant: Variant,
        seed: u64,
        undo_allowed: bool,
    ) -> Result<GameId, Fault> {
        let variant = variant_to_column(variant)?;
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO games (variant, seed, started_at, undo_allowed) VALUES (?1, ?2, ?3, ?4)",
            params![
                variant,
                seed_to_column(seed),
                now_millis(),
                i64::from(undo_allowed)
            ],
        )?;
        let id = tx.last_insert_rowid();
        tx.commit()?;
        Ok(GameId(id))
    }

    fn close_game(
        &mut self,
        id: GameId,
        outcome: Outcome,
        elapsed_ms: u64,
        moves: u32,
    ) -> Result<(), Fault> {
        let tx = self.conn.transaction()?;
        let changed = tx.execute(
            "UPDATE games SET outcome = ?1, finished_at = ?2, elapsed_ms = ?3, moves = ?4
             WHERE id = ?5",
            params![
                outcome_to_column(outcome),
                now_millis(),
                millis_to_column(elapsed_ms),
                count_to_column(moves),
                id.0
            ],
        )?;
        if changed == 0 {
            return Err(Fault::invalid(format!("no game with id {}", id.0)));
        }
        tx.commit()?;
        Ok(())
    }
}

impl SqliteStorage {
    fn bump_restarts(&mut self, id: GameId) -> Result<(), Fault> {
        let tx = self.conn.transaction()?;
        let changed = tx.execute(
            "UPDATE games SET restarts = restarts + 1 WHERE id = ?1",
            params![id.0],
        )?;
        if changed == 0 {
            return Err(Fault::invalid(format!("no game with id {}", id.0)));
        }
        tx.commit()?;
        Ok(())
    }

    fn write_proof(&mut self, id: GameId, proof: Proof) -> Result<(), Fault> {
        let tx = self.conn.transaction()?;
        let changed = tx.execute(
            "UPDATE games SET proof = ?1 WHERE id = ?2",
            params![proof_to_column(proof), id.0],
        )?;
        if changed == 0 {
            return Err(Fault::invalid(format!("no game with id {}", id.0)));
        }
        tx.commit()?;
        Ok(())
    }
}

impl Storage for SqliteStorage {
    fn load_settings(&self) -> Result<Option<Settings>, StorageError> {
        Ok(self.read_settings()?)
    }

    fn save_settings(&mut self, settings: &Settings) -> Result<(), StorageError> {
        Ok(self.write_settings(settings)?)
    }

    fn load_game(&self) -> Result<Option<SavedGame>, StorageError> {
        Ok(self.read_game()?)
    }

    fn save_game(&mut self, game: &SavedGame) -> Result<(), StorageError> {
        Ok(self.write_game(game)?)
    }

    fn clear_game(&mut self) -> Result<(), StorageError> {
        Ok(self.remove_game()?)
    }

    fn start_game(
        &mut self,
        variant: Variant,
        seed: u64,
        undo_allowed: bool,
    ) -> Result<GameId, StorageError> {
        Ok(self.insert_game(variant, seed, undo_allowed)?)
    }

    fn finish_game(
        &mut self,
        id: GameId,
        outcome: Outcome,
        elapsed_ms: u64,
        moves: u32,
    ) -> Result<(), StorageError> {
        Ok(self.close_game(id, outcome, elapsed_ms, moves)?)
    }

    fn record_restart(&mut self, id: GameId) -> Result<(), StorageError> {
        Ok(self.bump_restarts(id)?)
    }

    fn record_proof(&mut self, id: GameId, proof: Proof) -> Result<(), StorageError> {
        Ok(self.write_proof(id, proof)?)
    }

    fn history(&self) -> Result<Vec<GameRecord>, StorageError> {
        Ok(history::read_all(&self.conn)?)
    }
}
