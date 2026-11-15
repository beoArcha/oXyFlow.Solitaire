use super::*;
use crate::rng::SplitMix64;

const SEEDS: u64 = 4;
const STEPS: usize = 60;
const COUNTS: [u8; 17] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 255];

fn real_piles() -> Vec<PileRef> {
    (0..4)
        .map(found)
        .chain((0..4).map(cell))
        .chain((0..8).map(tab))
        .collect()
}

fn pick<T: Copy>(rng: &mut SplitMix64, items: &[T]) -> Option<T> {
    let bound = u64::try_from(items.len()).unwrap_or(0);
    let index = usize::try_from(rng.below(bound)).unwrap_or(0);
    items.get(index).copied()
}

/// Every transfer between every pair of real piles, every count: listed as legal exactly when
/// `apply` accepts it.
fn assert_legal_matches_apply(game: &FreeCell) {
    let legal = game.legal_moves();
    let mut stock = game.clone();
    assert_eq!(stock.apply(Move::Stock), Err(MoveError::Unsupported));
    assert!(!legal.contains(&Move::Stock));
    for from in real_piles() {
        for to in real_piles() {
            for count in COUNTS {
                let mv = transfer(from, to, count);
                let mut copy = game.clone();
                let accepted = copy.apply(mv).is_ok();
                assert_eq!(legal.contains(&mv), accepted, "{mv:?}");
            }
        }
    }
}

fn play_randomly(seed: u64) {
    let mut game = FreeCell::deal(seed);
    let mut rng = SplitMix64::new(seed);
    for _ in 0..STEPS {
        assert_legal_matches_apply(&game);
        let legal = game.legal_moves();
        let Some(mv) = pick(&mut rng, &legal) else {
            break;
        };
        assert_eq!(game.apply(mv), Ok(()), "legal {mv:?} was refused");
    }
}

#[test]
fn legal_moves_are_exactly_the_moves_apply_accepts() {
    for seed in 0..SEEDS {
        play_randomly(seed);
    }
}

#[test]
fn every_empty_cell_and_every_empty_foundation_for_an_ace_is_offered() {
    let mut game = table();
    game.columns[0] = cards("AS 2D");
    let legal = game.legal_moves();
    assert!(legal.contains(&transfer(tab(0), cell(0), 1)));
    assert!(legal.contains(&transfer(tab(0), cell(3), 1)));
    assert!(!legal.contains(&transfer(tab(0), found(3), 1)));
    game.columns[1] = cards("AH");
    let legal = game.legal_moves();
    for index in 0..4 {
        assert!(legal.contains(&transfer(tab(1), found(index), 1)));
    }
}
