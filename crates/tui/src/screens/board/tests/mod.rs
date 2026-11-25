//! Controller tests: key sequences on fixed seeds, driven through `App` on a `TestBackend`.

mod assist;
mod aura;
mod aura_look;
mod commands;
mod compressed;
mod counters;
mod face_down;
mod finish;
mod focus;
mod footer;
mod frame;
mod guidance;
mod guide_new_game;
mod guide_window;
mod guide_winnable;
mod layout;
mod mouse;
mod moves;
mod moves_panel;
mod navigation;
mod new_game_button;
mod now_row;
mod persist;
mod pyramid;
mod quick;
mod quick_off;
mod recycle;
mod restarts;
mod run_wrap;
mod yukon_hints;
mod zoom;

use oxysol_core::{Move, PileRef, Session, Variant};
use ratatui::crossterm::event::KeyCode;

use super::BoardScreen;
use super::cursor::{Cursor, max_depth};
use crate::audio::SoundCue;
use crate::screens::Effect;
use crate::storage::memory::MemoryStorage;
use crate::storage::{GameId, SavedGame};
use crate::test_support::{Harness, TRUE_COLOR};

pub(super) fn start(variant: Variant, seed: u64) -> Harness {
    let mut h = Harness::new();
    h.app.apply(Effect::Start {
        variant,
        seed: Some(seed),
        undo_allowed: true,
    });
    h
}

pub(super) fn start_no_undo(variant: Variant, seed: u64) -> Harness {
    let mut h = Harness::new();
    h.app.apply(Effect::Start {
        variant,
        seed: Some(seed),
        undo_allowed: false,
    });
    h
}

pub(super) fn resumed(variant: Variant, seed: u64, moves: &[Move]) -> Harness {
    let mut session = Session::new(variant, seed);
    for mv in moves {
        assert_eq!(session.apply(*mv), Ok(()));
    }
    let storage = MemoryStorage {
        game: Some(SavedGame {
            id: GameId(1),
            record: session.record(),
            elapsed_ms: 0,
            undo_allowed: true,
        }),
        ..MemoryStorage::default()
    };
    let mut h = Harness::with(storage, TRUE_COLOR, 120, 40);
    h.app.apply(Effect::Resume);
    h
}

pub(super) fn hint_moves(variant: Variant, seed: u64) -> Vec<Move> {
    let mut session = Session::new(variant, seed);
    let mut moves = Vec::new();
    while let Some(mv) = session.game().hint() {
        if session.apply(mv).is_err() {
            break;
        }
        moves.push(mv);
        if moves.len() > 1_000 {
            break;
        }
    }
    moves
}

pub(super) fn board(h: &Harness) -> &BoardScreen {
    h.app.board()
}

pub(super) fn session(h: &Harness) -> &Session {
    match h.app.board().session() {
        Some(session) => session,
        None => unreachable!("every test starts a game"),
    }
}

pub(super) fn legal(h: &Harness) -> Vec<Move> {
    session(h).game().legal_moves()
}

pub(super) fn put_cursor(h: &mut Harness, pile: PileRef) {
    h.app.board_mut().cursor = Cursor { pile, depth: 1 };
}

pub(super) fn cursor(h: &Harness) -> (PileRef, usize) {
    let cursor = board(h).cursor;
    (cursor.pile, cursor.depth)
}

pub(super) fn cues(h: &Harness) -> Vec<SoundCue> {
    h.audio.borrow().cues.clone()
}

pub(super) fn status(h: &Harness) -> Option<&'static str> {
    board(h).status_key()
}

pub(super) fn moves_made(h: &Harness) -> usize {
    session(h).record().cursor
}

pub(super) fn pile_len(h: &Harness, pile: PileRef) -> usize {
    session(h)
        .game()
        .piles()
        .iter()
        .find(|snapshot| snapshot.pile == pile)
        .map_or(0, |snapshot| snapshot.cards.len())
}

pub(super) fn run_len(h: &Harness, pile: PileRef) -> usize {
    session(h)
        .game()
        .piles()
        .iter()
        .find(|snapshot| snapshot.pile == pile)
        .map_or(1, max_depth)
}

pub(super) fn first_transfer(h: &Harness) -> Option<(PileRef, PileRef, u8)> {
    legal(h).into_iter().find_map(|mv| match mv {
        Move::Transfer { from, to, count } => Some((from, to, count)),
        _ => None,
    })
}

/// Picks `from` and drops on `to` with the keyboard, the way a player does.
pub(super) fn drag(h: &mut Harness, from: PileRef, to: PileRef) {
    put_cursor(h, from);
    h.press(KeyCode::Enter);
    put_cursor(h, to);
    h.press(KeyCode::Enter);
}
