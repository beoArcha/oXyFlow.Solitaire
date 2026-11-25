use oxysol_core::Move;

use super::score::{Candidate, Scored, percentiles, pick};

fn scored(seed: u64, difficulty: f64) -> Scored {
    Scored {
        candidate: Candidate {
            seed,
            effort: 0,
            length: 0,
            hint_win: false,
            moves: Vec::<Move>::new(),
        },
        difficulty,
    }
}

#[test]
fn equal_values_share_the_average_of_their_ranks() {
    assert_eq!(percentiles(&[5, 1, 5, 9]), vec![0.5, 0.0, 0.5, 1.0]);
}

#[test]
fn a_single_value_has_percentile_zero() {
    assert_eq!(percentiles(&[7]), vec![0.0]);
}

#[test]
fn the_three_seeds_nearest_the_quintile_median_are_picked_by_seed_on_ties() {
    let sorted: Vec<Scored> = (0..30).map(|seed| scored(seed, seed as f64)).collect();
    let picks = pick(&sorted);
    assert_eq!(picks.map(|levels| levels[0]), Some([1, 2, 3]));
    assert_eq!(picks.map(|levels| levels[4]), Some([25, 26, 27]));
}

#[test]
fn fewer_than_fifteen_seeds_pick_nothing() {
    let sorted: Vec<Scored> = (0..14).map(|seed| scored(seed, seed as f64)).collect();
    assert!(pick(&sorted).is_none());
}
