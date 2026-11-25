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
    use std::collections::HashSet;

    #[test]
    fn the_levels_run_from_easiest_to_hardest() {
        let indexes: Vec<usize> = Difficulty::ALL.iter().map(|level| level.index()).collect();
        assert_eq!(indexes, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn a_variant_in_the_table_has_three_distinct_seeds_on_every_level() {
        for variant in Variant::ALL {
            let all: Vec<u64> = Difficulty::ALL
                .iter()
                .flat_map(|&level| seeds(variant, level).iter().copied())
                .collect();
            if all.is_empty() {
                continue;
            }
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
}
