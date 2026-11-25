//! Quick move turned off (O-114): `a` leaves the table alone and says why, the `Do:` row, the guide
//! and the moves window drop or explain the entry, and turning it on again restores all of it.

use oxysol_core::{DrawCount, Move, PileRef, Variant};

use super::*;
use crate::action::Action;
use crate::audio::SoundCue;
use crate::i18n::{t, tf};
use crate::input::primary_key;
use crate::screens::board::cursor::Cursor;
use crate::settings::{Aids, Settings};

const KLONDIKE: Variant = Variant::Klondike(DrawCount::One);
const SEED: u64 = 5;
const WIDE: (u16, u16) = (140, 60);

fn game(quick_move: bool) -> Harness {
    let mut h = Harness::with(MemoryStorage::default(), TRUE_COLOR, WIDE.0, WIDE.1);
    h.app.apply(Effect::Start {
        variant: KLONDIKE,
        seed: Some(SEED),
        undo_allowed: true,
    });
    set_quick_move(&mut h, quick_move);
    h.lines();
    h
}

fn set_quick_move(h: &mut Harness, quick_move: bool) {
    h.app.apply(Effect::Quick(Aids {
        quick_move,
        ..Settings::default().aids()
    }));
}

/// A tableau pile with a transfer from it, and the count of that transfer.
fn source(h: &Harness) -> (PileRef, u8) {
    let found = legal(h).into_iter().find_map(|mv| match mv {
        Move::Transfer { from, count, .. } if matches!(from, PileRef::Tableau(_)) => {
            Some((from, count))
        }
        _ => None,
    });
    match found {
        Some(found) => found,
        None => unreachable!("the seed offers a transfer from a column"),
    }
}

fn point_at(h: &mut Harness, (pile, count): (PileRef, u8)) {
    h.app.board_mut().cursor = Cursor {
        pile,
        depth: usize::from(count),
    };
}

fn quick_key() -> String {
    primary_key(Action::QuickMove).unwrap_or_default()
}

fn quick_entry(h: &Harness) -> Option<crate::widgets::moves::MoveEntry> {
    board(h)
        .move_entries()
        .into_iter()
        .find(|entry| entry.action == Action::QuickMove)
}

#[test]
fn a_key_leaves_the_table_alone_and_says_so_when_quick_move_is_off() {
    let mut h = game(false);
    let at = source(&h);
    point_at(&mut h, at);
    let before = pile_len(&h, at.0);
    h.type_char('a');
    assert_eq!(moves_made(&h), 0);
    assert_eq!(pile_len(&h, at.0), before);
    assert_eq!(status(&h), Some(crate::screens::board::status::QUICK_OFF));
    assert_eq!(cues(&h).last(), Some(&SoundCue::Illegal));
    assert!(h.text().contains(&t("board.status.quick_off").into_owned()));
}

#[test]
fn turning_quick_move_on_again_restores_the_key() {
    let mut h = game(false);
    set_quick_move(&mut h, true);
    let at = source(&h);
    point_at(&mut h, at);
    h.type_char('a');
    assert_eq!(moves_made(&h), 1);
}

#[test]
fn the_do_row_and_the_guide_drop_quick_move_when_it_is_off() {
    let do_row = tf("board.action.quick", &[("key", &quick_key())]);
    let guide_row = t("board.guide.quick").into_owned();
    let double_click = t("board.guide.mouse.double_click").into_owned();

    let mut on = game(true);
    let at = source(&on);
    point_at(&mut on, at);
    assert!(on.text().contains(&do_row), "{}", on.text());
    assert!(on.text().contains(&guide_row));
    assert!(on.text().contains(&double_click));

    let mut off = game(false);
    let at = source(&off);
    point_at(&mut off, at);
    assert!(!off.text().contains(&do_row), "{}", off.text());
    assert!(!off.text().contains(&guide_row));
    assert!(!off.text().contains(&double_click));
}

#[test]
fn the_moves_window_shows_quick_move_unavailable_with_the_reason() {
    let mut h = game(false);
    let at = source(&h);
    point_at(&mut h, at);
    let Some(entry) = quick_entry(&h) else {
        unreachable!("the moves window lists quick move")
    };
    assert!(!entry.active);
    assert_eq!(entry.label, t("board.moves.label.quick"));
    assert_eq!(
        entry.reason.as_deref(),
        Some(t("board.moves.reason.quick_off").as_ref())
    );
}

#[test]
fn the_moves_window_lists_quick_move_as_usable_when_it_is_on() {
    let mut h = game(true);
    let at = source(&h);
    point_at(&mut h, at);
    let Some(entry) = quick_entry(&h) else {
        unreachable!("the moves window lists quick move")
    };
    assert!(entry.active);
    assert_eq!(entry.reason, None);
}
