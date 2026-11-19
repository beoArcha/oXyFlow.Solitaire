use oxysol_core::{DrawCount, Session, Variant};
use ratatui::crossterm::event::KeyCode;

use super::*;
use crate::input::Input;
use crate::screens::Effect;
use crate::settings::Settings;
use crate::solver::{DealCheck, Proof};
use crate::storage::memory::{HistoryRow, MemoryStorage};
use crate::storage::{GameId, Outcome, SavedGame};
use crate::test_support::{Harness, TRUE_COLOR};

const KLONDIKE: Variant = Variant::Klondike(DrawCount::One);

fn tick(h: &mut Harness) {
    h.send(Input::Tick);
}

fn seed_of_board(h: &Harness) -> Option<u64> {
    h.app.board().session().map(Session::seed)
}

fn proofs(h: &Harness) -> Vec<Option<Proof>> {
    h.storage
        .borrow()
        .history
        .iter()
        .map(|row| row.proof)
        .collect()
}

fn typed_seed_game(verdict: Option<Proof>, held: bool) -> Harness {
    let mut h = Harness::starting_at_seed(7);
    if let Some(proof) = verdict {
        h.solver.borrow_mut().verdicts.insert(7, proof);
    }
    h.solver.borrow_mut().held = held;
    h.press(KeyCode::Tab);
    h.press(KeyCode::Tab);
    h.press(KeyCode::Enter);
    assert!(h.app.settings().winnable_only);
    h.press(KeyCode::Tab);
    h.press(KeyCode::Enter);
    h
}

#[test]
fn a_typed_seed_is_dealt_as_typed_and_checked_in_the_background() {
    for (verdict, label) in [
        (Proof::Winnable, "Winnable: yes"),
        (Proof::Unwinnable, "Winnable: no"),
        (Proof::Unknown, "Winnable: unknown"),
    ] {
        let mut h = typed_seed_game(Some(verdict), false);
        assert_eq!(h.app.screen(), Screen::Board);
        assert!(h.app.modal().is_none());
        assert_eq!(seed_of_board(&h), Some(7));
        assert_eq!(h.solver.borrow().submitted.len(), 1);
        assert!(h.text().contains(label), "{label}");
        assert_eq!(proofs(&h), vec![Some(verdict)]);
    }
}

#[test]
fn the_check_of_a_typed_seed_never_blocks_the_board() {
    let mut h = typed_seed_game(Some(Proof::Winnable), true);
    assert_eq!(h.app.screen(), Screen::Board);
    assert_eq!(seed_of_board(&h), Some(7));
    assert!(h.app.modal().is_none());
    assert!(h.text().contains("Winnable: checking..."));
    assert_eq!(proofs(&h), vec![None]);
    h.solver.borrow_mut().release(1);
    tick(&mut h);
    assert!(h.text().contains("Winnable: yes"));
    assert_eq!(proofs(&h), vec![Some(Proof::Winnable)]);
}

#[test]
fn a_game_dealt_without_the_option_says_nothing_about_winnability() {
    let mut h = Harness::new();
    h.press(KeyCode::Enter);
    h.press(KeyCode::Enter);
    assert_eq!(h.solver.borrow().submitted.len(), 0);
    assert_eq!(h.app.board().deal(), DealCheck::Unchecked);
    assert!(!h.text().contains("Winnable"));
}

#[test]
fn replaying_a_game_from_the_statistics_keeps_its_seed_and_checks_it() {
    let row = HistoryRow::finished(1, Variant::FreeCell, 99, Outcome::Won, 60_000);
    let storage = MemoryStorage {
        history: vec![row],
        settings: Some(Settings {
            winnable_only: true,
            ..Settings::default()
        }),
        ..MemoryStorage::default()
    };
    let mut h = Harness::with(storage, TRUE_COLOR, 120, 40);
    h.solver.borrow_mut().verdicts.insert(99, Proof::Winnable);
    h.press(KeyCode::Down);
    h.press(KeyCode::Down);
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::Stats);
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::Board);
    assert_eq!(seed_of_board(&h), Some(99));
    assert!(h.app.modal().is_none());
    assert!(h.text().contains("Winnable: yes"));
    assert_eq!(proofs(&h)[1], Some(Proof::Winnable));
}

#[test]
fn a_resumed_game_is_checked_again() {
    let saved = SavedGame {
        id: GameId(7),
        record: Session::new(KLONDIKE, 5).record(),
        elapsed_ms: 1_000,
        undo_allowed: true,
    };
    let storage = MemoryStorage {
        game: Some(saved),
        history: vec![HistoryRow::open(7, KLONDIKE, 5)],
        ..MemoryStorage::default()
    };
    let mut h = Harness::with(storage, TRUE_COLOR, 120, 40);
    h.solver.borrow_mut().verdicts.insert(5, Proof::Winnable);
    h.app.apply(Effect::Resume);
    tick(&mut h);
    assert!(h.text().contains("Winnable: yes"));
    assert_eq!(proofs(&h), vec![Some(Proof::Winnable)]);
}
