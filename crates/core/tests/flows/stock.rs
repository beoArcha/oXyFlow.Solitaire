//! Stock counters through `Session`: the counters follow moves, undo, redo, restart and restore.

use crate::common::SEED;
use oxysol_core::{
    DrawCount, Move, MoveError, PyramidRules, Session, SessionRecord, StockStatus, SuitCount,
    Variant,
};

const KLONDIKE_PILE: usize = 24;
const PYRAMID_STOCK: usize = 24;
const SPIDER_ROWS: usize = 5;

fn status(session: &Session) -> Option<StockStatus> {
    session.game().stock_status()
}

fn draw(session: &mut Session, times: usize) {
    for _ in 0..times {
        assert_eq!(session.apply(Move::Stock), Ok(()));
    }
}

fn stock_status(recycles: u32, recycles_left: Option<u32>) -> Option<StockStatus> {
    Some(StockStatus {
        recycles,
        recycles_left,
        deals_left: None,
    })
}

fn restored_at(session: &Session, cursor: usize) -> Option<Session> {
    let mut record: SessionRecord = session.record();
    record.cursor = cursor;
    Session::restore(record).ok()
}

#[test]
fn klondike_recycle_survives_undo_redo_restore_and_restart() {
    let mut session = Session::new(Variant::Klondike(DrawCount::One), SEED);
    assert_eq!(status(&session), stock_status(0, None));
    draw(&mut session, KLONDIKE_PILE);
    assert_eq!(status(&session), stock_status(0, None));
    draw(&mut session, 1);
    assert_eq!(status(&session), stock_status(1, None));

    assert!(session.undo());
    assert_eq!(status(&session), stock_status(0, None));
    assert!(session.redo());
    assert_eq!(status(&session), stock_status(1, None));

    let restored = Session::restore(session.record()).expect("the record restores");
    assert_eq!(status(&restored), status(&session));
    let before_recycle = restored_at(&session, KLONDIKE_PILE).expect("a prefix restores");
    assert_eq!(status(&before_recycle), stock_status(0, None));
    assert!(before_recycle.can_redo());

    session.restart();
    assert_eq!(status(&session), stock_status(0, None));
}

#[test]
fn pyramid_relaxed_counts_passes_and_survives_restore() {
    let variant = Variant::Pyramid(PyramidRules::Relaxed);
    let mut session = Session::new(variant, 7);
    assert_eq!(status(&session), stock_status(0, Some(2)));

    draw(&mut session, PYRAMID_STOCK);
    draw(&mut session, 1);
    assert_eq!(status(&session), stock_status(1, Some(1)));
    draw(&mut session, PYRAMID_STOCK);
    draw(&mut session, 1);
    assert_eq!(status(&session), stock_status(2, Some(0)));
    draw(&mut session, PYRAMID_STOCK);
    assert_eq!(session.apply(Move::Stock), Err(MoveError::StockEmpty));
    assert_eq!(status(&session), stock_status(2, Some(0)));

    let restored = Session::restore(session.record()).expect("the record restores");
    assert_eq!(status(&restored), status(&session));
    let before_last_recycle =
        restored_at(&session, 2 * PYRAMID_STOCK + 1).expect("a prefix restores");
    assert_eq!(status(&before_last_recycle), stock_status(1, Some(1)));
}

#[test]
fn pyramid_classic_refuses_the_first_recycle() {
    let mut session = Session::new(Variant::Pyramid(PyramidRules::Classic), 7);
    draw(&mut session, PYRAMID_STOCK);
    assert_eq!(session.apply(Move::Stock), Err(MoveError::StockEmpty));
    assert_eq!(status(&session), stock_status(0, Some(0)));
}

#[test]
fn spider_deals_left_follow_undo_and_restore() {
    let mut session = Session::new(Variant::Spider(SuitCount::One), SEED);
    draw(&mut session, SPIDER_ROWS);
    let deals_left = |session: &Session| status(session).and_then(|s| s.deals_left);
    assert_eq!(deals_left(&session), Some(0));
    assert_eq!(session.apply(Move::Stock), Err(MoveError::StockEmpty));

    assert!(session.undo());
    assert_eq!(deals_left(&session), Some(1));
    let mut restored = Session::restore(session.record()).expect("the record restores");
    assert_eq!(status(&restored), status(&session));
    assert_eq!(deals_left(&restored), Some(1));
    assert!(restored.redo());
    assert_eq!(deals_left(&restored), Some(0));
}

#[test]
fn freecell_has_no_stock_status_in_a_session() {
    let session = Session::new(Variant::FreeCell, SEED);
    assert_eq!(status(&session), None);
}
