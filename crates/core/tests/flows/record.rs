//! The JSON of a `SessionRecord` is the save format (R-18, T-03). These tests pin its exact text:
//! a change breaks every saved game and must be a deliberate, versioned decision.

use oxysol_core::{DrawCount, Move, PileRef, Session, SessionRecord, Variant};

const SMALL_RECORD: &str = r#"{"variant":{"klondike":"three"},"seed":42,"moves":["stock",{"transfer":{"from":{"tableau":2},"to":{"foundation":0},"count":1}},"stock"],"cursor":2}"#;
const LARGEST_SEED: &str =
    r#"{"variant":{"klondike":"one"},"seed":18446744073709551615,"moves":[],"cursor":0}"#;

fn small_record() -> SessionRecord {
    SessionRecord {
        variant: Variant::Klondike(DrawCount::Three),
        seed: 42,
        moves: vec![
            Move::Stock,
            Move::Transfer {
                from: PileRef::Tableau(2),
                to: PileRef::Foundation(0),
                count: 1,
            },
            Move::Stock,
        ],
        cursor: 2,
    }
}

#[test]
fn a_small_record_serializes_to_the_pinned_json() {
    let json = serde_json::to_string(&small_record()).expect("a record serializes");
    assert_eq!(json, SMALL_RECORD);
}

#[test]
fn the_pinned_json_restores_the_same_game() {
    let record: SessionRecord = serde_json::from_str(SMALL_RECORD).expect("the pinned JSON parses");
    assert_eq!(record, small_record());
    let session = Session::restore(record).expect("the pinned record replays");
    assert_eq!(session.record(), small_record());
    assert_eq!(session.seed(), 42);
}

#[test]
fn the_largest_seed_survives_json() {
    let record: SessionRecord = serde_json::from_str(LARGEST_SEED).expect("the JSON parses");
    assert_eq!(record.seed, u64::MAX);
    assert_eq!(
        serde_json::to_string(&record).expect("a record serializes"),
        LARGEST_SEED
    );
    let session = Session::restore(record).expect("an empty log replays");
    assert_eq!(session.seed(), u64::MAX);
}
