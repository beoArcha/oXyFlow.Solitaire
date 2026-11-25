//! A footer button does not stay drawn as hovered after it was clicked or after the player went
//! back to the keyboard (O-93).

use oxysol_core::{DrawCount, Variant};
use ratatui::crossterm::event::{KeyCode, KeyModifiers, MouseEvent, MouseEventKind};
use ratatui::style::Modifier;

use super::footer::{each_locale, sized};
use crate::action::Action;
use crate::i18n::{Locale, tf};
use crate::input::{Input, primary_key};
use crate::screens::Effect;
use crate::test_support::Harness;

fn board(locale: Locale) -> Harness {
    let mut h = sized(locale, Vec::new(), (129, 40));
    h.app.apply(Effect::Start {
        variant: Variant::Klondike(DrawCount::One),
        seed: Some(7),
        undo_allowed: true,
    });
    h
}

fn moves_label() -> String {
    let key = primary_key(Action::Moves).unwrap_or_default();
    tf("board.action.moves", &[("key", &key)])
}

fn moves_cell(h: &mut Harness) -> (u16, u16) {
    let label = moves_label();
    let Some((column, row)) = h.find(&label) else {
        panic!("{label:?} is not drawn");
    };
    (column + 1, row)
}

fn reversed(h: &mut Harness, (column, row): (u16, u16)) -> bool {
    h.lines();
    h.terminal.backend().buffer()[(column, row)]
        .modifier
        .contains(Modifier::REVERSED)
}

fn hover(h: &mut Harness, (column, row): (u16, u16)) {
    h.send(Input::Mouse(MouseEvent {
        kind: MouseEventKind::Moved,
        column,
        row,
        modifiers: KeyModifiers::NONE,
    }));
}

#[test]
fn clicking_the_moves_button_on_and_off_leaves_no_highlight() {
    each_locale(|locale| {
        let mut h = board(locale);
        let cell = moves_cell(&mut h);
        hover(&mut h, cell);
        assert!(reversed(&mut h, cell), "{locale:?}: hover is drawn");
        h.click(cell.0, cell.1);
        let cell = moves_cell(&mut h);
        assert!(!reversed(&mut h, cell), "{locale:?}: after opening");
        h.click(cell.0, cell.1);
        let cell = moves_cell(&mut h);
        assert!(!reversed(&mut h, cell), "{locale:?}: after closing");
    });
}

#[test]
fn a_key_press_clears_the_hover_of_a_footer_button() {
    each_locale(|locale| {
        let mut h = board(locale);
        let cell = moves_cell(&mut h);
        hover(&mut h, cell);
        assert!(reversed(&mut h, cell), "{locale:?}: hover is drawn");
        h.press(KeyCode::Char('v'));
        h.press(KeyCode::Char('v'));
        let cell = moves_cell(&mut h);
        assert!(!reversed(&mut h, cell), "{locale:?}: after the key");
    });
}
