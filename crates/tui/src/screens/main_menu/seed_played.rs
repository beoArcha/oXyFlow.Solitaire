//! The games already played, as the seed screen offers them: the played-seeds page (newest first,
//! each row with its result and local time) and the random replay, which picks one of them.

use std::cmp::Reverse;

use oxysol_core::{SplitMix64, Variant};

use crate::app::Zone;
use crate::i18n::tf;
use crate::screens::stats::{format_date_time, outcome_label};
use crate::storage::{GameRecord, StorageError};
use crate::widgets::menu_list::Row;

/// The games the seed screens read, newest first, or the fact that the history could not be read.
/// `fresh_seed` is the source of the random replay pick, so one menu visit picks one game.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Played {
    games: Vec<GameRecord>,
    unreadable: bool,
    zone: Zone,
    fresh_seed: u64,
}

impl Played {
    /// The history as the storage gives it, oldest first, in the time zone of the statistics.
    pub fn read(
        history: Result<Vec<GameRecord>, StorageError>,
        zone: Zone,
        fresh_seed: u64,
    ) -> Self {
        match history {
            Ok(mut games) => {
                games.reverse();
                games.sort_by_key(|game| Reverse(game.started_at_ms));
                Self {
                    games,
                    unreadable: false,
                    zone,
                    fresh_seed,
                }
            }
            Err(_) => Self {
                unreadable: true,
                zone,
                fresh_seed,
                ..Self::default()
            },
        }
    }

    /// The games of `variant`, with the same zone and pick source.
    pub fn of_variant(&self, variant: Variant) -> Self {
        Self {
            games: self
                .games
                .iter()
                .filter(|game| game.variant == variant)
                .cloned()
                .collect(),
            unreadable: self.unreadable,
            zone: self.zone,
            fresh_seed: self.fresh_seed,
        }
    }

    pub fn is_unreadable(&self) -> bool {
        self.unreadable
    }

    pub fn len(&self) -> usize {
        self.games.len()
    }

    pub fn is_empty(&self) -> bool {
        self.games.is_empty()
    }

    /// The seed of the game at `index`, newest first.
    pub fn seed_at(&self, index: usize) -> Option<u64> {
        self.games.get(index).map(|game| game.seed)
    }

    /// The seed of one played game, picked from the fresh seed; `None` when no game was played.
    pub fn random_seed(&self) -> Option<u64> {
        let count = u64::try_from(self.games.len()).ok()?;
        let index = usize::try_from(SplitMix64::new(self.fresh_seed).below(count)).ok()?;
        self.seed_at(index)
    }

    /// One row per game: `2026-11-20 18:04 · Won · seed 1234`.
    pub fn rows(&self) -> Vec<Row> {
        self.games
            .iter()
            .map(|game| {
                let when = format_date_time(game.started_at_ms, self.zone);
                let result = outcome_label(game.outcome);
                let seed = game.seed.to_string();
                Row::new(tf(
                    "menu.seed.played_row",
                    &[("when", &when), ("result", &result), ("seed", &seed)],
                ))
            })
            .collect()
    }
}
