use oxysol_core::{DrawCount, PyramidRules, Variant};
use ratatui::crossterm::event::KeyCode;

use super::records::{WonGame, personal_best};
use crate::app::{Modal, Screen};
use crate::screens::Effect;
use crate::storage::memory::{HistoryRow, MemoryStorage};
use crate::storage::{GameCounters, GameId, GameRecord, Outcome};
use crate::test_support::{Harness, TRUE_COLOR};
use crate::widgets::overlay::PersonalBest;

const KLONDIKE: Variant = Variant::Klondike(DrawCount::Three);

fn won(id: i64, variant: Variant, undo_allowed: bool, elapsed_ms: u64, moves: u32) -> GameRecord {
    GameRecord {
        id: GameId(id),
        variant,
        seed: 1,
        started_at_ms: id * 60_000,
        finished_at_ms: Some(id * 60_000 + i64::try_from(elapsed_ms).unwrap_or(0)),
        outcome: Some(Outcome::Won),
        elapsed_ms: Some(elapsed_ms),
        moves: Some(moves),
        undo_allowed,
        proof: None,
        counters: GameCounters::default(),
    }
}

fn game(id: i64, undo_allowed: bool, elapsed_ms: u64, moves: u32) -> WonGame {
    WonGame {
        id: GameId(id),
        variant: KLONDIKE,
        undo_allowed,
        elapsed_ms,
        moves,
    }
}

#[test]
fn a_first_win_is_no_personal_best() {
    let records = [won(1, KLONDIKE, true, 90_000, 80)];
    assert_eq!(
        personal_best(&records, game(1, true, 90_000, 80)),
        PersonalBest::default()
    );
}

#[test]
fn a_faster_or_shorter_win_beats_the_earlier_wins_of_its_variant() {
    let records = [
        won(1, KLONDIKE, true, 120_000, 90),
        won(2, KLONDIKE, true, 100_000, 95),
    ];
    let current = game(3, true, 110_000, 85);
    assert_eq!(
        personal_best(&records, current),
        PersonalBest {
            time: false,
            moves: true
        }
    );
    let faster = game(3, true, 99_000, 99);
    assert_eq!(
        personal_best(&records, faster),
        PersonalBest {
            time: true,
            moves: false
        }
    );
}

#[test]
fn other_variants_and_the_other_undo_mode_do_not_count() {
    let records = [
        won(1, Variant::FreeCell, true, 60_000, 40),
        won(2, KLONDIKE, false, 60_000, 40),
    ];
    assert_eq!(
        personal_best(&records, game(3, true, 90_000, 80)),
        PersonalBest::default()
    );
}

#[test]
fn the_game_itself_and_its_open_or_lost_rows_do_not_count() {
    let mut lost = won(2, KLONDIKE, true, 10_000, 10);
    lost.outcome = Some(Outcome::Abandoned);
    let mut open = won(3, KLONDIKE, true, 10_000, 10);
    open.outcome = None;
    let records = [won(1, KLONDIKE, true, 120_000, 90), lost, open];
    assert_eq!(
        personal_best(&records, game(1, true, 50_000, 50)),
        PersonalBest {
            time: false,
            moves: false
        }
    );
}

#[test]
fn the_win_dialog_opened_by_a_faster_game_marks_the_best_time() {
    let mut storage = MemoryStorage::new();
    storage
        .history
        .push(HistoryRow::finished(1, KLONDIKE, 7, Outcome::Won, 120_000));
    let mut h = Harness::with(storage, TRUE_COLOR, 120, 40);
    h.app.apply(Effect::Start {
        variant: KLONDIKE,
        seed: Some(7),
        undo_allowed: true,
    });
    h.advance(60);
    h.app.apply(Effect::Open(Modal::win()));
    assert_eq!(
        h.app.modal(),
        Some(&Modal::Win {
            focus: 0,
            best: PersonalBest {
                time: true,
                moves: false
            }
        })
    );
}

#[test]
fn same_seed_deals_the_won_game_again_from_the_win_dialog() {
    let mut h = Harness::with(MemoryStorage::new(), TRUE_COLOR, 120, 40);
    h.app.apply(Effect::Start {
        variant: Variant::Pyramid(PyramidRules::Relaxed),
        seed: Some(9),
        undo_allowed: true,
    });
    h.app.apply(Effect::Open(Modal::win()));
    h.press(KeyCode::Right);
    h.press(KeyCode::Enter);
    assert!(h.app.modal().is_none());
    assert_eq!(h.app.screen(), Screen::Board);
    let dealt = h
        .app
        .board()
        .session()
        .map(|session| (session.variant(), session.seed()));
    assert_eq!(dealt, Some((Variant::Pyramid(PyramidRules::Relaxed), 9)));
}
