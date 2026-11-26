//! TriPeaks through `Session`: plays onto the waste, uncovering cards when both children leave,
//! the one-pass stock, undo, redo and restore.

use crate::common::SEED;
use oxysol_core::{Move, MoveError, PileRef, Session, SessionRecord, StockStatus, Variant};

const STOCK_AFTER_DEAL: usize = 23;

fn peak(row: u8, index: u8) -> PileRef {
    PileRef::Pyramid { row, index }
}

fn play(row: u8, index: u8) -> Move {
    Move::Transfer {
        from: peak(row, index),
        to: PileRef::Waste,
        count: 1,
    }
}

fn pile_len(session: &Session, pile: PileRef) -> usize {
    session
        .game()
        .piles()
        .iter()
        .find(|snapshot| snapshot.pile == pile)
        .map_or(0, |snapshot| snapshot.cards.len())
}

fn waste_top(session: &Session) -> Option<String> {
    session
        .game()
        .piles()
        .into_iter()
        .find(|snapshot| snapshot.pile == PileRef::Waste)
        .and_then(|snapshot| snapshot.cards.last().map(|c| format!("{:?}", c.card.rank)))
}

#[test]
fn legal_plays_uncover_upper_cards_and_undo_re_covers_them() {
    let mut session = Session::new(Variant::TriPeaks, SEED);
    assert!(session.game().is_covered(peak(2, 0)));
    let mv = loop {
        if let Some(play_mv) = session
            .game()
            .legal_moves()
            .into_iter()
            .find(|mv| matches!(mv, Move::Transfer { .. }))
        {
            break play_mv;
        }
        assert_eq!(session.apply(Move::Stock), Ok(()));
    };
    let Move::Transfer { from, .. } = mv else {
        unreachable!()
    };
    let before_waste = waste_top(&session);
    assert_eq!(session.apply(mv), Ok(()));
    assert_eq!(pile_len(&session, from), 0);
    assert_ne!(waste_top(&session), before_waste);

    assert!(session.undo());
    assert_eq!(pile_len(&session, from), 1);
    assert_eq!(waste_top(&session), before_waste);
    assert!(session.redo());
    assert_eq!(pile_len(&session, from), 0);
}

#[test]
fn the_stock_is_one_pass_and_survives_undo_and_restore() {
    let mut session = Session::new(Variant::TriPeaks, SEED);
    assert_eq!(pile_len(&session, PileRef::Stock), STOCK_AFTER_DEAL);
    for _ in 0..STOCK_AFTER_DEAL {
        assert_eq!(session.apply(Move::Stock), Ok(()));
    }
    assert_eq!(pile_len(&session, PileRef::Stock), 0);
    assert_eq!(pile_len(&session, PileRef::Waste), 1 + STOCK_AFTER_DEAL);
    assert_eq!(session.apply(Move::Stock), Err(MoveError::StockEmpty));
    assert_eq!(
        session.game().stock_status(),
        Some(StockStatus {
            recycles: 0,
            recycles_left: Some(0),
            deals_left: None,
        })
    );
    assert_eq!(session.apply(play(0, 0)), Err(MoveError::Blocked));

    assert!(session.undo());
    assert_eq!(pile_len(&session, PileRef::Stock), 1);
    let record: SessionRecord = session.record();
    let restored = Session::restore(record).expect("the record restores");
    assert_eq!(restored.game().piles(), session.game().piles());
}

#[test]
fn the_variant_is_saved_by_name() {
    let session = Session::new(Variant::TriPeaks, SEED);
    let json = serde_json::to_string(&session.record()).expect("the record serializes");
    assert!(json.contains(r#""variant":"tri_peaks""#), "{json}");
}
