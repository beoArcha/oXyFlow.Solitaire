//! The rule options of P6-06 seen from the application: a new game takes them from the settings,
//! records them, shows them in the header, and the solver-based aids stay for the default rules.

use oxysol_core::{DrawCount, GameOptions, Session, StockPasses, Variant};
use ratatui::crossterm::event::KeyCode;

use super::*;
use crate::screens::Effect;
use crate::settings::Settings;
use crate::solver::DealCheck;
use crate::storage::memory::{HistoryRow, MemoryStorage};
use crate::storage::{GameId, SavedGame};
use crate::test_support::{Harness, TRUE_COLOR};

const KLONDIKE: Variant = Variant::Klondike(DrawCount::One);

const HARD_KLONDIKE: GameOptions = GameOptions {
    klondike_passes: StockPasses::One,
    klondike_any_empty: true,
    ..GameOptions::DEFAULT
};

fn harness(options: GameOptions, winnable_only: bool) -> Harness {
    let storage = MemoryStorage {
        settings: Some(Settings {
            game_options: options,
            winnable_only,
            ..Settings::default()
        }),
        ..MemoryStorage::default()
    };
    Harness::with(storage, TRUE_COLOR, 130, 40)
}

fn start_klondike(h: &mut Harness) {
    h.press(KeyCode::Enter);
    h.press(KeyCode::Enter);
}

fn options_of_board(h: &Harness) -> Option<GameOptions> {
    h.app.board().session().map(Session::options)
}

#[test]
fn a_new_game_deals_with_the_options_of_the_settings_and_records_them() {
    let mut h = harness(HARD_KLONDIKE, false);
    start_klondike(&mut h);
    assert_eq!(
        h.app.board().session().map(Session::variant),
        Some(KLONDIKE)
    );
    assert_eq!(options_of_board(&h), Some(HARD_KLONDIKE));
    assert_eq!(h.storage.borrow().history[0].options, HARD_KLONDIKE);
    let text = h.text();
    assert!(text.contains("1 stock pass"), "{text}");
    assert!(text.contains("Any card on empty"), "{text}");
}

#[test]
fn the_default_rules_add_nothing_to_the_header() {
    let mut h = harness(GameOptions::DEFAULT, false);
    start_klondike(&mut h);
    let text = h.text();
    assert!(!text.contains("stock pass"), "{text}");
    assert!(!text.contains("Any card on empty"), "{text}");
}

#[test]
fn changed_rules_deal_without_the_winnable_search_and_without_a_check() {
    let mut h = harness(HARD_KLONDIKE, true);
    start_klondike(&mut h);
    assert_eq!(h.app.screen(), Screen::Board);
    assert!(h.app.modal().is_none());
    assert_eq!(h.solver.borrow().pending(), 0);
    assert_eq!(h.app.board().deal(), DealCheck::Unchecked);
    let seed = h.app.board().session().map(Session::seed);
    h.app.apply(Effect::SameSeed);
    assert_eq!(h.solver.borrow().pending(), 0);
    assert_eq!(h.app.board().session().map(Session::seed), seed);
    assert_eq!(h.app.board().deal(), DealCheck::Unchecked);
}

#[test]
fn options_of_another_game_leave_klondike_with_its_winnable_search() {
    let golf_only = GameOptions {
        golf_wrap: true,
        ..GameOptions::DEFAULT
    };
    let mut h = harness(golf_only, true);
    h.solver.borrow_mut().accept_nth = Some(1);
    start_klondike(&mut h);
    assert_eq!(options_of_board(&h), Some(GameOptions::DEFAULT));
    assert_eq!(
        h.app.board().deal(),
        DealCheck::Known(crate::solver::Proof::Winnable)
    );
    assert_eq!(h.storage.borrow().history[0].options, GameOptions::DEFAULT);
}

#[test]
fn same_seed_and_a_replay_keep_the_options_the_game_was_played_with() {
    let mut h = harness(HARD_KLONDIKE, false);
    start_klondike(&mut h);
    let mut settings = h.app.settings().clone();
    settings.game_options = GameOptions::DEFAULT;
    h.app.apply(Effect::Apply(settings));
    h.app.apply(Effect::SameSeed);
    assert_eq!(options_of_board(&h), Some(HARD_KLONDIKE));
    h.app.apply(Effect::Replay {
        variant: KLONDIKE,
        seed: 7,
        options: HARD_KLONDIKE,
        undo_allowed: true,
    });
    assert_eq!(h.app.board().session().map(Session::seed), Some(7));
    assert_eq!(options_of_board(&h), Some(HARD_KLONDIKE));
    let rows = &h.storage.borrow().history;
    assert!(rows.iter().all(|row| row.options == HARD_KLONDIKE));
}

#[test]
fn a_saved_game_from_an_earlier_run_resumes_with_its_options_without_a_check() {
    let record = Session::with_options(KLONDIKE, 9, HARD_KLONDIKE).record();
    let storage = MemoryStorage {
        game: Some(SavedGame {
            id: GameId(1),
            record,
            elapsed_ms: 1_000,
            undo_allowed: true,
        }),
        history: vec![HistoryRow::open(1, KLONDIKE, 9)],
        ..MemoryStorage::new()
    };
    let mut h = Harness::with(storage, TRUE_COLOR, 130, 40);
    h.app.apply(Effect::Resume);
    assert_eq!(h.app.screen(), Screen::Board);
    assert_eq!(options_of_board(&h), Some(HARD_KLONDIKE));
    assert_eq!(h.solver.borrow().pending(), 0);
    assert_eq!(h.app.board().deal(), DealCheck::Unchecked);
}
