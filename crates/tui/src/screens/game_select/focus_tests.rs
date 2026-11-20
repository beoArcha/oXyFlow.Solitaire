//! Tab and Shift+Tab on the game selection: the variants list, the undo toggle, then the winnable toggle, wrapping.

use oxysol_core::Variant;
use ratatui::crossterm::event::KeyCode;

use crate::app::Screen;
use crate::i18n::t;
use crate::screens::focus::marked_text;
use crate::screens::variant_label_key;
use crate::test_support::Harness;

const TOGGLE_ON: &str = "[ Undo: on ]";
const TOGGLE_OFF: &str = "[ Undo: off ]";
const WINNABLE_OFF: &str = "[ Winnable deals only: off ]";

fn select() -> Harness {
    let mut h = Harness::new();
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::GameSelect);
    h
}

fn focused(h: &mut Harness) -> Option<String> {
    marked_text(&h.lines(), "►").map(|text| match text.strip_prefix('[') {
        Some(_) => text,
        None => text.splitn(3, ' ').nth(2).unwrap_or_default().to_owned(),
    })
}

fn label_of(variant: Variant) -> String {
    t(variant_label_key(variant)).into_owned()
}

#[test]
fn the_list_has_the_focus_at_first() {
    let mut h = select();
    assert_eq!(focused(&mut h), Some(label_of(Variant::ALL[0])));
}

#[test]
fn tab_moves_the_focus_from_the_list_to_the_undo_toggle() {
    let mut h = select();
    h.press(KeyCode::Tab);
    assert_eq!(focused(&mut h).as_deref(), Some(TOGGLE_ON));
    assert_eq!(
        h.lines().iter().filter(|line| line.contains('►')).count(),
        1
    );
}

#[test]
fn tab_from_the_undo_toggle_reaches_the_winnable_toggle_and_then_the_list() {
    let mut h = select();
    h.press(KeyCode::Tab);
    h.press(KeyCode::Tab);
    assert_eq!(focused(&mut h).as_deref(), Some(WINNABLE_OFF));
    h.press(KeyCode::Tab);
    assert_eq!(focused(&mut h), Some(label_of(Variant::ALL[0])));
}

#[test]
fn shift_tab_from_the_list_wraps_to_the_winnable_toggle() {
    let mut h = select();
    h.press(KeyCode::BackTab);
    assert_eq!(focused(&mut h).as_deref(), Some(WINNABLE_OFF));
    h.press(KeyCode::BackTab);
    assert_eq!(focused(&mut h).as_deref(), Some(TOGGLE_ON));
}

#[test]
fn enter_on_the_toggle_switches_undo_and_starts_no_game() {
    let mut h = select();
    h.press(KeyCode::Tab);
    h.press(KeyCode::Enter);
    assert!(h.text().contains(TOGGLE_OFF));
    assert!(!h.app.settings().undo_allowed);
    assert_eq!(h.app.screen(), Screen::GameSelect);
}

#[test]
fn space_on_the_toggle_switches_undo_back() {
    let mut h = select();
    h.press(KeyCode::Tab);
    h.type_char(' ');
    assert!(h.text().contains(TOGGLE_OFF));
    h.type_char(' ');
    assert!(h.text().contains(TOGGLE_ON));
    assert!(h.app.settings().undo_allowed);
}

#[test]
fn left_and_right_switch_undo_while_the_toggle_has_the_focus() {
    let mut h = select();
    h.press(KeyCode::Tab);
    h.press(KeyCode::Right);
    assert!(h.text().contains(TOGGLE_OFF));
    h.press(KeyCode::Left);
    assert!(h.text().contains(TOGGLE_ON));
}

#[test]
fn enter_on_the_list_still_starts_the_focused_variant() {
    let mut h = select();
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::Board);
    assert_eq!(
        h.app.board().session().map(|s| s.variant()),
        Some(Variant::ALL[0])
    );
}

#[test]
fn up_from_the_toggle_moves_into_the_list_from_below() {
    let mut h = select();
    h.press(KeyCode::Tab);
    h.press(KeyCode::Up);
    assert_eq!(
        focused(&mut h),
        Some(label_of(Variant::ALL[Variant::ALL.len() - 1]))
    );
}

#[test]
fn the_toggle_is_drawn_as_a_button_and_marked_only_while_focused() {
    let mut h = select();
    assert!(!h.text().contains("► [ Undo"));
    h.press(KeyCode::Tab);
    assert!(h.text().contains("► [ Undo: on ]"));
    h.press(KeyCode::Tab);
    assert!(!h.text().contains("► [ Undo"));
    assert!(h.text().contains(TOGGLE_ON));
}
