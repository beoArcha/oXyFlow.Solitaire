//! One fixed-seed game per variant, driven through `Session`: moves, undo, redo, a move that
//! truncates the redo tail, `restart`, and the save format round trip.

use crate::common::SEED;
use oxysol_core::{Move, PileSnapshot, Session, SessionRecord, Variant};

const STEPS: usize = 10;
const UNDONE: usize = 3;
const REDONE: usize = 2;

type Positions = Vec<Vec<PileSnapshot>>;

fn next_move(session: &Session) -> Option<Move> {
    let game = session.game();
    game.hint().or_else(|| game.legal_moves().first().copied())
}

fn play(session: &mut Session, variant: Variant) -> Positions {
    let mut positions = vec![session.game().piles()];
    for step in 0..STEPS {
        let Some(mv) = next_move(session) else {
            break;
        };
        assert_eq!(session.apply(mv), Ok(()), "{variant:?} step {step}: {mv:?}");
        positions.push(session.game().piles());
    }
    positions
}

fn through_json(session: &Session) -> Option<Session> {
    let json = serde_json::to_string(&session.record()).ok()?;
    let record: SessionRecord = serde_json::from_str(&json).ok()?;
    Session::restore(record).ok()
}

fn assert_round_trip(session: &Session, variant: Variant) {
    let restored = through_json(session);
    assert!(
        restored.is_some(),
        "{variant:?}: the record is lost in JSON"
    );
    let Some(restored) = restored else {
        return;
    };
    assert_eq!(restored.record(), session.record(), "{variant:?} record");
    assert_eq!(
        restored.game().piles(),
        session.game().piles(),
        "{variant:?} piles"
    );
}

fn assert_undo_and_redo(session: &mut Session, positions: &Positions, variant: Variant) {
    let undone = positions
        .iter()
        .enumerate()
        .take(STEPS)
        .skip(STEPS - UNDONE);
    for (position, expected) in undone.rev() {
        assert!(session.undo(), "{variant:?} undo to {position}");
        assert_eq!(&session.game().piles(), expected, "{variant:?} {position}");
    }
    let cursor = STEPS - UNDONE;
    assert_eq!(session.record().cursor, cursor, "{variant:?}");
    assert_eq!(session.record().moves.len(), STEPS, "{variant:?}");
    assert_round_trip(session, variant);
    let redone = positions.iter().enumerate().skip(cursor + 1).take(REDONE);
    for (position, expected) in redone {
        assert!(session.redo(), "{variant:?} redo to {position}");
        assert_eq!(&session.game().piles(), expected, "{variant:?} {position}");
    }
}

fn assert_new_move_truncates_redo(session: &mut Session, variant: Variant) {
    let record = session.record();
    let upcoming = record.moves.get(record.cursor).copied();
    let legal = session.game().legal_moves();
    let other = legal.iter().copied().find(|mv| Some(*mv) != upcoming);
    let chosen = other.or_else(|| legal.first().copied());
    assert!(chosen.is_some(), "{variant:?}: no legal move after undo");
    let Some(mv) = chosen else {
        return;
    };
    assert_eq!(session.apply(mv), Ok(()), "{variant:?} {mv:?}");
    assert_eq!(
        session.record().moves.len(),
        record.cursor + 1,
        "{variant:?}"
    );
    assert_eq!(session.record().cursor, record.cursor + 1, "{variant:?}");
    assert!(!session.redo(), "{variant:?} redo after a new move");
}

fn assert_restart(session: &mut Session, dealt: &[PileSnapshot], variant: Variant) {
    session.restart();
    assert_eq!(session.game().piles(), dealt, "{variant:?} restart");
    let record = session.record();
    assert!(record.moves.is_empty(), "{variant:?}");
    assert_eq!(record.cursor, 0, "{variant:?}");
    assert_eq!(record.seed, SEED, "{variant:?}");
    assert!(!session.undo() && !session.redo(), "{variant:?}");
}

#[test]
fn every_variant_survives_a_full_session_flow() {
    for variant in Variant::ALL {
        let mut session = Session::new(variant, SEED);
        let positions = play(&mut session, variant);
        assert_eq!(
            positions.len(),
            STEPS + 1,
            "{variant:?} needs {STEPS} moves"
        );
        assert_round_trip(&session, variant);
        assert_undo_and_redo(&mut session, &positions, variant);
        assert_new_move_truncates_redo(&mut session, variant);
        assert_round_trip(&session, variant);
        assert_restart(&mut session, &positions[0], variant);
    }
}
