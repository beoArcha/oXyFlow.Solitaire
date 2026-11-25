//! Every curated seed is proven winnable at the default budget, and its move list wins the deal
//! when replayed. Solves 210 deals, so it runs in release only.

use oxysol_core::curated::{Difficulty, seeds};
use oxysol_core::solver::{SolveBudget, Verdict, solve_report};
use oxysol_core::{Move, Variant, deal};

fn winning_moves(verdict: Verdict) -> Option<Vec<Move>> {
    match verdict {
        Verdict::Winnable { moves } => Some(moves),
        Verdict::Unwinnable | Verdict::Unknown => None,
    }
}

#[test]
#[ignore = "slow in debug: solves 210 curated deals; release run about 23 s"]
fn every_curated_seed_is_winnable_and_replays() {
    for variant in Variant::ALL {
        for level in Difficulty::ALL {
            for &seed in seeds(variant, level) {
                let report = solve_report(variant, seed, SolveBudget::default_for(variant));
                let moves = winning_moves(report.verdict);
                assert!(
                    moves.is_some(),
                    "{variant:?} {level:?} seed {seed} is not proven winnable"
                );
                let mut game = deal(variant, seed);
                for mv in moves.unwrap_or_default() {
                    assert_eq!(game.apply(mv), Ok(()), "{variant:?} seed {seed}");
                }
                assert!(
                    game.is_won(),
                    "{variant:?} {level:?} seed {seed} does not win"
                );
            }
        }
    }
}
