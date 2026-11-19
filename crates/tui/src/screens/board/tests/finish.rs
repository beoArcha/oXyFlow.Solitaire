//! Auto-finish: a position the engine proves won is played out on the clock (or at once with the
//! setting off), interruptible by any key or mouse press, and recorded as a win with every move.

use oxysol_core::solver::{SolveBudget, Verdict, solve};
use oxysol_core::{DrawCount, Move, Session, SessionRecord, SuitCount, Variant};
use ratatui::crossterm::event::KeyCode;

use super::{board, moves_made, status};
use crate::app::Modal;
use crate::screens::Effect;
use crate::screens::board::status as board_status;
use crate::settings::Settings;
use crate::storage::memory::MemoryStorage;
use crate::storage::{Outcome, SavedGame, Storage};
use crate::test_support::{Harness, TRUE_COLOR};

const KLONDIKE: Variant = Variant::Klondike(DrawCount::One);
const KLONDIKE_SEED: u64 = 71;
const FREECELL_SEED: u64 = 128;
const SPIDER_SEED: u64 = 160;
const SPIDER: Variant = Variant::Spider(SuitCount::One);

/// The moves of a fixed deal up to its first forced win: the solver's winning line for the seed,
/// stopped at the first position where `Session::forced_finish` reports a finish.
fn to_forced_win(variant: Variant, seed: u64) -> Vec<Move> {
    let Verdict::Winnable { moves } = solve(variant, seed, SolveBudget::default_for(variant))
    else {
        unreachable!("{variant:?} seed {seed} is pinned as winnable");
    };
    let mut session = Session::new(variant, seed);
    let mut played = Vec::new();
    for mv in moves {
        if session.forced_finish().is_some() {
            break;
        }
        assert_eq!(session.apply(mv), Ok(()));
        played.push(mv);
    }
    assert!(
        session.forced_finish().is_some(),
        "{variant:?} seed {seed} reaches no forced win"
    );
    played
}

/// A board resumed at the first forced win of `variant` and `seed`, with the setting `auto_finish`
/// and a history row for the game. Returns the harness, the moves played before the finish and the
/// number of finishing moves.
fn at_forced_win(variant: Variant, seed: u64, auto_finish: bool) -> (Harness, usize, usize) {
    let moves = to_forced_win(variant, seed);
    let mut storage = MemoryStorage::new();
    let id = match storage.start_game(variant, seed, true) {
        Ok(id) => id,
        Err(error) => unreachable!("memory storage does not fail: {error}"),
    };
    let mut session = Session::new(variant, seed);
    for mv in &moves {
        assert_eq!(session.apply(*mv), Ok(()));
    }
    let record: SessionRecord = session.record();
    storage.game = Some(SavedGame {
        id,
        record,
        elapsed_ms: 0,
        undo_allowed: true,
    });
    let mut h = Harness::with(storage, TRUE_COLOR, 120, 40);
    let settings = Settings {
        auto_finish,
        ..h.app.settings().clone()
    };
    h.app.apply(Effect::Apply(settings));
    h.app.apply(Effect::Resume);
    let base = moves_made(&h);
    let finish = session.forced_finish().map_or(0, |moves| moves.len());
    (h, base, finish)
}

fn is_won(h: &Harness) -> bool {
    board(h).is_won()
}

fn win_open(h: &Harness) -> bool {
    matches!(h.app.modal(), Some(Modal::Win { .. }))
}

fn finished_at(h: &Harness) -> Vec<(Outcome, Option<u32>)> {
    let storage = h.storage.borrow();
    storage
        .history
        .iter()
        .map(|row| (row.finished.map_or(Outcome::Abandoned, |f| f.0), row.moves))
        .collect()
}

fn run_to_win(h: &mut Harness, ticks: usize) -> usize {
    let mut steps = 0;
    for _ in 0..ticks {
        let before = moves_made(h);
        h.advance(1);
        let after = moves_made(h);
        assert!(after - before <= 1, "one move per clock step");
        steps += after - before;
        if is_won(h) {
            break;
        }
    }
    steps
}

