use std::time::Instant;

use super::help::Row;
use super::*;
use crate::action::HelpGroup;
use crate::i18n::t;
use crate::input::{KEY_TABLE, KeySpec};
use ratatui::crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEventKind};

fn key() -> KeyEvent {
    KeyEvent::new(KeyCode::Null, KeyModifiers::NONE)
}

fn screen() -> Rect {
    Rect::new(0, 0, 100, 30)
}

fn click(column: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

fn click_button(modal: &mut Modal, index: usize) -> Option<Effect> {
    let board = BoardScreen::new();
    let now = Instant::now();
    let rect = geometry(screen(), modal, &board, now).buttons[index];
    handle_mouse(modal, screen(), &board, now, &click(rect.x, rect.y))
}

#[test]
fn simple_modals_close_on_select_and_back() {
    for action in [Action::Select, Action::Back] {
        let mut modal = Modal::Help;
        assert_eq!(handle(&mut modal, &key(), Some(action)), Some(Effect::None));
    }
    let mut modal = Modal::Help;
    assert_eq!(handle(&mut modal, &key(), Some(Action::Hint)), None);
    assert_eq!(handle(&mut modal, &key(), None), None);
}

#[test]
fn pause_closes_on_its_own_key_only() {
    let mut modal = Modal::Pause;
    assert_eq!(handle(&mut modal, &key(), Some(Action::Hint)), None);
    assert_eq!(
        handle(&mut modal, &key(), Some(Action::Pause)),
        Some(Effect::None)
    );
}

#[test]
fn confirm_defaults_to_no_and_runs_the_effect_only_on_yes() {
    let mut modal = Modal::confirm("common.dialog.win_body", Effect::Quit);
    assert_eq!(
        handle(&mut modal, &key(), Some(Action::Select)),
        Some(Effect::None)
    );

    let mut modal = Modal::confirm("common.dialog.win_body", Effect::Quit);
    assert_eq!(handle(&mut modal, &key(), Some(Action::Left)), None);
    assert_eq!(
        handle(&mut modal, &key(), Some(Action::Select)),
        Some(Effect::Quit)
    );
}

#[test]
fn confirm_back_cancels() {
    let mut modal = Modal::confirm("common.dialog.win_body", Effect::Quit);
    assert_eq!(
        handle(&mut modal, &key(), Some(Action::Back)),
        Some(Effect::None)
    );
}

#[test]
fn win_buttons_move_focus_and_lead_to_their_screens() {
    let mut modal = Modal::win();
    assert_eq!(handle(&mut modal, &key(), Some(Action::Right)), None);
    assert_eq!(
        handle(&mut modal, &key(), Some(Action::Select)),
        Some(Effect::SameSeed)
    );

    let mut modal = Modal::win();
    assert_eq!(handle(&mut modal, &key(), Some(Action::Left)), None);
    assert_eq!(
        handle(&mut modal, &key(), Some(Action::Select)),
        Some(Effect::Goto(Screen::MainMenu))
    );

    let mut modal = Modal::win();
    assert_eq!(
        handle(&mut modal, &key(), Some(Action::Select)),
        Some(Effect::Goto(Screen::GameSelect))
    );
    let mut modal = Modal::win();
    assert_eq!(
        handle(&mut modal, &key(), Some(Action::Back)),
        Some(Effect::None)
    );
}

#[test]
fn clicks_press_the_button_under_them() {
    let mut confirm = Modal::confirm("common.dialog.win_body", Effect::Quit);
    assert_eq!(click_button(&mut confirm, 0), Some(Effect::Quit));
    let mut confirm = Modal::confirm("common.dialog.win_body", Effect::Quit);
    assert_eq!(click_button(&mut confirm, 1), Some(Effect::None));

    let mut win = Modal::win();
    assert_eq!(
        click_button(&mut win, 0),
        Some(Effect::Goto(Screen::GameSelect))
    );
    let mut win = Modal::win();
    assert_eq!(click_button(&mut win, 1), Some(Effect::SameSeed));
    let mut win = Modal::win();
    assert_eq!(
        click_button(&mut win, 2),
        Some(Effect::Goto(Screen::MainMenu))
    );

    let mut pause = Modal::Pause;
    assert_eq!(click_button(&mut pause, 0), Some(Effect::None));
    let mut help = Modal::Help;
    assert_eq!(click_button(&mut help, 0), Some(Effect::None));
}

#[test]
fn clicks_outside_the_buttons_keep_the_modal_open() {
    let board = BoardScreen::new();
    let mut modal = Modal::confirm("common.dialog.win_body", Effect::Quit);
    let now = Instant::now();
    assert_eq!(
        handle_mouse(&mut modal, screen(), &board, now, &click(0, 0)),
        None
    );
    let moved = MouseEvent {
        kind: MouseEventKind::Moved,
        column: 50,
        row: 15,
        modifiers: KeyModifiers::NONE,
    };
    assert_eq!(
        handle_mouse(&mut modal, screen(), &board, now, &moved),
        None
    );
}

#[test]
fn help_sections_list_every_binding_of_the_key_table_once() {
    let rows: Vec<Row> = help::sections()
        .into_iter()
        .flat_map(|section| section.rows)
        .collect();
    let gestures = 5;
    assert_eq!(rows.len(), KEY_TABLE.len() + gestures);
    for binding in KEY_TABLE.iter() {
        let keys = binding
            .keys
            .iter()
            .map(KeySpec::display)
            .collect::<Vec<_>>()
            .join(" / ");
        let label = t(binding.action.label_key()).into_owned();
        let matching = rows
            .iter()
            .filter(|row| row.keys == keys && row.text == label)
            .count();
        assert_eq!(matching, 1, "{keys} {label}");
    }
}

#[test]
fn help_topics_follow_the_help_groups_and_end_with_the_mouse() {
    let titles: Vec<String> = help::sections()
        .into_iter()
        .map(|section| section.title)
        .collect();
    let mut expected: Vec<String> = HelpGroup::ALL
        .iter()
        .map(|group| t(group.title_key()).into_owned())
        .collect();
    expected.push(t("help.group.mouse").into_owned());
    assert_eq!(titles, expected);
}

#[test]
fn help_switches_to_rules_on_tab_or_right_and_round_the_three_tabs_on_tab() {
    let mut modal = Modal::Help;
    let tab_key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
    assert_eq!(handle(&mut modal, &tab_key, None), None);
    let rules = HowToPlay::new(Rules::Klondike);
    assert_eq!(modal, Modal::HowToPlay(rules.focused(Focus::Tabs)));

    for _ in 0..3 {
        assert_eq!(handle(&mut modal, &tab_key, None), None);
    }
    assert_eq!(modal, Modal::CommandLine(CommandLine::new()));

    assert_eq!(handle(&mut modal, &tab_key, None), None);
    assert_eq!(modal, Modal::Help);

    assert_eq!(handle(&mut modal, &key(), Some(Action::Right)), None);
    assert_eq!(modal, Modal::HowToPlay(rules.focused(Focus::Tabs)));
}
