//! An in-memory [`Storage`]: the test double for the TUI and the fallback when no database opens.

use oxysol_core::Variant;

use super::{GameId, Outcome, SavedGame, Storage, StorageError};
use crate::settings::Settings;

/// One recorded deal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryRow {
    pub id: GameId,
    pub variant: Variant,
    pub seed: u64,
    pub finished: Option<(Outcome, u64)>,
}

/// Storage that keeps everything in memory. With `failing` set, every call fails, which lets
/// tests exercise the error paths.
#[derive(Debug, Default)]
pub struct MemoryStorage {
    pub settings: Option<Settings>,
    pub game: Option<SavedGame>,
    pub history: Vec<HistoryRow>,
    pub failing: bool,
}

impl MemoryStorage {
    /// An empty storage that works.
    pub fn new() -> Self {
        Self::default()
    }

    fn check(&self) -> Result<(), StorageError> {
        if self.failing {
            return Err(StorageError::Backend {
                message: "injected failure".to_owned(),
            });
        }
        Ok(())
    }
}

impl Storage for MemoryStorage {
    fn load_settings(&self) -> Result<Option<Settings>, StorageError> {
        self.check()?;
        Ok(self.settings.clone())
    }

    fn save_settings(&mut self, settings: &Settings) -> Result<(), StorageError> {
        self.check()?;
        self.settings = Some(settings.clone());
        Ok(())
    }

    fn load_game(&self) -> Result<Option<SavedGame>, StorageError> {
        self.check()?;
        Ok(self.game.clone())
    }

    fn save_game(&mut self, game: &SavedGame) -> Result<(), StorageError> {
        self.check()?;
        self.game = Some(game.clone());
        Ok(())
    }

    fn clear_game(&mut self) -> Result<(), StorageError> {
        self.check()?;
        self.game = None;
        Ok(())
    }

    fn start_game(&mut self, variant: Variant, seed: u64) -> Result<GameId, StorageError> {
        self.check()?;
        let id = GameId(i64::try_from(self.history.len()).unwrap_or(i64::MAX) + 1);
        self.history.push(HistoryRow {
            id,
            variant,
            seed,
            finished: None,
        });
        Ok(id)
    }

    fn finish_game(
        &mut self,
        id: GameId,
        outcome: Outcome,
        elapsed_ms: u64,
    ) -> Result<(), StorageError> {
        self.check()?;
        match self.history.iter_mut().find(|row| row.id == id) {
            Some(row) => {
                row.finished = Some((outcome, elapsed_ms));
                Ok(())
            }
            None => Err(StorageError::Corrupt {
                reason: format!("no game with id {}", id.0),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxysol_core::{DrawCount, Session};

    #[test]
    fn settings_and_game_round_trip() {
        let mut storage = MemoryStorage::new();
        assert!(matches!(storage.load_settings(), Ok(None)));
        let settings = Settings::default();
        assert!(storage.save_settings(&settings).is_ok());
        assert!(matches!(storage.load_settings(), Ok(Some(ref s)) if *s == settings));

        let variant = Variant::Klondike(DrawCount::One);
        let saved = SavedGame {
            id: GameId(1),
            record: Session::new(variant, 7).record(),
            elapsed_ms: 1_500,
        };
        assert!(storage.save_game(&saved).is_ok());
        assert!(matches!(storage.load_game(), Ok(Some(ref g)) if *g == saved));
        assert!(storage.clear_game().is_ok());
        assert!(matches!(storage.load_game(), Ok(None)));
    }

    #[test]
    fn history_rows_get_distinct_ids_and_close() {
        let mut storage = MemoryStorage::new();
        let first = storage.start_game(Variant::FreeCell, 1);
        let second = storage.start_game(Variant::FreeCell, 2);
        let (Ok(first), Ok(second)) = (first, second) else {
            panic!("start_game failed");
        };
        assert_ne!(first, second);
        assert!(storage.finish_game(first, Outcome::Won, 9).is_ok());
        assert_eq!(storage.history[0].finished, Some((Outcome::Won, 9)));
        assert!(storage.finish_game(GameId(99), Outcome::Won, 9).is_err());
    }

    #[test]
    fn failing_storage_fails_every_call() {
        let mut storage = MemoryStorage {
            failing: true,
            ..MemoryStorage::new()
        };
        assert!(storage.load_settings().is_err());
        assert!(storage.save_settings(&Settings::default()).is_err());
        assert!(storage.load_game().is_err());
        assert!(storage.clear_game().is_err());
        assert!(storage.start_game(Variant::FreeCell, 1).is_err());
    }
}
