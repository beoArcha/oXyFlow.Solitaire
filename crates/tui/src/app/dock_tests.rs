use super::*;
use crate::settings::DockSide;
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, TRUE_COLOR};
use ratatui::crossterm::event::{KeyCode, MouseEvent, MouseEventKind};

fn harness(width: u16, height: u16) -> Harness {
    Harness::with(MemoryStorage::new(), TRUE_COLOR, width, height)
}

fn start_first_game(h: &mut Harness) {
    h.press(KeyCode::Enter);
    h.press(KeyCode::Enter);
    h.lines();
}

fn pick_up_a_card(h: &mut Harness) -> bool {
    for _ in 0..4 {
        for key in [
            KeyCode::Down,
            KeyCode::Right,
            KeyCode::Right,
            KeyCode::Down,
            KeyCode::Right,
        ] {
            h.press(key);
            h.press(KeyCode::Enter);
            if h.app.board().carrying() {
                return true;
            }
        }
    }
    false
}

fn column_of(h: &mut Harness, needle: &str) -> Option<u16> {
    h.find(needle).map(|(column, _)| column)
}

#[test]
fn the_help_window_docks_at_the_right_at_130_by_40() {
    let mut h = harness(130, 40);
    start_first_game(&mut h);
    h.type_char('?');
    h.type_char('d');
    let column = column_of(&mut h, "Dock: right");
    assert!(column.is_some_and(|column| column >= 100), "{column:?}");
    assert!(h.app.board().clock_running());
}

#[test]
fn the_help_window_docks_at_the_left_at_200_by_60() {
    let mut h = harness(200, 60);
    start_first_game(&mut h);
    h.type_char('?');
    h.type_char('d');
    h.type_char('d');
    let column = column_of(&mut h, "Dock: left");
    assert!(column.is_some_and(|column| column < 60), "{column:?}");
    assert_eq!(h.app.settings().help_dock, DockSide::Left);
}

#[test]
fn a_narrow_screen_keeps_the_window_centred_with_a_note() {
    let mut h = harness(100, 30);
    start_first_game(&mut h);
    h.type_char('?');
    h.type_char('d');
    assert!(matches!(h.app.modal(), Some(Modal::Help(_))));
    let text = h.text();
    assert!(text.contains("Docking needs 130 columns"), "{text}");
    assert!(column_of(&mut h, "Dock: right").is_some());
}

#[test]
fn the_chosen_side_is_remembered_in_the_settings() {
    let mut storage = MemoryStorage::new();
    storage.settings = Some(Settings {
        help_dock: DockSide::Left,
        ..Settings::default()
    });
    let mut h = Harness::with(storage, TRUE_COLOR, 130, 40);
    start_first_game(&mut h);
    h.type_char('?');
    assert!(column_of(&mut h, "Dock: left").is_some_and(|column| column < 30));
    let saved = h.storage.borrow().settings.as_ref().map(|s| s.help_dock);
    assert_eq!(saved, Some(DockSide::Left));
}

#[test]
fn the_board_keeps_the_keys_while_the_docked_window_is_open() {
    let mut h = harness(130, 40);
    start_first_game(&mut h);
    h.type_char('?');
    h.type_char('d');
    assert!(h.app.modal().is_some());
    assert!(h.app.board().clock_running());
    h.press(KeyCode::F(6));
    assert!(!h.app.board().clock_running());
    h.press(KeyCode::F(6));
    assert!(h.app.board().clock_running());
    assert!(h.app.modal().is_some());
}

#[test]
fn f6_focuses_the_docked_window_and_tab_then_steps_its_tabs() {
    let mut h = harness(130, 40);
    start_first_game(&mut h);
    h.type_char('?');
    h.type_char('d');
    h.press(KeyCode::F(6));
    h.press(KeyCode::Tab);
    assert!(matches!(h.app.modal(), Some(Modal::HowToPlay(_))));
    h.press(KeyCode::F(6));
    h.press(KeyCode::Tab);
    assert!(matches!(h.app.modal(), Some(Modal::HowToPlay(_))));
}

#[test]
fn question_mark_closes_a_docked_window_the_board_has_the_keys_of() {
    let mut h = harness(130, 40);
    start_first_game(&mut h);
    h.type_char('?');
    h.type_char('d');
    h.type_char('?');
    assert!(h.app.modal().is_none());
}

#[test]
fn a_click_on_the_placement_button_cycles_the_side_and_keeps_the_window_open() {
    let mut h = harness(130, 40);
    start_first_game(&mut h);
    h.type_char('?');
    let key = crate::input::primary_key(crate::action::Action::Dock).unwrap_or_default();
    let label = crate::i18n::tf(
        "help.dock.button",
        &[
            ("side", &crate::i18n::t("help.dock.side_centre")),
            ("key", &key),
        ],
    );
    let (column, row) = h.find(&label).unwrap_or((0, 0));
    h.click(column + 2, row);
    assert!(matches!(h.app.modal(), Some(Modal::Help(_))));
    assert_eq!(h.app.settings().help_dock, DockSide::Right);
}

#[test]
fn a_click_inside_the_docked_window_gives_it_the_focus() {
    let mut h = harness(130, 40);
    start_first_game(&mut h);
    h.type_char('?');
    h.type_char('d');
    h.click(115, 20);
    assert!(!h.app.board().clock_running());
    h.press(KeyCode::Esc);
    assert!(h.app.modal().is_none());
}

#[test]
fn the_wheel_outside_the_docked_window_does_not_reach_it() {
    let mut h = harness(130, 40);
    start_first_game(&mut h);
    h.type_char('?');
    h.type_char('d');
    h.send(Input::Mouse(MouseEvent {
        kind: MouseEventKind::ScrollDown,
        column: 10,
        row: 10,
        modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
    }));
    assert!(matches!(h.app.modal(), Some(Modal::Help(_))));
    assert!(h.app.board().clock_running());
}

#[test]
fn esc_closes_a_docked_help_window_before_the_board_when_nothing_is_picked() {
    let mut h = harness(130, 40);
    start_first_game(&mut h);
    h.type_char('?');
    h.type_char('d');
    h.press(KeyCode::Esc);
    assert!(h.app.modal().is_none());
    assert_eq!(h.app.screen(), Screen::Board);
}

#[test]
fn esc_puts_a_picked_card_back_and_keeps_the_docked_help_window_open() {
    let mut h = harness(130, 40);
    start_first_game(&mut h);
    h.type_char('?');
    h.type_char('d');
    assert!(pick_up_a_card(&mut h));
    h.press(KeyCode::Esc);
    assert!(!h.app.board().carrying());
    assert!(matches!(h.app.modal(), Some(Modal::Help(_))));
    assert_eq!(h.app.screen(), Screen::Board);
}

#[test]
fn esc_without_a_docked_help_window_still_leaves_the_board_for_the_menu() {
    let mut h = harness(130, 40);
    start_first_game(&mut h);
    h.press(KeyCode::Esc);
    assert!(h.app.modal().is_none());
    assert_eq!(h.app.screen(), Screen::MainMenu);
}
