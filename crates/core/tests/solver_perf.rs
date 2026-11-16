//! The solver benchmark: `cargo test -p oxysol-core --release --test solver_perf -- --ignored
//! --nocapture`. Solves 50 seeds per variant with the default budget and prints the share of each
//! verdict and the time per deal. `OXYSOL_SOLVER_NODES` overrides the budget (a node count) and
//! `OXYSOL_SOLVER_ONLY` keeps the variants whose debug name contains the text.

use oxysol_core::solver::{Report, SolveBudget, Verdict, solve_report};
use oxysol_core::{Variant, deal};
use std::time::{Duration, Instant};

const SEEDS: u64 = 50;

#[derive(Default)]
struct Tally {
    winnable: u32,
    unwinnable: u32,
    unknown: u32,
    nodes: u64,
    total: Duration,
    slowest: Duration,
}

impl Tally {
    fn add(&mut self, report: &Report, took: Duration) {
        match report.verdict {
            Verdict::Winnable { .. } => self.winnable += 1,
            Verdict::Unwinnable => self.unwinnable += 1,
            Verdict::Unknown => self.unknown += 1,
        }
        self.nodes += report.nodes;
        self.total += took;
        self.slowest = self.slowest.max(took);
    }

    fn line(&self, variant: Variant, budget: u64) -> String {
        let share = |count: u32| f64::from(count) * 100.0 / SEEDS as f64;
        format!(
            "{variant:?} budget {budget}: winnable {:.0}% unwinnable {:.0}% unknown {:.0}% | avg {:.0} ms, slowest {:.0} ms, avg {} nodes",
            share(self.winnable),
            share(self.unwinnable),
            share(self.unknown),
            self.total.as_secs_f64() * 1000.0 / SEEDS as f64,
            self.slowest.as_secs_f64() * 1000.0,
            self.nodes / SEEDS,
        )
    }
}

fn budget_for(variant: Variant) -> u64 {
    std::env::var("OXYSOL_SOLVER_NODES")
        .ok()
        .and_then(|text| text.parse().ok())
        .unwrap_or(SolveBudget::default_for(variant).nodes)
}

#[test]
#[ignore = "benchmark: run with --release -- --ignored --nocapture"]
fn solve_fifty_seeds_per_variant() {
    let only = std::env::var("OXYSOL_SOLVER_ONLY").unwrap_or_default();
    for variant in Variant::ALL {
        if !format!("{variant:?}").contains(&only) {
            continue;
        }
        let budget = budget_for(variant);
        let mut tally = Tally::default();
        for seed in 0..SEEDS {
            let started = Instant::now();
            let report = solve_report(variant, seed, SolveBudget::nodes(budget));
            tally.add(&report, started.elapsed());
            if let Verdict::Winnable { moves } = &report.verdict {
                let mut game = deal(variant, seed);
                assert!(moves.iter().all(|&mv| game.apply(mv).is_ok()) && game.is_won());
            }
        }
        println!("{}", tally.line(variant, budget));
    }
}
