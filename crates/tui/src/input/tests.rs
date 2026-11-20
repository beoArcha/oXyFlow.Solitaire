use std::collections::HashSet;

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use super::*;
use crate::action::{Action, HelpGroup};
use crate::i18n::{Locale, set_locale};

fn press(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent::new(code, modifiers)
}

#[test]
fn every_action_has_exactly_one_binding_with_a_key() {
    for action in Action::ALL {
        let rows = KEY_TABLE.iter().filter(|b| b.action == action).count();
        assert_eq!(rows, 1, "{action:?}");
        assert!(binding_for(action).is_some_and(|b| !b.keys.is_empty()));
    }
}

#[test]
fn no_key_is_bound_twice() {
    let mut seen = HashSet::new();
    for spec in KEY_TABLE.iter().flat_map(|b| b.keys.iter()) {
        assert!(seen.insert(spec.display()), "duplicate {}", spec.display());
    }
}

#[test]
fn keys_map_to_their_actions() {
    let cases = [
        (KeyCode::Enter, KeyModifiers::NONE, Some(Action::Select)),
        (KeyCode::Char('j'), KeyModifiers::NONE, Some(Action::Down)),
        (
            KeyCode::Char('c'),
            KeyModifiers::CONTROL,
            Some(Action::ForceQuit),
        ),
        (
            KeyCode::Char('R'),
            KeyModifiers::SHIFT,
            Some(Action::Restart),
        ),
        (KeyCode::Char('?'), KeyModifiers::SHIFT, Some(Action::Help)),
        (
            KeyCode::Char('c'),
            KeyModifiers::NONE,
            Some(Action::CopySeed),
        ),
        (KeyCode::Char('C'), KeyModifiers::SHIFT, None),
        (KeyCode::Char('q'), KeyModifiers::CONTROL, None),
    ];
    for (code, modifiers, expected) in cases {
        assert_eq!(
            action_for_key(&press(code, modifiers)),
            expected,
            "{code:?}"
        );
    }
}

#[test]
fn the_zoom_keys_are_brackets_and_zero_in_the_display_group() {
    let cases = [
        (']', Action::ZoomIn, "]"),
        ('[', Action::ZoomOut, "["),
        ('0', Action::ZoomAuto, "0"),
    ];
    for (key, action, name) in cases {
        let event = press(KeyCode::Char(key), KeyModifiers::NONE);
        assert_eq!(action_for_key(&event), Some(action));
        assert_eq!(primary_key(action).as_deref(), Some(name));
        assert_eq!(
            binding_for(action).map(|binding| binding.group),
            Some(HelpGroup::Display)
        );
    }
}

#[test]
fn plus_and_minus_stay_the_run_size_keys() {
    let plus = press(KeyCode::Char('+'), KeyModifiers::SHIFT);
    let minus = press(KeyCode::Char('-'), KeyModifiers::NONE);
    assert_eq!(action_for_key(&plus), Some(Action::RunMore));
    assert_eq!(action_for_key(&minus), Some(Action::RunLess));
    for code in [KeyCode::Up, KeyCode::Down] {
        let shifted = press(code, KeyModifiers::SHIFT);
        assert_eq!(action_for_key(&shifted), None, "{code:?}");
    }
}

#[test]
fn the_font_keys_are_not_in_the_key_table() {
    for code in ['+', '=', '-', '0'] {
        let event = press(KeyCode::Char(code), KeyModifiers::CONTROL);
        assert_eq!(action_for_key(&event), None, "{code}");
    }
}

#[test]
fn the_rules_key_is_i_and_is_listed_with_the_general_help() {
    let key = press(KeyCode::Char('i'), KeyModifiers::NONE);
    assert_eq!(action_for_key(&key), Some(Action::Rules));
    assert_eq!(primary_key(Action::Rules).as_deref(), Some("i"));
    assert_eq!(
        binding_for(Action::Rules).map(|binding| binding.group),
        Some(HelpGroup::General)
    );
}

#[test]
fn the_guide_key_is_g_and_is_listed_with_the_board_moves() {
    let key = press(KeyCode::Char('g'), KeyModifiers::NONE);
    assert_eq!(action_for_key(&key), Some(Action::Guide));
    assert_eq!(primary_key(Action::Guide).as_deref(), Some("g"));
    assert_eq!(
        binding_for(Action::Guide).map(|binding| binding.group),
        Some(HelpGroup::Board)
    );
}

