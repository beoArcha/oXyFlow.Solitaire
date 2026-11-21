//! Golf through `Session`: plays onto the waste, the one-pass stock, undo, redo and restore.

use crate::common::SEED;
use oxysol_core::{Move, MoveError, PileRef, Session, SessionRecord, StockStatus, Variant};

const STOCK_AFTER_DEAL: usize = 16;

fn play(column: u8) -> Move {
    Move::Transfer {
        from: PileRef::Tableau(column),
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
fn a_scripted_run_plays_down_a_column_and_undoes_it() {
    let mut session = Session::new(Variant::Golf, SEED);
    assert_eq!(waste_top(&session).as_deref(), Some("Three"));
    assert_eq!(session.apply(play(0)), Ok(()));
    assert_eq!(waste_top(&session).as_deref(), Some("Two"));
    assert_eq!(session.apply(play(0)), Ok(()));
    assert_eq!(waste_top(&session).as_deref(), Some("Ace"));
    assert_eq!(pile_len(&session, PileRef::Tableau(0)), 3);

    assert_eq!(session.apply(play(0)), Err(MoveError::TargetRejects));
    assert_eq!(pile_len(&session, PileRef::Tableau(0)), 3);

    assert!(session.undo());
    assert!(session.undo());
    assert_eq!(waste_top(&session).as_deref(), Some("Three"));
    assert_eq!(pile_len(&session, PileRef::Tableau(0)), 5);
    assert!(session.redo());
    assert_eq!(waste_top(&session).as_deref(), Some("Two"));
}

#[test]
fn the_stock_is_one_pass_and_survives_undo_and_restore() {
    let mut session = Session::new(Variant::Golf, SEED);
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

    assert!(session.undo());
    assert_eq!(pile_len(&session, PileRef::Stock), 1);
    let record: SessionRecord = session.record();
    let restored = Session::restore(record).expect("the record restores");
    assert_eq!(restored.game().piles(), session.game().piles());
}

#[test]
fn the_variant_is_saved_by_name() {
    let session = Session::new(Variant::Golf, SEED);
    let json = serde_json::to_string(&session.record()).expect("the record serializes");
    assert!(json.contains(r#""variant":"golf""#), "{json}");
}
