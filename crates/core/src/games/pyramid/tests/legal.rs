use super::*;
use crate::rng::SplitMix64;

const EXTRA_PILES: [PileRef; 3] = [
    PileRef::Tableau(0),
    PileRef::Cell(0),
    PileRef::Pyramid { row: 7, index: 0 },
];
const GAMES_PER_RULE_SET: u64 = 6;
const STEPS: usize = 60;

fn pick(rng: &mut SplitMix64, moves: &[Move]) -> Option<Move> {
    if moves.is_empty() {
        return None;
    }
    let bound = u64::try_from(moves.len()).ok()?;
    let index = usize::try_from(rng.below(bound)).ok()?;
    moves.get(index).copied()
}

/// Every move the game could be asked for: the stock move, and for every pile and every second pile
/// a removal, a lone removal and a transfer. Includes missing piles and repeated piles.
fn candidates(game: &Pyramid) -> Vec<Move> {
    let piles: Vec<PileRef> = game
        .piles()
        .iter()
        .map(|p| p.pile)
        .chain(EXTRA_PILES)
        .collect();
    let mut moves = vec![Move::Stock];
    for &first in &piles {
        moves.push(Move::Transfer {
            from: first,
            to: PileRef::Waste,
            count: 1,
        });
        moves.push(remove(first, None));
        for &second in &piles {
            moves.push(remove(first, Some(second)));
        }
    }
    moves
}

/// The brute-force contract: every listed move is accepted; a candidate is accepted exactly when
/// it is listed in one of its two orders; a refused candidate leaves the state unchanged.
fn check_state(game: &Pyramid) {
    let legal = game.legal_moves();
    for &mv in &legal {
        let mut copy = game.clone();
        assert_eq!(copy.apply(mv), Ok(()), "{mv:?} is listed but refused");
    }
    for mv in candidates(game) {
        let mut copy = game.clone();
        let accepted = copy.apply(mv);
        let listed = legal.contains(&mv) || legal.contains(&reversed(mv));
        assert_eq!(
            accepted.is_ok(),
            listed,
            "{mv:?}: apply gave {accepted:?}, listed {listed}"
        );
        if accepted.is_err() {
            assert_eq!(copy.piles(), game.piles(), "{mv:?} changed the state");
        }
    }
    if let Some(hint) = game.hint() {
        assert!(legal.contains(&hint), "hint {hint:?} is not legal");
    }
}

#[test]
fn legal_moves_agree_with_apply_along_random_games() {
    for rules in [PyramidRules::Relaxed, PyramidRules::Classic] {
        for seed in 0..GAMES_PER_RULE_SET {
            let mut game = Pyramid::deal(rules, seed);
            let mut rng = SplitMix64::new(seed);
            for _ in 0..STEPS {
                check_state(&game);
                let legal = game.legal_moves();
                let Some(mv) = pick(&mut rng, &legal) else {
                    break;
                };
                assert_eq!(game.apply(mv), Ok(()), "{mv:?}");
            }
        }
    }
}

#[test]
fn a_pair_is_listed_once_in_the_order_of_the_available_cards() {
    let rows = [
        ".",
        ". .",
        ". . .",
        ". . . .",
        ". . . . .",
        ". . . . . .",
        "6C 7D . . . . .",
    ];
    let game = table(PyramidRules::Relaxed, rows, "", "");
    let pairs: Vec<Move> = game
        .legal_moves()
        .into_iter()
        .filter(|mv| {
            matches!(
                mv,
                Move::Remove {
                    second: Some(_),
                    ..
                }
            )
        })
        .collect();
    assert_eq!(pairs, [remove(pyr(6, 0), Some(pyr(6, 1)))]);
}
