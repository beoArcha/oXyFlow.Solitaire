//! Tab and Shift+Tab in the dialogs (confirm, win, alert, pause) and in the help window, whose Tab
//! keeps switching its tabs.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{handle, help};
use crate::app::{Modal, Screen};
use crate::i18n::t;
use crate::input::action_for_key;
use crate::screens::Effect;

fn send(modal: &mut Modal, code: KeyCode, modifiers: KeyModifiers) -> Option<Effect> {
    let key = KeyEvent::new(code, modifiers);
    handle(modal, &key, action_for_key(&key))
}

fn tab(modal: &mut Modal) -> Option<Effect> {
    send(modal, KeyCode::Tab, KeyModifiers::NONE)
}

fn back_tab(modal: &mut Modal) -> Option<Effect> {
    send(modal, KeyCode::BackTab, KeyModifiers::SHIFT)
}

fn enter(modal: &mut Modal) -> Option<Effect> {
    send(modal, KeyCode::Enter, KeyModifiers::NONE)
}

fn space(modal: &mut Modal) -> Option<Effect> {
    send(modal, KeyCode::Char(' '), KeyModifiers::NONE)
}

fn confirm() -> Modal {
    Modal::confirm("board.confirm.quit", Effect::Quit)
}

fn yes_focused(modal: &Modal) -> bool {
    matches!(
        modal,
        Modal::Confirm {
            yes_focused: true,
            ..
        }
    )
}

#[test]
fn tab_moves_the_confirm_focus_from_no_to_yes_and_wraps_back() {
    let mut modal = confirm();
    assert!(!yes_focused(&modal));
    assert_eq!(tab(&mut modal), None);
    assert!(yes_focused(&modal));
    assert_eq!(tab(&mut modal), None);
    assert!(!yes_focused(&modal));
}

#[test]
fn shift_tab_moves_the_confirm_focus_back_and_wraps() {
    let mut modal = confirm();
    assert_eq!(back_tab(&mut modal), None);
    assert!(yes_focused(&modal));
    assert_eq!(back_tab(&mut modal), None);
    assert!(!yes_focused(&modal));
}

#[test]
fn enter_presses_the_focused_yes_after_tab() {
    let mut modal = confirm();
    tab(&mut modal);
    assert_eq!(enter(&mut modal), Some(Effect::Quit));
}

#[test]
fn enter_on_no_closes_the_confirm_without_effect() {
    let mut modal = confirm();
    assert_eq!(enter(&mut modal), Some(Effect::None));
}

#[test]
fn space_presses_the_focused_button_like_enter() {
    let mut modal = confirm();
    tab(&mut modal);
    assert_eq!(space(&mut modal), Some(Effect::Quit));
}

#[test]
fn tab_moves_the_win_focus_between_new_game_and_menu() {
    let mut modal = Modal::win();
    assert_eq!(tab(&mut modal), None);
    assert!(matches!(modal, Modal::Win { menu_focused: true }));
    assert_eq!(tab(&mut modal), None);
    assert!(matches!(
        modal,
        Modal::Win {
            menu_focused: false
        }
    ));
}

#[test]
fn enter_on_the_win_buttons_goes_to_their_screens() {
    let mut new_game = Modal::win();
    assert_eq!(enter(&mut new_game), Some(Effect::Goto(Screen::GameSelect)));
    let mut menu = Modal::win();
    back_tab(&mut menu);
    assert_eq!(enter(&mut menu), Some(Effect::Goto(Screen::MainMenu)));
}

#[test]
fn tab_keeps_the_only_button_of_an_alert_and_enter_closes_it() {
    let mut modal = Modal::alert("common.error.storage");
    assert_eq!(tab(&mut modal), None);
    assert!(matches!(modal, Modal::Alert { .. }));
    assert_eq!(enter(&mut modal), Some(Effect::None));
}

#[test]
fn tab_keeps_the_continue_button_of_the_pause_and_space_resumes() {
    let mut modal = Modal::Pause;
    assert_eq!(tab(&mut modal), None);
    assert_eq!(modal, Modal::Pause);
    assert_eq!(back_tab(&mut modal), None);
    assert_eq!(space(&mut modal), Some(Effect::None));
}

#[test]
fn tab_cycles_the_help_tabs_and_shift_tab_steps_back() {
    let mut modal = Modal::Help;
    assert_eq!(tab(&mut modal), None);
    assert!(matches!(modal, Modal::HowToPlay(_)));
    tab(&mut modal);
    tab(&mut modal);
    assert!(matches!(modal, Modal::HowToPlay(_)));
    assert_eq!(tab(&mut modal), None);
    assert!(matches!(modal, Modal::CommandLine(_)));
    assert_eq!(tab(&mut modal), None);
    assert!(matches!(modal, Modal::Help));
    assert_eq!(back_tab(&mut modal), None);
    assert!(matches!(modal, Modal::CommandLine(_)));
}

#[test]
fn the_help_window_lists_tab_and_shift_tab_under_navigation() {
    let rows: Vec<(String, String)> = help::sections()
        .into_iter()
        .flat_map(|section| section.rows)
        .map(|row| (row.keys, row.text))
        .collect();
    let next = t("help.action.focus_next").into_owned();
    let previous = t("help.action.focus_previous").into_owned();
    assert!(rows.contains(&("Tab".to_owned(), next)));
    assert!(rows.contains(&("Shift+Tab".to_owned(), previous)));
}
