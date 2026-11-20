use ratatui::crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use super::*;

fn press(column: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

fn located() -> LanguagePicker {
    let mut picker = LanguagePicker::new();
    picker.spots = Spots {
        body: Rect::new(1, 1, 10, 2),
        left: Rect::new(1, 1, 1, 2),
        right: Rect::new(8, 1, 1, 1),
        down: Rect::new(10, 1, 1, 2),
        list: Rect::new(1, 3, 20, 6),
        entries: vec![Rect::new(2, 4, 18, 2), Rect::new(2, 6, 18, 2)],
    };
    picker
}

#[test]
fn left_and_right_step_through_the_languages_and_wrap() {
    let mut picker = LanguagePicker::new();
    assert_eq!(
        picker.on_action(Action::Right, Locale::EnUs),
        Some(Locale::PlPl)
    );
    assert_eq!(
        picker.on_action(Action::Left, Locale::PlPl),
        Some(Locale::EnUs)
    );
    assert_eq!(
        picker.on_action(Action::Left, Locale::EnUs),
        Locale::offered().last().copied()
    );
    let last = Locale::offered().last().copied().unwrap_or(Locale::EnUs);
    assert_eq!(picker.on_action(Action::Right, last), Some(Locale::EnUs));
}

#[test]
fn down_and_enter_open_the_list_on_the_current_language() {
    for action in [Action::Down, Action::Select] {
        let mut picker = LanguagePicker::new();
        assert_eq!(picker.on_action(action, Locale::PlPl), None);
        assert!(picker.is_open());
        assert_eq!(
            picker.on_action(Action::Select, Locale::PlPl),
            Some(Locale::PlPl)
        );
        assert!(!picker.is_open());
    }
}

#[test]
fn other_keys_do_nothing_while_closed_and_are_swallowed_while_open() {
    let mut picker = LanguagePicker::new();
    assert_eq!(picker.on_action(Action::Up, Locale::EnUs), None);
    assert!(!picker.is_open());
    picker.on_action(Action::Select, Locale::EnUs);
    assert_eq!(picker.on_action(Action::Left, Locale::EnUs), None);
    assert!(picker.is_open());
}

#[test]
fn back_closes_without_choosing() {
    let mut picker = LanguagePicker::new();
    picker.on_action(Action::Select, Locale::EnUs);
    picker.on_action(Action::Down, Locale::EnUs);
    assert_eq!(picker.on_action(Action::Back, Locale::EnUs), None);
    assert!(!picker.is_open());
}

#[test]
fn clicks_on_the_arrows_choose_and_on_the_down_arrow_open() {
    let mut picker = located();
    assert_eq!(
        picker.on_mouse(&press(1, 1), Locale::PlPl),
        Pointed::Choose(Locale::EnUs)
    );
    assert_eq!(
        picker.on_mouse(&press(8, 1), Locale::EnUs),
        Pointed::Choose(Locale::PlPl)
    );
    assert_eq!(picker.on_mouse(&press(10, 2), Locale::EnUs), Pointed::Focus);
    assert!(picker.is_open());
}

#[test]
fn a_click_on_the_body_focuses_and_elsewhere_is_ignored() {
    let mut picker = located();
    assert_eq!(picker.on_mouse(&press(5, 2), Locale::EnUs), Pointed::Focus);
    assert!(!picker.is_open());
    assert_eq!(
        picker.on_mouse(&press(50, 20), Locale::EnUs),
        Pointed::Ignored
    );
}

#[test]
fn movement_over_a_closed_selector_is_ignored() {
    let mut picker = located();
    let moved = MouseEvent {
        kind: MouseEventKind::Moved,
        column: 5,
        row: 1,
        modifiers: KeyModifiers::NONE,
    };
    assert_eq!(picker.on_mouse(&moved, Locale::EnUs), Pointed::Ignored);
}

#[test]
fn an_open_list_chooses_on_a_row_keeps_open_on_its_border_and_closes_outside() {
    let mut picker = located();
    picker.on_action(Action::Select, Locale::EnUs);
    assert_eq!(
        picker.on_mouse(&press(1, 3), Locale::EnUs),
        Pointed::Handled
    );
    assert!(picker.is_open());
    assert_eq!(
        picker.on_mouse(&press(50, 20), Locale::EnUs),
        Pointed::Handled
    );
    assert!(!picker.is_open());
    picker.on_action(Action::Select, Locale::EnUs);
    assert_eq!(
        picker.on_mouse(&press(5, 7), Locale::EnUs),
        Pointed::Choose(Locale::PlPl)
    );
    assert!(!picker.is_open());
}

#[test]
fn the_selector_steps_and_lists_only_offered_locales() {
    let offered = Locale::offered();
    let mut picker = LanguagePicker::new();
    let mut current = Locale::EnUs;
    for _ in 0..Locale::ALL.len() {
        let Some(next) = picker.on_action(Action::Right, current) else {
            unreachable!("Right always chooses")
        };
        assert!(offered.contains(&next), "{next:?}");
        current = next;
    }
    let listed: Vec<Option<Locale>> = (0..offered.len())
        .map(|downs| {
            let mut picker = LanguagePicker::new();
            picker.on_action(Action::Select, Locale::EnUs);
            for _ in 0..downs {
                picker.on_action(Action::Down, Locale::EnUs);
            }
            picker.on_action(Action::Select, Locale::EnUs)
        })
        .collect();
    let offered: Vec<Option<Locale>> = offered.into_iter().map(Some).collect();
    assert_eq!(listed, offered);
}
