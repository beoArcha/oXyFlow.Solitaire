//! The curated levels of the seed screen, read through a fake table so that the tests do not depend
//! on the shipped one (decision T-49).

use super::seed_levels::{LevelSeed, entries, rows};
use crate::i18n::{t, tf};
use oxysol_core::Variant;
use oxysol_core::curated::{self, Difficulty};

fn fake(variant: Variant, level: Difficulty) -> &'static [u64] {
    match (variant, level) {
        (Variant::Golf, Difficulty::VeryEasy) => &[1, 2, 3],
        (Variant::Golf, Difficulty::Hard) => &[11, 12, 13],
        _ => &[],
    }
}

#[test]
fn the_seeds_of_a_variant_run_easiest_level_first() {
    let found = entries(Variant::Golf, fake);
    let seeds: Vec<u64> = found.iter().map(|entry| entry.seed).collect();
    assert_eq!(seeds, vec![1, 2, 3, 11, 12, 13]);
    assert_eq!(found[0].level, Difficulty::VeryEasy);
    assert_eq!(found[5].level, Difficulty::Hard);
}

#[test]
fn a_variant_without_entries_has_no_seeds() {
    assert!(entries(Variant::FreeCell, fake).is_empty());
}

#[test]
fn each_seed_is_one_row_named_by_its_level() {
    let found = [LevelSeed {
        level: Difficulty::Hard,
        seed: 11,
    }];
    let level = t("menu.seed.level.hard").into_owned();
    let expected = tf(
        "menu.seed.level_seed",
        &[("level", level.as_str()), ("seed", "11")],
    );
    let labels: Vec<String> = rows(&found).iter().map(|row| format!("{row:?}")).collect();
    assert_eq!(labels.len(), 1);
    assert!(labels[0].contains(&expected), "{labels:?} / {expected}");
}

#[test]
fn no_seeds_make_no_rows() {
    assert!(rows(&[]).is_empty());
}

#[test]
fn the_shipped_table_gives_three_seeds_per_level_or_nothing() {
    for variant in Variant::ALL {
        let found = entries(variant, curated::seeds);
        if found.is_empty() {
            continue;
        }
        for level in Difficulty::ALL {
            let count = found.iter().filter(|entry| entry.level == level).count();
            assert_eq!(count, curated::PER_LEVEL, "{variant:?} {level:?}");
        }
    }
}
