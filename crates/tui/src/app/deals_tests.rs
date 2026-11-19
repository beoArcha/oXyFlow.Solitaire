use oxysol_core::{DrawCount, Session, Variant};
use ratatui::crossterm::event::KeyCode;

use super::*;
use crate::input::Input;
use crate::screens::Effect;
use crate::settings::Settings;
use crate::solver::{DealCheck, Proof};
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, TRUE_COLOR};

const KLONDIKE: Variant = Variant::Klondike(DrawCount::One);

fn harness_with(winnable_only: bool, width: u16, height: u16) -> Harness {
    let storage = MemoryStorage {
        settings: Some(Settings {
            winnable_only,
            ..Settings::default()
        }),
        ..MemoryStorage::default()
    };
    Harness::with(storage, TRUE_COLOR, width, height)
}

fn start_first_game(h: &mut Harness) {
    h.press(KeyCode::Enter);
    h.press(KeyCode::Enter);
}

fn tick(h: &mut Harness) {
    h.send(Input::Tick);
}

fn search_modal(h: &Harness) -> Option<SearchProgress> {
    match h.app.modal() {
        Some(Modal::Search(progress)) => Some(*progress),
        _ => None,
    }
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

#[test]
fn the_search_stops_at_the_nth_seed_and_deals_it_marked_winnable() {
    let mut h = harness_with(true, 120, 40);
    h.solver.borrow_mut().accept_nth = Some(3);
    start_first_game(&mut h);
    assert_eq!(h.app.screen(), Screen::Board);
    assert!(h.app.modal().is_none());
    assert_eq!(seed_of_board(&h), Some(44));
    assert_eq!(h.app.board().deal(), DealCheck::Known(Proof::Winnable));
    assert!(h.text().contains("Winnable: yes"));
    assert_eq!(proofs(&h), vec![Some(Proof::Winnable)]);
    assert_eq!(h.storage.borrow().history[0].seed, 44);
    assert_eq!(h.solver.borrow().pending(), 0);
}

#[test]
fn the_search_shows_progress_and_a_cancelled_search_deals_nothing() {
    let mut h = harness_with(true, 120, 40);
    h.solver.borrow_mut().held = true;
    start_first_game(&mut h);
    assert_eq!(search_modal(&h).map(|p| (p.tried, p.cap)), Some((0, 40)));
    assert!(h.text().contains("Seeds tried: 0 of 40"));
    h.solver.borrow_mut().release(5);
    tick(&mut h);
    assert_eq!(search_modal(&h).map(|p| p.tried), Some(5));
    assert!(h.text().contains("Seeds tried: 5 of 40"));
    assert!(h.text().contains("Esc cancels"));
    h.press(KeyCode::Esc);
    assert!(h.app.modal().is_none());
    assert_eq!(h.app.screen(), Screen::GameSelect);
    assert_eq!(h.solver.borrow().pending(), 0);
    h.solver.borrow_mut().release(100);
    tick(&mut h);
    assert!(h.app.modal().is_none());
    assert_eq!(h.app.screen(), Screen::GameSelect);
    assert!(h.storage.borrow().history.is_empty());
}

#[test]
fn a_click_on_the_cancel_button_cancels_too() {
    let mut h = harness_with(true, 120, 40);
    h.solver.borrow_mut().held = true;
    start_first_game(&mut h);
    let (column, row) = h.find("Cancel").unwrap_or_default();
    h.click(column + 1, row);
    assert!(h.app.modal().is_none());
    assert!(h.storage.borrow().history.is_empty());
}

#[test]
fn cancelling_a_search_started_over_a_running_game_keeps_that_game() {
    let mut h = harness_with(false, 120, 40);
    start_first_game(&mut h);
    assert_eq!(seed_of_board(&h), Some(42));
    h.solver.borrow_mut().held = true;
    let settings = Settings {
        winnable_only: true,
        ..h.app.settings().clone()
    };
    h.app.apply(Effect::Apply(settings));
    h.app.apply(Effect::Start {
        variant: KLONDIKE,
        seed: None,
        undo_allowed: true,
    });
    assert!(search_modal(&h).is_some());
    h.press(KeyCode::Esc);
    assert_eq!(h.app.screen(), Screen::Board);
    assert_eq!(seed_of_board(&h), Some(42));
    assert_eq!(h.storage.borrow().history.len(), 1);
}

#[test]
fn the_search_is_capped_and_offers_the_best_candidate_as_unknown() {
    let mut h = harness_with(true, 120, 40);
    start_first_game(&mut h);
    assert_eq!(h.solver.borrow().submitted.len(), 40);
    let progress = search_modal(&h);
    assert_eq!(progress.map(|p| (p.tried, p.capped)), Some((40, true)));
    let text = h.text();
    assert!(
        text.contains("No winnable deal was found in 40 seeds"),
        "{text}"
    );
    for button in ["Play as unknown", "Keep searching", "Cancel"] {
        assert!(text.contains(button), "{button}");
    }
    h.press(KeyCode::Enter);
    assert!(h.app.modal().is_none());
    assert_eq!(seed_of_board(&h), Some(42));
    assert_eq!(h.app.board().deal(), DealCheck::Known(Proof::Unknown));
    assert!(h.text().contains("Winnable: unknown"));
    assert_eq!(h.solver.borrow().submitted.len(), 40);
}

#[test]
fn keeping_on_searching_tries_another_batch_and_finds_a_later_seed() {
    let mut h = harness_with(true, 120, 40);
    h.solver.borrow_mut().accept_nth = Some(45);
    start_first_game(&mut h);
    assert!(search_modal(&h).is_some_and(|p| p.capped));
    h.press(KeyCode::Tab);
    h.press(KeyCode::Enter);
    assert_eq!(h.solver.borrow().submitted.len(), 80);
    assert!(h.app.modal().is_none());
    assert_eq!(seed_of_board(&h), Some(86));
    assert_eq!(h.app.board().deal(), DealCheck::Known(Proof::Winnable));
}

#[test]
fn without_an_undecided_seed_the_dialog_offers_no_unknown_game() {
    let mut h = harness_with(true, 120, 40);
    for seed in 42..82 {
        h.solver
            .borrow_mut()
            .verdicts
            .insert(seed, Proof::Unwinnable);
    }
    start_first_game(&mut h);
    assert!(search_modal(&h).is_some_and(|p| p.capped && !p.can_play_unproven));
    let text = h.text();
    assert!(!text.contains("Play as unknown"));
    assert!(text.contains("Keep searching"));
    h.press(KeyCode::Esc);
    assert!(h.app.modal().is_none());
    assert!(h.storage.borrow().history.is_empty());
}

#[test]
fn a_new_game_from_the_board_searches_too_and_keeps_the_undo_mode() {
    let mut h = harness_with(true, 120, 40);
    h.solver.borrow_mut().accept_nth = Some(1);
    h.app.apply(Effect::Start {
        variant: KLONDIKE,
        seed: None,
        undo_allowed: false,
    });
    tick(&mut h);
    assert_eq!(seed_of_board(&h), Some(42));
    assert!(!h.app.board().undo_allowed());
}

#[test]
fn the_marker_is_in_the_header_on_a_compact_screen_and_in_the_game_box_on_a_wide_one() {
    for (width, height) in [(100, 30), (130, 40), (200, 60)] {
        let mut h = harness_with(true, width, height);
        h.solver.borrow_mut().accept_nth = Some(1);
        start_first_game(&mut h);
        let text = h.text();
        assert_eq!(text.matches("Winnable: yes").count(), 1, "{width}x{height}");
    }
}

#[test]
fn the_search_dialog_and_the_marker_are_translated() {
    let polish = || {
        let storage = MemoryStorage {
            settings: Some(Settings {
                locale: crate::i18n::Locale::PlPl,
                winnable_only: true,
                ..Settings::default()
            }),
            ..MemoryStorage::default()
        };
        Harness::with(storage, TRUE_COLOR, 120, 40)
    };
    let mut capped = polish();
    start_first_game(&mut capped);
    let text = capped.text();
    assert!(
        text.contains("Nie znaleziono rozdania do wygrania w 40 ziarnach"),
        "{text}"
    );
    assert!(text.contains("Zagraj jako nieznane"), "{text}");
    let mut found = polish();
    found.solver.borrow_mut().accept_nth = Some(1);
    start_first_game(&mut found);
    assert!(found.text().contains("Do wygrania: tak"));
}
