//! Hint playout benchmark: `cargo test -p oxysol-core --release --test hint_perf -- --ignored
//! --nocapture`. Plays seeds 0..=199 per variant using `Game::hint()` only and prints wins,
//! losses/stalls, repeated-position loops, and timing.

use oxysol_core::{PileSnapshot, Variant, deal};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};

const SEEDS: u64 = 200;
const STEP_CAP: usize = 2_000;

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

#[derive(Default)]
struct Tally {
    won: u32,
    stuck: u32,
    repeated: u32,
    capped: u32,
    total_steps: u64,
    total_time: Duration,
    slowest_hint: Duration,
}

impl Tally {
    fn line(&self, variant: Variant) -> String {
        let pct = |n: u32| f64::from(n) * 100.0 / SEEDS as f64;
        format!(
            "{variant:?}: won {}/{} ({:.1}%), stuck {}, repeated {}, capped {} | avg steps {}, total {:.0} ms, slowest hint {:.2} ms",
            self.won,
            SEEDS,
            pct(self.won),
            self.stuck,
            self.repeated,
            self.capped,
            self.total_steps / SEEDS,
            self.total_time.as_secs_f64() * 1000.0,
            self.slowest_hint.as_secs_f64() * 1000.0,
        )
    }
}

#[test]
#[ignore = "benchmark: run with --release -- --ignored --nocapture"]
fn play_two_hundred_seeds_by_hints_only() {
    let only = std::env::var("OXYSOL_HINT_ONLY").unwrap_or_default();
    for variant in Variant::ALL {
        if !format!("{variant:?}").contains(&only) {
            continue;
        }
        let mut tally = Tally::default();
        for seed in 0..SEEDS {
            let mut game = deal(variant, seed);
            let mut seen = HashSet::new();
            seen.insert(hash_piles(&game.piles()));
            let mut steps = 0usize;
            loop {
                if game.is_won() {
                    tally.won += 1;
                    break;
                }
                if steps >= STEP_CAP {
                    tally.capped += 1;
                    break;
                }
                let t0 = Instant::now();
                let hint = game.hint();
                let took = t0.elapsed();
                tally.total_time += took;
                tally.slowest_hint = tally.slowest_hint.max(took);
                let Some(mv) = hint else {
                    tally.stuck += 1;
                    break;
                };
                assert_eq!(game.apply(mv), Ok(()), "{variant:?} seed {seed}: {mv:?}");
                steps += 1;
                if !seen.insert(hash_piles(&game.piles())) {
                    tally.repeated += 1;
                    break;
                }
            }
            tally.total_steps += u64::try_from(steps).unwrap_or(0);
        }
        println!("{}", tally.line(variant));
    }
}