#[test]
fn the_moves_key_is_v_and_is_listed_with_the_board_moves() {
    let key = press(KeyCode::Char('v'), KeyModifiers::NONE);
    assert_eq!(action_for_key(&key), Some(Action::Moves));
    assert_eq!(primary_key(Action::Moves).as_deref(), Some("v"));
    assert_eq!(
        binding_for(Action::Moves).map(|binding| binding.group),
        Some(HelpGroup::Board)
    );
}

#[test]
fn the_copy_seed_key_is_c_and_is_listed_with_the_game_keys() {
    assert_eq!(primary_key(Action::CopySeed).as_deref(), Some("c"));
    assert_eq!(
        binding_for(Action::CopySeed).map(|binding| binding.group),
        Some(HelpGroup::Game)
    );
}

#[test]
fn typed_characters_are_text_and_control_keys_are_not() {
    assert!(is_text_key(&press(KeyCode::Char('?'), KeyModifiers::NONE)));
    assert!(is_text_key(&press(KeyCode::Char('I'), KeyModifiers::SHIFT)));
    assert!(!is_text_key(&press(
        KeyCode::Char('c'),
        KeyModifiers::CONTROL
    )));
    assert!(!is_text_key(&press(KeyCode::Enter, KeyModifiers::NONE)));
    assert!(!is_text_key(&press(KeyCode::F(1), KeyModifiers::NONE)));
    assert!(!is_text_key(&press(KeyCode::Backspace, KeyModifiers::NONE)));
}

#[test]
fn key_release_maps_to_nothing() {
    let mut key = press(KeyCode::Enter, KeyModifiers::NONE);
    key.kind = KeyEventKind::Release;
    assert_eq!(action_for_key(&key), None);
}

#[test]
fn tab_and_shift_tab_move_the_focus_with_or_without_the_shift_flag() {
    let tab = press(KeyCode::Tab, KeyModifiers::NONE);
    let back = press(KeyCode::BackTab, KeyModifiers::SHIFT);
    assert_eq!(action_for_key(&tab), Some(Action::FocusNext));
    assert_eq!(action_for_key(&back), Some(Action::FocusPrevious));
    let plain_back = press(KeyCode::BackTab, KeyModifiers::NONE);
    assert_eq!(action_for_key(&plain_back), Some(Action::FocusPrevious));
    assert!(!is_text_key(&tab));
    assert!(!is_text_key(&back));
}

#[test]
fn tab_keys_are_named_for_help() {
    assert_eq!(primary_key(Action::FocusNext).as_deref(), Some("Tab"));
    assert_eq!(
        primary_key(Action::FocusPrevious).as_deref(),
        Some("Shift+Tab")
    );
    assert_eq!(
        binding_for(Action::FocusPrevious).map(|binding| binding.group),
        Some(HelpGroup::Navigation)
    );
}

#[test]
fn the_primary_key_event_maps_back_to_its_action() {
    for action in Action::ALL {
        if let Some(key) = primary_key_event(action) {
            assert_eq!(action_for_key(&key), Some(action), "{action:?}");
        }
    }
}

#[test]
fn display_names_are_plain_text() {
    assert_eq!(primary_key(Action::Select).as_deref(), Some("Enter"));
    assert_eq!(primary_key(Action::ForceQuit).as_deref(), Some("Ctrl+C"));
    assert_eq!(KeySpec::ch(' ').display(), "Space");
}

#[test]
fn restart_is_shown_as_shift_r() {
    assert_eq!(primary_key(Action::Restart).as_deref(), Some("Shift+R"));
    assert_eq!(KeySpec::ctrl('z').display(), "Ctrl+Z");
}

#[test]
fn key_names_follow_the_locale() {
    set_locale(Locale::PlPl);
    assert_eq!(KeySpec::plain(KeyCode::Up).display(), "Góra");
    assert_eq!(KeySpec::plain(KeyCode::Down).display(), "Dół");
    assert_eq!(KeySpec::ch(' ').display(), "Spacja");
    assert_eq!(KeySpec::plain(KeyCode::BackTab).display(), "Shift+Tab");
    set_locale(Locale::EnUs);
}
