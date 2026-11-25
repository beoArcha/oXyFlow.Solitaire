//! The scoring of T-49: percentiles, the difficulty D, the sorted pool and the quintile picks.

use oxysol_core::{Move, Variant};

use oxysol_core::curated::PER_LEVEL;

/// Five levels, so the pool needs at least this many seeds for three per level.
pub const LEVELS: usize = 5;

/// A winnable seed of the pool with the metrics of T-49.
pub struct Candidate {
    pub seed: u64,
    pub effort: u64,
    pub length: u64,
    pub hint_win: bool,
    pub moves: Vec<Move>,
}

/// A candidate with its difficulty D.
pub struct Scored {
    pub candidate: Candidate,
    pub difficulty: f64,
}

/// The variant name used in messages and the report, for example `Klondike(One)`.
pub fn name(variant: Variant) -> String {
    format!("{variant:?}")
}

/// Rank percentile in 0..1 of every value: the smallest is 0, the largest 1, and equal values share
/// the average of their ranks. A pool of one value gives 0.
pub fn percentiles(values: &[u64]) -> Vec<f64> {
    let count = values.len();
    if count < 2 {
        return vec![0.0; count];
    }
    let mut order: Vec<usize> = (0..count).collect();
    order.sort_by_key(|&index| values[index]);
    let mut out = vec![0.0; count];
    let mut start = 0;
    while start < count {
        let value = values[order[start]];
        let mut end = start + 1;
        while end < count && values[order[end]] == value {
            end += 1;
        }
        let average_rank = (start + 1 + end) as f64 / 2.0;
        for &index in &order[start..end] {
            out[index] = (average_rank - 1.0) / (count - 1) as f64;
        }
        start = end;
    }
    out
}

/// D = 0.6 · pct(E) + 0.4 · pct(L) − 0.3 · H for every candidate, sorted by D and then by seed.
pub fn score_pool(pool: Vec<Candidate>) -> Vec<Scored> {
    let effort: Vec<u64> = pool.iter().map(|candidate| candidate.effort).collect();
    let length: Vec<u64> = pool.iter().map(|candidate| candidate.length).collect();
    let effort_pct = percentiles(&effort);
    let length_pct = percentiles(&length);
    let mut scored: Vec<Scored> = pool
        .into_iter()
        .zip(effort_pct.into_iter().zip(length_pct))
        .map(|(candidate, (effort, length))| {
            let hint = if candidate.hint_win { 0.3 } else { 0.0 };
            let difficulty = 0.6 * effort + 0.4 * length - hint;
            Scored {
                candidate,
                difficulty,
            }
        })
        .collect();
    scored.sort_by(|a, b| {
        a.difficulty
            .total_cmp(&b.difficulty)
            .then(a.candidate.seed.cmp(&b.candidate.seed))
    });
    scored
}

/// For each level, the indexes into `sorted` (which must be sorted by D) of the three seeds nearest
/// the median D of its quintile, ties by seed, listed in seed order. `None` below 15 seeds.
pub fn pick(sorted: &[Scored]) -> Option<[[usize; PER_LEVEL]; LEVELS]> {
    let count = sorted.len();
    if count < LEVELS * PER_LEVEL {
        return None;
    }
    let mut levels = [[0; PER_LEVEL]; LEVELS];
    for (level, slots) in levels.iter_mut().enumerate() {
        let start = level * count / LEVELS;
        let end = (level + 1) * count / LEVELS;
        let median = median(&sorted[start..end]);
        let mut nearest: Vec<usize> = (start..end).collect();
        nearest.sort_by(|&a, &b| {
            let distance_a = (sorted[a].difficulty - median).abs();
            let distance_b = (sorted[b].difficulty - median).abs();
            distance_a
                .total_cmp(&distance_b)
                .then(sorted[a].candidate.seed.cmp(&sorted[b].candidate.seed))
        });
        nearest.truncate(PER_LEVEL);
        nearest.sort_by_key(|&index| sorted[index].candidate.seed);
        for (slot, index) in slots.iter_mut().zip(nearest) {
            *slot = index;
        }
    }
    Some(levels)
}

fn median(scored: &[Scored]) -> f64 {
    let mut values: Vec<f64> = scored.iter().map(|entry| entry.difficulty).collect();
    values.sort_by(f64::total_cmp);
    let middle = values.len() / 2;
    match values.len() % 2 {
        1 => values[middle],
        _ => (values[middle - 1] + values[middle]) / 2.0,
    }
}
