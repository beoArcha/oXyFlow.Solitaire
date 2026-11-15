//! Random playouts over every variant: the contract checks every game must keep, stub or real.

use oxysol_core::{Move, PileRef, PileSnapshot, Session, SplitMix64, Variant, deal};

const SEEDS: u64 = 50;
const MAX_STEPS: usize = 200;
const PROBE_EVERY: usize = 5;
const PROBES_DRAWN: usize = 32;
const PROBES_KEPT: usize = 8;
const COUNTS: [u8; 6] = [0, 1, 2, 3, 13, 255];
const MISSING: [PileRef; 4] = [
    PileRef::Tableau(250),
    PileRef::Foundation(250),
    PileRef::Cell(250),
    PileRef::Pyramid {
        row: 200,
        index: 200,
    },
];

fn card_ids(piles: &[PileSnapshot]) -> Vec<u16> {
    let mut ids: Vec<u16> = piles
        .iter()
        .flat_map(|pile| pile.cards.iter().map(|c| c.card.id.0))
        .collect();
    ids.sort_unstable();
    ids
}

fn pick<T: Copy>(rng: &mut SplitMix64, items: &[T]) -> Option<T> {
    let bound = u64::try_from(items.len()).unwrap_or(0);
    let index = usize::try_from(rng.below(bound)).unwrap_or(0);
    items.get(index).copied()
}

fn probe(rng: &mut SplitMix64, pool: &[PileRef]) -> Option<Move> {
    let first = pick(rng, pool)?;
    let second = pick(rng, pool)?;
    match rng.below(4) {
        0 => Some(Move::Stock),
        1 => Some(Move::Remove {
            first,
            second: None,
        }),
        2 => Some(Move::Remove {
            first,
            second: Some(second),
        }),
        _ => Some(Move::Transfer {
            from: first,
            to: second,
            count: pick(rng, &COUNTS)?,
        }),
    }
}

fn swapped(mv: Move) -> Move {
    match mv {
        Move::Remove {
            first,
            second: Some(second),
        } => Move::Remove {
            first: second,
            second: Some(first),
        },
        other => other,
    }
}

fn is_listed(legal: &[Move], mv: Move) -> bool {
    legal.contains(&mv) || legal.contains(&swapped(mv))
}

fn illegal_probes(session: &Session, legal: &[Move], rng: &mut SplitMix64) -> Vec<Move> {
    let mut pool: Vec<PileRef> = session.game().piles().iter().map(|p| p.pile).collect();
    pool.extend(MISSING);
    (0..PROBES_DRAWN)
        .filter_map(|_| probe(rng, &pool))
        .filter(|&mv| !is_listed(legal, mv))
        .take(PROBES_KEPT)
        .collect()
}

fn assert_probes_rejected(session: &mut Session, legal: &[Move], rng: &mut SplitMix64) {
    let piles = session.game().piles();
    let record = session.record();
    for mv in illegal_probes(session, legal, rng) {
        assert!(session.apply(mv).is_err(), "accepted illegal {mv:?}");
        assert_eq!(session.game().piles(), piles, "{mv:?} changed the piles");
        assert_eq!(session.record(), record, "{mv:?} changed the history");
    }
}

fn play_out(session: &mut Session, ids: &[u16], rng: &mut SplitMix64) {
    for step in 0..MAX_STEPS {
        let legal = session.game().legal_moves();
        let Some(mv) = pick(rng, &legal) else {
            break;
        };
        if step % PROBE_EVERY == 0 {
            assert_probes_rejected(session, &legal, rng);
        }
        if let Some(hint) = session.game().hint() {
            assert!(legal.contains(&hint), "hint {hint:?} is not legal");
        }
        assert_eq!(session.apply(mv), Ok(()), "legal {mv:?} was rejected");
        assert_eq!(card_ids(&session.game().piles()), ids, "card set changed");
    }
}

fn check_variant_and_seed(variant: Variant, seed: u64) {
    let mut session = Session::new(variant, seed);
    let dealt = session.game().piles();
    let ids = card_ids(&dealt);
    let mut rng = SplitMix64::new(seed);
    play_out(&mut session, &ids, &mut rng);

    let finished = session.game().piles();
    let restored = Session::restore(session.record());
    assert_eq!(
        restored.map(|s| s.game().piles()),
        Ok(finished.clone()),
        "restore differs for {variant:?} seed {seed}"
    );

    while session.undo() {}
    assert_eq!(
        session.game().piles(),
        dealt,
        "undo differs for {variant:?} seed {seed}"
    );
    while session.redo() {}
    assert_eq!(
        session.game().piles(),
        finished,
        "redo differs for {variant:?} seed {seed}"
    );
}

#[test]
fn random_playouts_keep_the_game_contract() {
    for variant in Variant::ALL {
        for seed in 0..SEEDS {
            check_variant_and_seed(variant, seed);
        }
    }
}

#[test]
fn deals_are_reproducible_and_seed_dependent() {
    for variant in Variant::ALL {
        let first = deal(variant, 42).piles();
        assert_eq!(first, deal(variant, 42).piles(), "{variant:?}");
        assert_ne!(first, deal(variant, 43).piles(), "{variant:?}");
    }
}
