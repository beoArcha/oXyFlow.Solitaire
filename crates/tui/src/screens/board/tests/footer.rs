//! The `Do:` row as buttons: `g` and `f` are shown and clickable, a click on the board interrupts
//! a running finish and a click on the `f` button starts it, and a narrow row keeps the way to the
//! key list and the way out.

use ratatui::crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use super::finish::{KLONDIKE, KLONDIKE_SEED, at_forced_win, is_won, win_open};
use super::start;
use crate::action::Action;
use crate::app::Modal;
use crate::i18n::tf;
use crate::input::primary_key;
use crate::screens::Effect;
use crate::screens::board::guidance::{Guidance, part};
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, TRUE_COLOR};
use oxysol_core::{DrawCount, Variant};

fn click(h: &mut Harness, column: u16, row: u16) {
    h.send(crate::input::Input::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column,
        row,
        modifiers: KeyModifiers::NONE,
    }));
}

/// The drawn text of an action in the `Do:` row, for example `f: finish game`.
fn do_text(action: Action, key: &str) -> String {
    let name = primary_key(action).unwrap_or_default();
    tf(key, &[("key", &name)])
}

fn finishing(h: &Harness) -> bool {
    h.app.board().is_finishing()
}

#[test]
fn the_guide_button_is_shown_on_a_compact_board_and_its_click_opens_the_guide_window() {
    let mut h = start(KLONDIKE, KLONDIKE_SEED);
    let text = do_text(Action::Guide, "board.action.guide");
    let Some((column, row)) = h.find(&text) else {
        panic!("{text:?} is not drawn");
    };
    click(&mut h, column + 1, row);
    assert!(matches!(h.app.modal(), Some(Modal::Guide(_))));
}

#[test]
fn the_finish_button_is_shown_when_offered_and_its_click_plays_the_win() {
    let (mut h, _, _) = at_forced_win(KLONDIKE, KLONDIKE_SEED, false);
    let text = do_text(Action::Finish, "board.action.finish");
    let Some((column, row)) = h.find(&text) else {
        panic!("{text:?} is not drawn");
    };
    click(&mut h, column + 1, row);
    assert!(is_won(&h));
    assert!(win_open(&h));
}

#[test]
fn a_click_on_the_board_interrupts_a_running_finish_and_the_finish_button_starts_it_again() {
    let (mut h, _, _) = at_forced_win(KLONDIKE, KLONDIKE_SEED, true);
    h.advance(1);
    assert!(finishing(&h), "the finish runs on the clock");
    click(&mut h, 0, 0);
    assert!(!finishing(&h), "a click on the board stops it");
    let text = do_text(Action::Finish, "board.action.finish");
    let Some((column, row)) = h.find(&text) else {
        panic!("{text:?} is not drawn after the interruption");
    };
    click(&mut h, column + 1, row);
    assert!(finishing(&h), "the finish button starts it again");
}

#[test]
fn a_narrow_do_row_drops_the_least_useful_entries_and_keeps_the_way_out() {
    let guidance = Guidance {
        now: String::new(),
        possible: vec![
            part(Action::Undo, "board.action.undo", 3),
            part(Action::Guide, "board.action.guide", 8),
            part(Action::Rules, "board.action.rules", 9),
        ],
        tail: vec![
            part(Action::Help, "board.action.help", 0),
            part(Action::Back, "board.action.menu", 0),
        ],
        usable: Vec::new(),
    };
    let text = |width: usize| -> String {
        guidance
            .do_segments(width)
            .iter()
            .map(|segment| segment.text.as_str())
            .collect()
    };
    assert_eq!(
        text(200),
        "Do: u: undo | g: guide | i: how to play | ?: keys | Esc: menu"
    );
    assert_eq!(text(40), "Do: u: undo | ?: keys | Esc: menu");
    assert_eq!(text(30), "Do: ?: keys | Esc: menu");
}

fn board_at(width: u16, height: u16) -> Harness {
    let mut h = Harness::with(MemoryStorage::default(), TRUE_COLOR, width, height);
    h.app.apply(Effect::Start {
        variant: Variant::Klondike(DrawCount::One),
        seed: Some(5),
        undo_allowed: true,
    });
    h.lines();
    h
}

fn click_button(h: &mut Harness, action: Action, key: &str) {
    let text = do_text(action, key);
    let Some((column, row)) = h.find(&text) else {
        panic!("{text:?} is not drawn");
    };
    click(h, column + 1, row);
}

#[test]
fn the_pause_button_is_drawn_and_its_click_opens_pause_at_100_and_130_columns() {
    for (width, height) in [(100, 30), (130, 40)] {
        let mut h = board_at(width, height);
        click_button(&mut h, Action::Pause, "board.action.pause");
        assert!(
            matches!(h.app.modal(), Some(Modal::Pause { .. })),
            "{width} x {height}"
        );
    }
}

#[test]
fn the_moves_button_opens_the_moves_window_at_100_columns() {
    let mut h = board_at(100, 30);
    click_button(&mut h, Action::Moves, "board.action.moves");
    assert!(matches!(h.app.modal(), Some(Modal::Moves(_))));
}

#[test]
fn the_moves_button_toggles_the_moves_panel_at_130_columns() {
    let mut h = board_at(130, 40);
    assert!(h.app.aids().moves_panel);
    click_button(&mut h, Action::Moves, "board.action.moves");
    assert!(!h.app.aids().moves_panel);
    click_button(&mut h, Action::Moves, "board.action.moves");
    assert!(h.app.aids().moves_panel);
}
