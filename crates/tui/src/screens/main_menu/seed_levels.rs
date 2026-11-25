//! The difficulty levels of the seed screen: the curated seeds of a game (decision T-49), three per
//! level, easiest level first. The seeds are read through a [`Lookup`], so a test can give the
//! screen seeds of its own; the screen itself passes `oxysol_core::curated::seeds`.

use oxysol_core::Variant;
use oxysol_core::curated::Difficulty;

use crate::i18n::{t, tf};
use crate::widgets::menu_list::Row;

/// Reads the seeds of one level of a variant, as `oxysol_core::curated::seeds` does.
pub type Lookup = fn(Variant, Difficulty) -> &'static [u64];

/// One curated seed and the level it belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelSeed {
    pub level: Difficulty,
    pub seed: u64,
}

/// The curated seeds of `variant` read through `lookup`, easiest level first. Empty while the table
/// has no entry for the variant.
pub fn entries(variant: Variant, lookup: Lookup) -> Vec<LevelSeed> {
    Difficulty::ALL
        .into_iter()
        .flat_map(|level| {
            lookup(variant, level)
                .iter()
                .map(move |&seed| LevelSeed { level, seed })
        })
        .collect()
}

/// Localization key of the name of a level.
pub const fn level_key(level: Difficulty) -> &'static str {
    match level {
        Difficulty::VeryEasy => "menu.seed.level.very_easy",
        Difficulty::Easy => "menu.seed.level.easy",
        Difficulty::Medium => "menu.seed.level.medium",
        Difficulty::Hard => "menu.seed.level.hard",
        Difficulty::VeryHard => "menu.seed.level.very_hard",
    }
}

/// One row per curated seed, named by its level: `Very easy: 1234`.
pub fn rows(entries: &[LevelSeed]) -> Vec<Row> {
    entries
        .iter()
        .map(|entry| {
            let level = t(level_key(entry.level));
            let seed = entry.seed.to_string();
            Row::new(tf(
                "menu.seed.level_seed",
                &[("level", level.as_ref()), ("seed", seed.as_str())],
            ))
        })
        .collect()
}