#[test]
fn a_forced_win_is_played_out_one_move_per_clock_step() {
    let (mut h, base, finish) = at_forced_win(KLONDIKE, KLONDIKE_SEED, true);
    h.advance(1);
    assert_eq!(moves_made(&h), base, "the first step waits for the pace");
    assert_eq!(status(&h), Some(board_status::FINISHING));
    assert!(h.text().contains("Game solved — finishing"));
    let played = run_to_win(&mut h, 400);
    assert_eq!(played, finish);
    assert!(is_won(&h));
    assert!(win_open(&h));
}

#[test]
fn the_played_out_win_is_recorded_with_every_move() {
    let (mut h, base, finish) = at_forced_win(KLONDIKE, KLONDIKE_SEED, true);
    run_to_win(&mut h, 400);
    let rows = finished_at(&h);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].0, Outcome::Won);
    assert_eq!(rows[0].1, u32::try_from(base + finish).ok());
    assert!(
        h.storage.borrow().game.is_none(),
        "the saved game is closed"
    );
}

#[test]
fn any_key_interrupts_the_finish_and_leaves_the_game_to_the_player() {
    let (mut h, base, _) = at_forced_win(KLONDIKE, KLONDIKE_SEED, true);
    h.advance(1);
    h.advance(1);
    let stopped = moves_made(&h);
    assert_eq!(stopped, base + 1);
    h.press(KeyCode::Left);
    h.advance(30);
    assert_eq!(moves_made(&h), stopped);
    assert!(!is_won(&h));
    assert!(!win_open(&h));
    assert_eq!(status(&h), None);
    assert!(h.text().contains("f: finish game"));
}

#[test]
fn a_mouse_press_interrupts_the_finish() {
    let (mut h, _, _) = at_forced_win(KLONDIKE, KLONDIKE_SEED, true);
    h.advance(1);
    h.advance(1);
    let stopped = moves_made(&h);
    h.click(0, 0);
    h.advance(30);
    assert_eq!(moves_made(&h), stopped);
    assert!(!is_won(&h));
}

#[test]
fn the_finish_key_starts_the_paced_finish_again_after_an_interruption() {
    let (mut h, _, _) = at_forced_win(KLONDIKE, KLONDIKE_SEED, true);
    h.advance(1);
    h.press(KeyCode::Left);
    h.advance(30);
    assert!(!is_won(&h));
    h.type_char('f');
    h.advance(1);
    let before = moves_made(&h);
    h.advance(1);
    assert_eq!(moves_made(&h), before + 1);
    run_to_win(&mut h, 400);
    assert!(is_won(&h));
}

#[test]
fn with_auto_finish_off_nothing_moves_by_itself_and_the_key_finishes_at_once() {
    let (mut h, base, finish) = at_forced_win(KLONDIKE, KLONDIKE_SEED, false);
    h.advance(30);
    assert_eq!(moves_made(&h), base);
    assert!(!is_won(&h));
    assert!(h.text().contains("f: finish game"));
    h.type_char('f');
    assert!(is_won(&h));
    assert!(win_open(&h));
    assert_eq!(moves_made(&h), base + finish);
    let rows = finished_at(&h);
    assert_eq!(rows[0].0, Outcome::Won);
}

#[test]
fn the_finish_key_without_a_forced_win_says_so_and_moves_nothing() {
    let mut h = Harness::new();
    h.app.apply(Effect::Start {
        variant: KLONDIKE,
        seed: Some(KLONDIKE_SEED),
        undo_allowed: true,
    });
    h.type_char('f');
    assert_eq!(moves_made(&h), 0);
    assert_eq!(status(&h), Some(board_status::NO_FINISH));
    assert!(h.text().contains("cannot be finished automatically"));
}

#[test]
fn a_freecell_forced_win_is_finished_on_the_clock() {
    let (mut h, _, finish) = at_forced_win(Variant::FreeCell, FREECELL_SEED, true);
    assert_eq!(run_to_win(&mut h, 400), finish);
    assert!(win_open(&h));
}

#[test]
fn a_spider_forced_win_is_finished_on_the_clock() {
    let (mut h, _, finish) = at_forced_win(SPIDER, SPIDER_SEED, true);
    assert_eq!(run_to_win(&mut h, 400), finish);
    assert!(win_open(&h));
}
