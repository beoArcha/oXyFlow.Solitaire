//! The scan of one variant: the pool of winnable seeds, the hint playout and the replay check.

use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use oxysol_core::curated::PER_LEVEL;
use oxysol_core::solver::{SolveBudget, Verdict, solve_report};
use oxysol_core::{Move, PileSnapshot, Variant, deal};

use super::CuratedSeedsError;
use super::score::{Candidate, LEVELS, Scored, name, pick, score_pool};

const POOL_TARGET: usize = 60;
const FIRST_SCAN_END: u64 = 999;
const LAST_SEED: u64 = 4999;
const HINT_STEP_CAP: usize = 2_000;

/// The result of scanning one variant: the sorted pool and the picked seeds per level.
pub struct VariantScan {
    pub variant: Variant,
    pub scanned_to: u64,
    pub sorted: Vec<Scored>,
    pub picks: [[usize; PER_LEVEL]; LEVELS],
}

impl VariantScan {
    pub fn pick(&self, level: usize, slot: usize) -> &Scored {
        &self.sorted[self.picks[level][slot]]
    }
}

/// Scans seeds from 0 until the pool holds 60 winnable seeds, or seed 999 has passed with at least
/// 15, or seed 4999 is reached; then picks and re-verifies the seeds of every level.
pub fn variant(variant: Variant) -> Result<VariantScan, CuratedSeedsError> {
    let budget = SolveBudget::default_for(variant);
    let mut pool = Vec::new();
    let mut scanned_to = 0;
    for seed in 0..=LAST_SEED {
        let enough = pool.len() >= POOL_TARGET;
        let enough_after_first_scan = seed > FIRST_SCAN_END && pool.len() >= LEVELS * PER_LEVEL;
        if enough || enough_after_first_scan {
            break;
        }
        scanned_to = seed;
        let report = solve_report(variant, seed, budget);
        if let Verdict::Winnable { moves } = report.verdict {
            let hint_win = hint_wins(variant, seed)?;
            pool.push(Candidate {
                seed,
                effort: report.nodes,
                length: u64::try_from(moves.len()).unwrap_or(u64::MAX),
                hint_win,
                moves,
            });
        }
    }
    let found = pool.len();
    let sorted = score_pool(pool);
    let picks = pick(&sorted).ok_or_else(|| CuratedSeedsError::TooFewWinnable {
        variant: name(variant),
        found,
        last: scanned_to,
        needed: LEVELS * PER_LEVEL,
    })?;
    let scan = VariantScan {
        variant,
        scanned_to,
        sorted,
        picks,
    };
    for level in 0..LEVELS {
        for slot in 0..PER_LEVEL {
            let chosen = &scan.pick(level, slot).candidate;
            if !replays(variant, chosen.seed, &chosen.moves) {
                return Err(CuratedSeedsError::NotReplayable {
                    variant: name(variant),
                    seed: chosen.seed,
                });
            }
        }
    }
    Ok(scan)
}

/// Whether the move list wins the deal of `variant` and `seed` when applied in order.
pub fn replays(variant: Variant, seed: u64, moves: &[Move]) -> bool {
    let mut game = deal(variant, seed);
    moves.iter().all(|&mv| game.apply(mv).is_ok()) && game.is_won()
}

/// Plays the deal with `Game::hint()` alone; true when it is won within the step cap, stopping at a
/// missing hint or a repeated position.
fn hint_wins(variant: Variant, seed: u64) -> Result<bool, CuratedSeedsError> {
    let mut game = deal(variant, seed);
    let mut seen = HashSet::new();
    seen.insert(hash_piles(&game.piles()));
    let mut steps = 0;
    loop {
        if game.is_won() {
            return Ok(true);
        }
        if steps >= HINT_STEP_CAP {
            return Ok(false);
        }
        let Some(mv) = game.hint() else {
            return Ok(false);
        };
        game.apply(mv).map_err(|_| CuratedSeedsError::IllegalHint {
            variant: name(variant),
            seed,
        })?;
        steps += 1;
        if !seen.insert(hash_piles(&game.piles())) {
            return Ok(false);
        }
    }
}

fn hash_piles(piles: &[PileSnapshot]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for pile in piles {
        pile.pile.hash(&mut hasher);
        pile.cards.len().hash(&mut hasher);
        for card in &pile.cards {
            card.card.id.0.hash(&mut hasher);
            card.face_up.hash(&mut hasher);
        }
    }
    hasher.finish()
}
