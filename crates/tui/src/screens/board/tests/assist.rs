//! The assist counters (P4-06): manual moves, quick moves, failed quick moves, the longest quick
//! streak and the auto-finish, counted on scripted games, stored with the history row and kept
//! through a resume. They are information only.

use oxysol_core::{DrawCount, PileRef, Variant};

use super::finish::{KLONDIKE_SEED, at_forced_win};
use super::{board, moves_made, put_cursor, session, start, start_no_undo};
use crate::screens::Effect;
use crate::storage::AssistCounters;
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, TRUE_COLOR};

const KLONDIKE: Variant = Variant::Klondike(DrawCount::One);
const SEED: u64 = 5;
const ROUNDS: usize = 60;
const MANUAL_EVERY: usize = 7;

#[derive(Default)]
struct Observed {
    manual: u32,
    quick: u32,
    failed: u32,
    streak: u32,
    longest: u32,
}

impl Observed {
    fn counters(&self) -> AssistCounters {
        AssistCounters {
            manual: self.manual,
            quick: self.quick,
            finish: 0,
            failed: self.failed,
            longest_streak: self.longest,
        }
    }
}

fn piles(h: &Harness) -> Vec<PileRef> {
    session(h)
        .game()
        .piles()
        .iter()
        .map(|snapshot| snapshot.pile)
        .collect()
}

fn quick_on(h: &mut Harness, pile: PileRef, seen: &mut Observed) {
    let (moves, failed) = (moves_made(h), board(h).assist().failed);
    put_cursor(h, pile);
    h.type_char('a');
    if moves_made(h) > moves {
        seen.quick += 1;
        seen.streak += 1;
        seen.longest = seen.longest.max(seen.streak);
    } else if board(h).assist().failed > failed {
        seen.failed += 1;
    }
}

fn manual_stock(h: &mut Harness, seen: &mut Observed) {
    let moves = moves_made(h);
    h.type_char('s');
    if moves_made(h) > moves {
        seen.manual += 1;
        seen.streak = 0;
    }
}

fn play_script(h: &mut Harness) -> Observed {
    let mut seen = Observed::default();
    for round in 0..ROUNDS {
        if board(h).is_won() {
            break;
        }
        for pile in piles(h) {
            quick_on(h, pile, &mut seen);
        }
        if round % MANUAL_EVERY == 0 {
            manual_stock(h, &mut seen);
        }
    }
    seen
}

#[test]
fn every_counter_follows_a_scripted_game() {
    let mut h = start(KLONDIKE, SEED);
    let seen = play_script(&mut h);
    assert_eq!(board(&h).assist(), seen.counters());
    assert!(
        seen.quick > 0 && seen.failed > 0 && seen.manual > 0,
        "{:?}",
        board(&h).assist()
    );
    assert!(
        seen.longest > 1,
        "the script must make quick moves in a row"
    );
    assert_eq!(
        u32::try_from(moves_made(&h)).ok(),
        Some(seen.quick + seen.manual)
    );
}

#[test]
fn a_manual_move_ends_the_streak_but_not_the_longest() {
    let mut h = start(KLONDIKE, SEED);
    let seen = play_script(&mut h);
    let longest = board(&h).assist().longest_streak;
    manual_stock(&mut h, &mut Observed::default());
    assert_eq!(board(&h).assist().longest_streak, longest);
    assert_eq!(longest, seen.longest);
}

#[test]
fn the_counters_are_stored_with_the_history_row_as_the_game_goes() {
    let mut h = start(KLONDIKE, SEED);
    let seen = play_script(&mut h);
    let stored = h.storage.borrow().history[0].counters.assist;
    assert_eq!(stored, seen.counters());
}

#[test]
fn a_resumed_game_keeps_its_counters_and_starts_a_new_streak() {
    let mut h = start(KLONDIKE, SEED);
    let seen = play_script(&mut h);
    let storage = MemoryStorage {
        game: h.storage.borrow().game.clone(),
        history: h.storage.borrow().history.clone(),
        ..MemoryStorage::default()
    };
    assert!(storage.game.is_some());
    let mut resumed = Harness::with(storage, TRUE_COLOR, 120, 40);
    resumed.app.apply(Effect::Resume);
    assert_eq!(board(&resumed).assist(), seen.counters());
    let mut again = Observed::default();
    for pile in piles(&resumed) {
        quick_on(&mut resumed, pile, &mut again);
    }
    let expected = AssistCounters {
        quick: seen.quick + again.quick,
        failed: seen.failed + again.failed,
        longest_streak: seen.longest.max(again.longest),
        ..seen.counters()
    };
    assert_eq!(board(&resumed).assist(), expected);
}

#[test]
fn a_restart_counts_from_zero_again() {
    let mut h = start_no_undo(KLONDIKE, SEED);
    let seen = play_script(&mut h);
    assert!(seen.quick + seen.manual > 0);
    h.app.apply(Effect::Restart);
    assert_eq!(board(&h).assist(), AssistCounters::default());
    assert_eq!(
        h.storage.borrow().history[0].counters.assist,
        AssistCounters::default()
    );
}

#[test]
fn a_new_game_starts_with_zero_counters() {
    let mut h = start(KLONDIKE, SEED);
    play_script(&mut h);
    h.app.apply(Effect::Start {
        variant: KLONDIKE,
        seed: Some(SEED + 1),
        undo_allowed: true,
    });
    assert_eq!(board(&h).assist(), AssistCounters::default());
    assert_eq!(
        h.storage.borrow().history[1].counters.assist,
        AssistCounters::default()
    );
}

#[test]
fn the_auto_finish_is_counted_apart_from_manual_and_quick_moves() {
    let (mut h, _, finish) = at_forced_win(KLONDIKE, KLONDIKE_SEED, true);
    for _ in 0..400 {
        h.advance(1);
        if board(&h).is_won() {
            break;
        }
    }
    assert!(board(&h).is_won());
    let counters = h.storage.borrow().history[0].counters.assist;
    assert_eq!(counters.finish, u32::try_from(finish).unwrap_or(u32::MAX));
    assert_eq!(
        (counters.manual, counters.quick, counters.failed),
        (0, 0, 0)
    );
    assert_eq!(counters.share_percent(), None);
}

#[test]
fn the_finish_key_counts_its_moves_as_the_finish_too() {
    let (mut h, _, finish) = at_forced_win(KLONDIKE, KLONDIKE_SEED, false);
    h.type_char('f');
    let counters = h.storage.borrow().history[0].counters.assist;
    assert_eq!(counters.finish, u32::try_from(finish).unwrap_or(u32::MAX));
    assert_eq!(counters.decided(), 0);
}
