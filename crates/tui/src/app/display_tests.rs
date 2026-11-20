use std::cell::RefCell;
use std::rc::Rc;

use super::*;
use crate::settings::{FullScreen, Settings, TerminalKind};
use crate::storage::memory::MemoryStorage;
use crate::test_support::Harness;
use ratatui::crossterm::event::KeyCode;

#[derive(Default)]
struct Calls {
    requests: Vec<bool>,
}

struct Recording {
    calls: Rc<RefCell<Calls>>,
    obeys: bool,
}

impl ConsoleDisplay for Recording {
    fn set_full_screen(&mut self, on: bool) -> bool {
        self.calls.borrow_mut().requests.push(on);
        self.obeys
    }
}

fn harness(setting: FullScreen, facts: DisplayFacts, obeys: bool) -> (Harness, Rc<RefCell<Calls>>) {
    let calls = Rc::new(RefCell::new(Calls::default()));
    let storage = MemoryStorage {
        settings: Some(Settings {
            full_screen: setting,
            ..Settings::default()
        }),
        ..MemoryStorage::default()
    };
    let display = DisplayDeps {
        display: Box::new(Recording {
            calls: Rc::clone(&calls),
            obeys,
        }),
        facts,
        font: Box::new(NullConsoleFont),
    };
    (Harness::with_display(storage, display), calls)
}

fn facts(owns_window: bool, kind: TerminalKind, flag: Option<bool>) -> DisplayFacts {
    DisplayFacts {
        owns_window,
        kind,
        flag,
    }
}

fn start_game(h: &mut Harness) {
    h.press(KeyCode::Enter);
    h.press(KeyCode::Enter);
}

fn requests(calls: &Rc<RefCell<Calls>>) -> Vec<bool> {
    calls.borrow().requests.clone()
}

#[test]
fn auto_enters_full_screen_in_a_window_the_game_owns() {
    let (h, calls) = harness(
        FullScreen::Auto,
        facts(true, TerminalKind::ClassicConsole, None),
        true,
    );
    assert_eq!(requests(&calls), [true]);
    drop(h);
}

#[test]
fn auto_leaves_an_existing_console_alone() {
    let (mut h, calls) = harness(
        FullScreen::Auto,
        facts(false, TerminalKind::ClassicConsole, None),
        true,
    );
    start_game(&mut h);
    assert!(requests(&calls).is_empty());
    assert!(!h.text().contains("Full screen"));
}

#[test]
fn the_flag_forces_full_screen_in_an_existing_console() {
    let (_h, calls) = harness(
        FullScreen::Off,
        facts(false, TerminalKind::ClassicConsole, Some(true)),
        true,
    );
    assert_eq!(requests(&calls), [true]);
}

#[test]
fn the_no_flag_keeps_an_owned_window_windowed() {
    let (_h, calls) = harness(
        FullScreen::On,
        facts(true, TerminalKind::ClassicConsole, Some(false)),
        true,
    );
    assert!(requests(&calls).is_empty());
}

#[test]
fn a_refused_switch_names_alt_enter_once_in_the_now_slot() {
    let (mut h, calls) = harness(
        FullScreen::On,
        facts(false, TerminalKind::ClassicConsole, None),
        false,
    );
    start_game(&mut h);
    assert_eq!(requests(&calls), [true]);
    assert!(h.text().contains("Full screen: press Alt+Enter"));
    h.press(KeyCode::Right);
    assert!(!h.text().contains("Full screen: press"));
    h.app.goto(Screen::MainMenu);
    h.press(KeyCode::Enter);
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::Board);
    assert!(!h.text().contains("Full screen: press"));
}

#[test]
fn a_terminal_the_game_cannot_switch_gets_its_own_key_named() {
    for (kind, text) in [
        (
            TerminalKind::WindowsTerminal,
            "Full screen: press F11 or Alt+Enter",
        ),
        (TerminalKind::Linux, "Full screen: press F11"),
        (TerminalKind::MacOs, "Full screen: press Ctrl+Cmd+F"),
    ] {
        let (mut h, calls) = harness(FullScreen::On, facts(false, kind, None), true);
        start_game(&mut h);
        assert!(requests(&calls).is_empty(), "{kind:?}");
        assert!(h.text().contains(text), "{kind:?}");
    }
}

#[test]
fn changing_the_setting_in_the_game_switches_at_once_and_drops_the_flag() {
    let (mut h, calls) = harness(
        FullScreen::On,
        facts(false, TerminalKind::ClassicConsole, None),
        true,
    );
    assert_eq!(requests(&calls), [true]);
    let off = Settings {
        full_screen: FullScreen::Off,
        ..h.app.settings().clone()
    };
    h.app.apply_settings(off);
    assert_eq!(requests(&calls), [true, false]);
    let on = Settings {
        full_screen: FullScreen::On,
        ..h.app.settings().clone()
    };
    h.app.apply_settings(on);
    assert_eq!(requests(&calls), [true, false, true]);
}

#[test]
fn a_setting_change_that_is_not_full_screen_never_touches_the_console() {
    let (mut h, calls) = harness(
        FullScreen::Off,
        facts(false, TerminalKind::ClassicConsole, None),
        true,
    );
    let louder = Settings {
        volume: 10,
        ..h.app.settings().clone()
    };
    h.app.apply_settings(louder);
    assert!(requests(&calls).is_empty());
}
