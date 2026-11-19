//! The public contract: pinned winnable and unwinnable deals, verified move lists, the budget and
//! determinism. Seeds were found with the solver itself; the node counts are the cost of the pin.

use crate::game::{DrawCount, PyramidRules, SuitCount, Variant, deal};
use crate::solver::{SolveBudget, Verdict, replays_to_win, solve, solve_report};

const KLONDIKE_1: Variant = Variant::Klondike(DrawCount::One);
const KLONDIKE_3: Variant = Variant::Klondike(DrawCount::Three);
const SPIDER_1: Variant = Variant::Spider(SuitCount::One);
const SPIDER_2: Variant = Variant::Spider(SuitCount::Two);
const SPIDER_4: Variant = Variant::Spider(SuitCount::Four);
const RELAXED: Variant = Variant::Pyramid(PyramidRules::Relaxed);
const CLASSIC: Variant = Variant::Pyramid(PyramidRules::Classic);

/// Deals the solver wins, with the positions that win them.
const WINNABLE: [(Variant, u64, u64); 10] = [
    (KLONDIKE_1, 71, 42),
    (KLONDIKE_3, 156, 45),
    (SPIDER_1, 160, 160),
    (SPIDER_2, 286, 217),
    (SPIDER_4, 45, 4436),
    (Variant::FreeCell, 128, 266),
    (Variant::FreeCell, 253, 294),
    (RELAXED, 113, 30),
    (CLASSIC, 113, 180),
    (CLASSIC, 160, 413),
];

/// Deals whose search space the solver exhausts without a win.
const UNWINNABLE: [(Variant, u64); 5] = [
    (KLONDIKE_1, 20),
    (KLONDIKE_3, 163),
    (KLONDIKE_3, 274),
    (CLASSIC, 49),
    (CLASSIC, 291),
];

fn budget(variant: Variant) -> SolveBudget {
    SolveBudget::default_for(variant)
}

#[test]
fn known_winnable_deals_are_won_with_verified_moves() {
    for (variant, seed, _) in WINNABLE {
        let Verdict::Winnable { moves } = solve(variant, seed, budget(variant)) else {
            panic!("{variant:?} seed {seed} should be winnable");
        };
        assert!(replays_to_win(variant, seed, &moves), "{variant:?} seed {seed}");
        let mut game = deal(variant, seed);
        for mv in &moves {
            assert!(!game.is_won(), "{variant:?} seed {seed}: moves continue after the win");
            game.apply(*mv).unwrap();
        }
        assert!(game.is_won());
    }
}

#[test]
fn known_unwinnable_deals_are_exhausted() {
    for (variant, seed) in UNWINNABLE {
        assert_eq!(
            solve(variant, seed, budget(variant)),
            Verdict::Unwinnable,
            "{variant:?} seed {seed}"
        );
    }
}

#[test]
#[ignore = "heavy: exhausts a Relaxed Pyramid deal in about 60,000 positions"]
fn a_relaxed_pyramid_deal_can_be_proven_unwinnable() {
    assert_eq!(solve(RELAXED, 170, budget(RELAXED)), Verdict::Unwinnable);
}

#[test]
fn the_budget_is_exact_and_a_tiny_one_gives_unknown() {
    for (variant, seed, nodes) in WINNABLE {
        let enough = solve_report(variant, seed, SolveBudget::nodes(nodes));
        assert!(matches!(enough.verdict, Verdict::Winnable { .. }), "{variant:?} seed {seed}");
        assert_eq!(enough.nodes, nodes, "{variant:?} seed {seed}");
        let short = solve_report(variant, seed, SolveBudget::nodes(nodes - 1));
        assert_eq!(short.verdict, Verdict::Unknown, "{variant:?} seed {seed}");
        assert_eq!(short.nodes, nodes - 1, "{variant:?} seed {seed}");
    }
}

#[test]
fn a_budget_of_zero_or_one_never_proves_anything_about_a_hard_deal() {
    for (variant, seed, _) in WINNABLE {
        for nodes in [0, 1] {
            let report = solve_report(variant, seed, SolveBudget::nodes(nodes));
            assert_eq!(report.verdict, Verdict::Unknown, "{variant:?} seed {seed}");
            assert!(report.nodes <= nodes.max(1));
        }
    }
}

#[test]
fn the_same_input_gives_the_same_verdict_and_moves() {
    for variant in Variant::ALL {
        for seed in [3, 4, 5] {
            let small = SolveBudget::nodes(1_500);
            assert_eq!(
                solve_report(variant, seed, small),
                solve_report(variant, seed, small),
                "{variant:?} seed {seed}"
            );
        }
    }
}

#[test]
fn a_bigger_budget_keeps_the_same_winning_moves() {
    for (variant, seed, nodes) in WINNABLE {
        let at_cost = solve(variant, seed, SolveBudget::nodes(nodes));
        let roomy = solve(variant, seed, SolveBudget::nodes(nodes * 3));
        assert_eq!(at_cost, roomy, "{variant:?} seed {seed}");
    }
}

#[test]
fn replay_rejects_lists_that_do_not_win() {
    let Verdict::Winnable { moves } = solve(KLONDIKE_1, 71, budget(KLONDIKE_1)) else {
        panic!("seed 71 should be winnable");
    };
    assert!(!replays_to_win(KLONDIKE_1, 71, &[]));
    assert!(!replays_to_win(KLONDIKE_1, 71, &moves[..moves.len() - 1]));
    assert!(!replays_to_win(KLONDIKE_1, 72, &moves));
}

#[test]
fn default_budgets_are_set_for_every_variant() {
    for variant in Variant::ALL {
        assert!(SolveBudget::default_for(variant).nodes >= 100_000, "{variant:?}");
    }
}
