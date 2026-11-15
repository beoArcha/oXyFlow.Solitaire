use oxysol_core::{Move, PileRef, PyramidRules, Session, Variant};

use super::*;
use crate::app::Modal;
use crate::storage::memory::HistoryRow;
use crate::storage::{GameId, Outcome};

const PYRAMID: Variant = Variant::Pyramid(PyramidRules::Relaxed);

fn almost_won(last: &mut Option<Move>) -> Harness {
    let moves = hint_moves(PYRAMID, 2);
    let Some((final_move, earlier)) = moves.split_last() else {
        unreachable!("seed 2 is solved by hints")
    };
    *last = Some(*final_move);
    let mut session = Session::new(PYRAMID, 2);
    for mv in earlier {
        assert_eq!(session.apply(*mv), Ok(()));
    }
    let storage = MemoryStorage {
        game: Some(SavedGame {
            id: GameId(1),
            record: session.record(),
            elapsed_ms: 30_000,
        }),
        history: vec![HistoryRow::open(1, PYRAMID, 2)],
        ..MemoryStorage::default()
    };
    let mut h = Harness::with(storage, TRUE_COLOR, 120, 40);
    h.app.apply(Effect::Resume);
    h
}

#[test]
fn a_won_game_is_recorded_as_won_and_its_save_removed() {
    let mut last = None;
    let mut h = almost_won(&mut last);
    assert!(h.storage.borrow().game.is_some());
    h.advance(15);
    let Some(Move::Remove { first, second }) = last else {
        unreachable!("pyramid moves are removals")
    };
    match second {
        Some(second) if first == PileRef::Stock => drag(&mut h, second, first),
        Some(second) => drag(&mut h, first, second),
        None => drag(&mut h, first, first),
    }
    assert_eq!(h.app.modal(), Some(&Modal::win()));
    let storage = h.storage.borrow();
    assert!(storage.game.is_none());
    assert_eq!(
        storage.history[0].moves,
        u32::try_from(hint_moves(PYRAMID, 2).len()).ok()
    );
    let finished = storage.history[0].finished;
    assert!(
        matches!(finished, Some((Outcome::Won, ms)) if (45_000..46_000).contains(&ms)),
        "{finished:?}"
    );
}

#[test]
fn leaving_after_a_win_does_not_bring_the_save_back() {
    let mut last = None;
    let mut h = almost_won(&mut last);
    let Some(Move::Remove { first, second }) = last else {
        unreachable!("pyramid moves are removals")
    };
    match second {
        Some(second) if first == PileRef::Stock => drag(&mut h, second, first),
        Some(second) => drag(&mut h, first, second),
        None => drag(&mut h, first, first),
    }
    h.press(KeyCode::Enter);
    h.press(KeyCode::Esc);
    assert!(h.storage.borrow().game.is_none());
}
