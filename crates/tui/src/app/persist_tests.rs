use super::*;
use crate::storage::memory::{HistoryRow, MemoryStorage};
use crate::storage::{GameId, Outcome};
use crate::test_support::{Harness, TRUE_COLOR};
use oxysol_core::{DrawCount, Move, Session, Variant};
use ratatui::crossterm::event::KeyCode;
use std::time::Duration;

const KLONDIKE: Variant = Variant::Klondike(DrawCount::One);

fn started() -> Harness {
    let mut h = Harness::new();
    h.press(KeyCode::Enter);
    h.press(KeyCode::Enter);
    h
}

fn saved_cursor(h: &Harness) -> Option<usize> {
    h.storage
        .borrow()
        .game
        .as_ref()
        .map(|game| game.record.cursor)
}

fn is_alert(h: &Harness) -> bool {
    matches!(h.app.modal(), Some(Modal::Alert { .. }))
}

#[test]
fn a_new_game_is_saved_at_once_and_after_every_move() {
    let mut h = started();
    assert_eq!(saved_cursor(&h), Some(0));
    h.type_char('s');
    assert_eq!(saved_cursor(&h), Some(1));
    h.type_char('u');
    assert_eq!(saved_cursor(&h), Some(0));
    let game = h.storage.borrow().game.clone();
    assert!(game.is_some_and(|g| g.id == GameId(1) && g.record.seed == 42));
}

#[test]
fn leaving_the_board_saves_the_playing_time() {
    let mut h = started();
    h.advance(75);
    h.press(KeyCode::Esc);
    assert_eq!(h.app.screen(), Screen::MainMenu);
    let elapsed = h.storage.borrow().game.as_ref().map(|g| g.elapsed_ms);
    assert_eq!(elapsed, Some(75_000));
}

#[test]
fn resume_from_the_menu_continues_the_same_game() {
    let mut h = started();
    h.type_char('s');
    h.advance(20);
    h.press(KeyCode::Esc);
    h.app.apply(Effect::Resume);
    assert_eq!(h.app.screen(), Screen::Board);
    assert!(
        h.app
            .board()
            .session()
            .is_some_and(|s| s.record().cursor == 1)
    );
    assert_eq!(h.app.board().elapsed(h.now), Duration::from_secs(20));
}

#[test]
fn quitting_mid_game_keeps_the_save_and_the_history_row_open() {
    let mut h = started();
    h.type_char('s');
    h.advance(5);
    h.app.apply(Effect::Quit);
    assert!(h.app.should_quit());
    let storage = h.storage.borrow();
    assert_eq!(storage.game.as_ref().map(|g| g.elapsed_ms), Some(5_000));
    assert_eq!(storage.history[0].finished, None);
}

#[test]
fn dealing_another_game_over_a_save_abandons_it() {
    let mut h = started();
    h.type_char('s');
    h.advance(12);
    h.app.apply(Effect::Start {
        variant: Variant::FreeCell,
        seed: Some(3),
        undo_allowed: true,
    });
    let storage = h.storage.borrow();
    assert_eq!(storage.history.len(), 2);
    assert_eq!(
        storage.history[0].finished,
        Some((Outcome::Abandoned, 12_000))
    );
    assert_eq!(storage.history[1].finished, None);
    assert!(
        storage
            .game
            .as_ref()
            .is_some_and(|g| g.id == GameId(2) && g.record.seed == 3)
    );
}

#[test]
fn an_abandoned_game_records_its_moves_with_undo_taking_a_move_back() {
    let mut h = started();
    h.type_char('s');
    h.type_char('s');
    h.type_char('s');
    h.type_char('u');
    h.app.apply(Effect::Start {
        variant: Variant::FreeCell,
        seed: Some(3),
        undo_allowed: true,
    });
    let storage = h.storage.borrow();
    assert_eq!(storage.history[0].moves, Some(2));
    assert!(storage.history[0].undo_allowed);
    assert!(storage.history[1].undo_allowed);
    assert_eq!(storage.history[1].moves, None);
}

#[test]
fn a_game_without_moves_is_abandoned_with_a_move_count_of_zero() {
    let mut h = started();
    h.app.apply(Effect::Start {
        variant: Variant::FreeCell,
        seed: Some(3),
        undo_allowed: true,
    });
    assert_eq!(h.storage.borrow().history[0].moves, Some(0));
}

#[test]
fn a_saved_game_from_an_earlier_run_is_abandoned_by_a_new_game() {
    let record = Session::new(KLONDIKE, 9).record();
    let storage = MemoryStorage {
        game: Some(SavedGame {
            id: GameId(1),
            record,
            elapsed_ms: 4_000,
            undo_allowed: true,
        }),
        history: vec![HistoryRow::open(1, KLONDIKE, 9)],
        ..MemoryStorage::new()
    };
    let mut h = Harness::with(storage, TRUE_COLOR, 120, 40);
    h.app.apply(Effect::Start {
        variant: KLONDIKE,
        seed: Some(1),
        undo_allowed: true,
    });
    let stored = h.storage.borrow();
    assert_eq!(
        stored.history[0].finished,
        Some((Outcome::Abandoned, 4_000))
    );
    assert_eq!(stored.history.len(), 2);
}

#[test]
fn a_failing_write_alerts_once_and_the_game_continues() {
    let mut h = started();
    h.storage.borrow_mut().failing = true;
    h.type_char('s');
    assert!(is_alert(&h));
    assert!(
        h.app
            .board()
            .session()
            .is_some_and(|s| s.record().cursor == 1)
    );
    h.press(KeyCode::Enter);
    assert!(h.app.modal().is_none());
    h.type_char('u');
    h.type_char('s');
    assert!(h.app.modal().is_none());
    assert!(
        h.app
            .board()
            .session()
            .is_some_and(|s| s.record().cursor == 1)
    );
}

#[test]
fn a_failing_deal_record_alerts_once_and_nothing_else_is_written() {
    let mut h = Harness::new();
    h.storage.borrow_mut().failing = true;
    h.press(KeyCode::Enter);
    h.press(KeyCode::Enter);
    assert!(is_alert(&h));
    assert!(!h.app.board().clock_running());
    h.press(KeyCode::Enter);
    h.type_char('s');
    assert!(h.app.modal().is_none());
    assert!(h.storage.borrow().game.is_none());
}

#[test]
fn an_unrestorable_save_is_removed_and_closed_in_the_history() {
    let mut record = Session::new(Variant::FreeCell, 99).record();
    record.moves.push(Move::Stock);
    record.cursor = 1;
    let storage = MemoryStorage {
        game: Some(SavedGame {
            id: GameId(1),
            record,
            elapsed_ms: 8_000,
            undo_allowed: true,
        }),
        history: vec![HistoryRow::open(1, Variant::FreeCell, 99)],
        ..MemoryStorage::new()
    };
    let mut h = Harness::with(storage, TRUE_COLOR, 120, 40);
    h.app.apply(Effect::Resume);
    assert_eq!(h.app.screen(), Screen::MainMenu);
    assert!(is_alert(&h));
    let stored = h.storage.borrow();
    assert!(stored.game.is_none());
    assert_eq!(
        stored.history[0].finished,
        Some((Outcome::Abandoned, 8_000))
    );
}
