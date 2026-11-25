//! Curated deals by difficulty (P5-06, decision T-49): for every game variant, five levels with
//! three seeds each, every one proven winnable by the solver. The table is generated offline by
//! `cargo xtask-art curated-seeds` from solver and hint measurements and checked into
//! [`table`]; this module only reads it.

mod table;

use crate::game::Variant;

/// Seeds per difficulty level of one variant.
pub const PER_LEVEL: usize = 3;

/// How hard a curated deal is, from the easiest to the hardest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Difficulty {
    VeryEasy,
    Easy,
    Medium,
    Hard,
    VeryHard,
}

impl Difficulty {
    /// Every level, easiest first.
    pub const ALL: [Difficulty; 5] = [
        Difficulty::VeryEasy,
        Difficulty::Easy,
        Difficulty::Medium,
        Difficulty::Hard,
        Difficulty::VeryHard,
    ];

    const fn index(self) -> usize {
        match self {
            Difficulty::VeryEasy => 0,
            Difficulty::Easy => 1,
            Difficulty::Medium => 2,
            Difficulty::Hard => 3,
            Difficulty::VeryHard => 4,
        }
    }
}

/// The curated seeds of `variant` at `level`, or none while the table has no entry for the variant.
pub fn seeds(variant: Variant, level: Difficulty) -> &'static [u64] {
    table::levels(variant)
        .and_then(|levels| levels.get(level.index()))
        .map_or(&[], |seeds| seeds.as_slice())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, HashSet};

    #[test]
    fn the_levels_run_from_easiest_to_hardest() {
        let indexes: Vec<usize> = Difficulty::ALL.iter().map(|level| level.index()).collect();
        assert_eq!(indexes, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn every_variant_has_three_distinct_seeds_on_every_level() {
        for variant in Variant::ALL {
            let all: Vec<u64> = Difficulty::ALL
                .iter()
                .flat_map(|&level| seeds(variant, level).iter().copied())
                .collect();
            for level in Difficulty::ALL {
                assert_eq!(
                    seeds(variant, level).len(),
                    PER_LEVEL,
                    "{variant:?} {level:?}"
                );
            }
            let distinct: HashSet<u64> = all.iter().copied().collect();
            assert_eq!(distinct.len(), all.len(), "{variant:?}");
        }
    }

    #[test]
    fn the_mean_difficulty_rises_from_level_to_level_in_the_report() {
        let report = include_str!("../../../../docs/tech/curated-seeds.md");
        let mut by_variant: BTreeMap<&str, [Vec<f64>; 5]> = BTreeMap::new();
        let mut variant = "";
        for line in report.lines() {
            if let Some(heading) = line.strip_prefix("## ") {
                variant = heading;
                continue;
            }
            let cells: Vec<&str> = line.split('|').map(str::trim).collect();
            let level = Difficulty::ALL.iter().position(|level| {
                let label = match level {
                    Difficulty::VeryEasy => "very easy",
                    Difficulty::Easy => "easy",
                    Difficulty::Medium => "medium",
                    Difficulty::Hard => "hard",
                    Difficulty::VeryHard => "very hard",
                };
                cells.get(1) == Some(&label)
            });
            let difficulty = cells.get(6).and_then(|cell| cell.parse::<f64>().ok());
            if let (Some(level), Some(difficulty)) = (level, difficulty) {
                by_variant.entry(variant).or_default()[level].push(difficulty);
            }
        }
        assert_eq!(by_variant.len(), Variant::ALL.len());
        for (variant, levels) in &by_variant {
            let means: Vec<f64> = levels
                .iter()
                .map(|values| values.iter().sum::<f64>() / values.len() as f64)
                .collect();
            assert!(
                means.windows(2).all(|pair| pair[0] < pair[1]),
                "{variant}: {means:?}"
            );
        }
    }
}
