//! Restarts in a game without undo are counted, written to the history at once, shown in the
//! header and kept through a resume (P4-05). A game with undo is unchanged.

use oxysol_core::{DrawCount, PyramidRules, Variant};

use super::{board, start, start_no_undo};
use crate::i18n::{Locale, set_locale, t, tf};
use crate::input::Input;
use crate::screens::Effect;
use crate::settings::Settings;
use crate::solver::Proof;
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, TRUE_COLOR};

const KLONDIKE: Variant = Variant::Klondike(DrawCount::One);
const SEED: u64 = 5;

fn stored_restarts(h: &Harness) -> u32 {
    h.storage.borrow().history[0].counters.restarts
}

fn header(h: &mut Harness) -> String {
    h.lines().into_iter().next().unwrap_or_default()
}

#[test]
fn a_restart_without_undo_is_counted_stored_and_shown_in_the_header() {
    let mut h = start_no_undo(KLONDIKE, SEED);
    assert!(!header(&mut h).contains("Restarts"));
    h.app.apply(Effect::Restart);
    assert_eq!(board(&h).restarts(), 1);
    assert_eq!(stored_restarts(&h), 1);
    let first = header(&mut h);
    assert!(first.contains("No undo | Restarts: 1"), "{first}");
    h.app.apply(Effect::Restart);
    assert_eq!(board(&h).restarts(), 2);
    assert_eq!(stored_restarts(&h), 2);
    assert!(header(&mut h).contains("Restarts: 2"));
}

#[test]
fn a_restart_confirmed_in_the_dialog_is_counted_once() {
    let mut h = start_no_undo(KLONDIKE, SEED);
    h.type_char('R');
    assert_eq!(stored_restarts(&h), 0);
    h.press(ratatui::crossterm::event::KeyCode::Left);
    h.press(ratatui::crossterm::event::KeyCode::Enter);
    assert_eq!(board(&h).restarts(), 1);
    assert_eq!(stored_restarts(&h), 1);
}

#[test]
fn a_restart_with_undo_is_not_counted_and_not_shown() {
    let mut h = start(KLONDIKE, SEED);
    h.app.apply(Effect::Restart);
    h.app.apply(Effect::Restart);
    assert_eq!(board(&h).restarts(), 0);
    assert_eq!(stored_restarts(&h), 0);
    assert!(!header(&mut h).contains("Restarts"));
}

#[test]
fn a_new_game_starts_again_from_zero() {
    let mut h = start_no_undo(KLONDIKE, SEED);
    h.app.apply(Effect::Restart);
    h.app.apply(Effect::Start {
        variant: KLONDIKE,
        seed: Some(SEED + 1),
        undo_allowed: false,
    });
    assert_eq!(board(&h).restarts(), 0);
    assert!(!header(&mut h).contains("Restarts"));
    assert_eq!(h.storage.borrow().history[1].counters.restarts, 0);
}

#[test]
fn a_resumed_game_without_undo_keeps_its_restart_count() {
    let mut h = start_no_undo(KLONDIKE, SEED);
    h.app.apply(Effect::Restart);
    h.app.apply(Effect::Restart);
    h.type_char('s');
    let storage = MemoryStorage {
        game: h.storage.borrow().game.clone(),
        history: h.storage.borrow().history.clone(),
        ..MemoryStorage::default()
    };
    assert!(storage.game.is_some());
    let mut resumed = Harness::with(storage, TRUE_COLOR, 120, 40);
    resumed.app.apply(Effect::Resume);
    assert_eq!(board(&resumed).restarts(), 2);
    assert!(header(&mut resumed).contains("Restarts: 2"));
    resumed.app.apply(Effect::Restart);
    assert_eq!(board(&resumed).restarts(), 3);
    assert_eq!(stored_restarts(&resumed), 3);
}

#[test]
fn the_header_keeps_the_restart_count_at_100_columns_in_every_offered_locale() {
    let pyramid = Variant::Pyramid(PyramidRules::Classic);
    for locale in Locale::offered() {
        set_locale(locale);
        let undo = t("board.header.no_undo").into_owned();
        let restarts = tf("board.header.restarts", &[("count", "3")]);
        set_locale(Locale::EnUs);
        let storage = MemoryStorage {
            settings: Some(Settings {
                locale,
                ..Settings::default()
            }),
            ..MemoryStorage::default()
        };
        let mut h = Harness::with(storage, TRUE_COLOR, 100, 30);
        h.solver
            .borrow_mut()
            .verdicts
            .insert(u64::MAX, Proof::Unknown);
        h.app.apply(Effect::Start {
            variant: pyramid,
            seed: Some(u64::MAX),
            undo_allowed: false,
        });
        for _ in 0..3 {
            h.app.apply(Effect::Restart);
        }
        h.send(Input::Tick);
        let line = header(&mut h);
        assert!(line.contains(&undo), "{line}");
        assert!(line.contains(&restarts), "{line}");
        assert!(line.contains(&u64::MAX.to_string()), "{line}");
        assert!(line.trim_end().chars().count() <= 100, "{line}");
    }
}
