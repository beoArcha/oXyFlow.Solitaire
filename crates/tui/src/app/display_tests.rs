use std::cell::RefCell;
use std::rc::Rc;

use super::*;
use crate::settings::{FullScreen, Settings, TerminalKind, WindowSize};
use crate::storage::memory::MemoryStorage;
use crate::test_support::Harness;
use ratatui::crossterm::event::KeyCode;

#[derive(Default)]
struct Calls {
    requests: Vec<bool>,
    sizes: Vec<(u16, u16)>,
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

    fn set_window_size(&mut self, columns: u16, rows: u16) -> bool {
        self.calls.borrow_mut().sizes.push((columns, rows));
        self.obeys
    }
}

fn harness(setting: FullScreen, facts: DisplayFacts, obeys: bool) -> (Harness, Rc<RefCell<Calls>>) {
    harness_with(
        Settings {
            full_screen: setting,
            ..Settings::default()
        },
        facts,
        obeys,
    )
}

fn harness_with(
    settings: Settings,
    facts: DisplayFacts,
    obeys: bool,
) -> (Harness, Rc<RefCell<Calls>>) {
    let calls = Rc::new(RefCell::new(Calls::default()));
    let storage = MemoryStorage {
        settings: Some(settings),
        ..MemoryStorage::default()
    };
    let display = DisplayDeps {
        priority: crate::app::PriorityDeps::none(),
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

fn sizes(calls: &Rc<RefCell<Calls>>) -> Vec<(u16, u16)> {
    calls.borrow().sizes.clone()
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

fn open_full_screen_row(h: &mut Harness) {
    let settings_label = crate::i18n::t("common.menu.settings");
    let mark = format!("► {settings_label}");
    for _ in 0..16 {
        if h.lines().iter().any(|line| line.contains(&mark)) {
            break;
        }
        h.press(KeyCode::Down);
    }
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::Settings);
    h.press(KeyCode::BackTab);

    let full_screen_label = crate::i18n::t("settings.row.full_screen");
    let row_mark = format!("► {full_screen_label}");
    for _ in 0..32 {
        if h.lines().iter().any(|line| line.contains(&row_mark)) {
            break;
        }
        h.press(KeyCode::Down);
    }
}

#[test]
fn settings_screen_enters_and_leaves_full_screen_on_the_classic_console() {
    let (mut h, calls) = harness(
        FullScreen::Auto,
        facts(false, TerminalKind::ClassicConsole, None),
        true,
    );
    assert!(requests(&calls).is_empty());
    open_full_screen_row(&mut h);

    h.press(KeyCode::Right);
    assert_eq!(h.app.settings().full_screen, FullScreen::On);
    assert_eq!(requests(&calls), [true]);

    h.press(KeyCode::Right);
    assert_eq!(h.app.settings().full_screen, FullScreen::Off);
    assert_eq!(requests(&calls), [true, false]);
}

#[test]
fn settings_screen_overrides_the_launch_flag_to_leave_or_enter_full_screen() {
    let (mut forced_on, calls_on) = harness(
        FullScreen::Off,
        facts(false, TerminalKind::ClassicConsole, Some(true)),
        true,
    );
    assert_eq!(requests(&calls_on), [true]);
    open_full_screen_row(&mut forced_on);
    forced_on.press(KeyCode::Right);
    assert_eq!(forced_on.app.settings().full_screen, FullScreen::Auto);
    assert_eq!(requests(&calls_on), [true, false]);

    let (mut forced_off, calls_off) = harness(
        FullScreen::Auto,
        facts(true, TerminalKind::ClassicConsole, Some(false)),
        true,
    );
    assert!(requests(&calls_off).is_empty());
    open_full_screen_row(&mut forced_off);
    forced_off.press(KeyCode::Right);
    assert_eq!(forced_off.app.settings().full_screen, FullScreen::On);
    assert_eq!(requests(&calls_off), [true]);
}

fn open_window_size_row(h: &mut Harness) {
    open_full_screen_row(h);
    h.press(KeyCode::Down);
}

#[test]
fn each_window_size_requests_its_dimensions_on_start_when_windowed() {
    for (size, expected) in [
        (WindowSize::Small, (130, 40)),
        (WindowSize::Medium, (195, 60)),
        (WindowSize::Large, (260, 80)),
    ] {
        let (_h, calls) = harness_with(
            Settings {
                full_screen: FullScreen::Off,
                window_size: size,
                ..Settings::default()
            },
            facts(true, TerminalKind::ClassicConsole, None),
            true,
        );
        assert!(requests(&calls).is_empty(), "{size:?}");
        assert_eq!(sizes(&calls), [expected], "{size:?}");
    }

    let (_h, calls) = harness_with(
        Settings {
            full_screen: FullScreen::Auto,
            window_size: WindowSize::Medium,
            ..Settings::default()
        },
        facts(true, TerminalKind::ClassicConsole, Some(false)),
        true,
    );
    assert_eq!(sizes(&calls), [(195, 60)]);
}

#[test]
fn window_size_is_not_requested_while_full_screen_and_applies_on_leaving_full_screen() {
    let (mut h, calls) = harness_with(
        Settings {
            full_screen: FullScreen::Auto,
            window_size: WindowSize::Small,
            ..Settings::default()
        },
        facts(true, TerminalKind::ClassicConsole, None),
        true,
    );
    assert_eq!(requests(&calls), [true]);
    assert!(sizes(&calls).is_empty());

    open_window_size_row(&mut h);
    h.press(KeyCode::Right);
    assert_eq!(h.app.settings().window_size, WindowSize::Medium);
    assert!(sizes(&calls).is_empty());

    h.press(KeyCode::Up);
    h.press(KeyCode::Right);
    h.press(KeyCode::Right);
    assert_eq!(h.app.settings().full_screen, FullScreen::Off);
    assert_eq!(requests(&calls), [true, false]);
    assert_eq!(sizes(&calls), [(195, 60)]);
}

#[test]
fn changing_window_size_in_settings_requests_each_size_when_windowed() {
    let (mut h, calls) = harness_with(
        Settings {
            full_screen: FullScreen::Off,
            window_size: WindowSize::Small,
            ..Settings::default()
        },
        facts(true, TerminalKind::ClassicConsole, None),
        true,
    );
    assert_eq!(sizes(&calls), [(130, 40)]);

    open_window_size_row(&mut h);
    h.press(KeyCode::Right);
    assert_eq!(h.app.settings().window_size, WindowSize::Medium);
    h.press(KeyCode::Right);
    assert_eq!(h.app.settings().window_size, WindowSize::Large);
    h.press(KeyCode::Right);
    assert_eq!(h.app.settings().window_size, WindowSize::Small);
    assert_eq!(sizes(&calls), [(130, 40), (195, 60), (260, 80), (130, 40)]);
}

#[test]
fn window_size_is_never_requested_for_unowned_or_non_resizable_consoles() {
    for (owns, kind) in [
        (false, TerminalKind::ClassicConsole),
        (true, TerminalKind::WindowsTerminal),
        (false, TerminalKind::WindowsTerminal),
        (true, TerminalKind::Linux),
        (false, TerminalKind::Linux),
        (true, TerminalKind::MacOs),
        (false, TerminalKind::MacOs),
    ] {
        let (mut h, calls) = harness_with(
            Settings {
                full_screen: FullScreen::Off,
                window_size: WindowSize::Medium,
                ..Settings::default()
            },
            facts(owns, kind, None),
            true,
        );
        assert!(sizes(&calls).is_empty(), "owns={owns} {kind:?}");
        let next = Settings {
            window_size: WindowSize::Large,
            ..h.app.settings().clone()
        };
        h.app.apply_settings(next);
        assert!(sizes(&calls).is_empty(), "owns={owns} {kind:?}");
    }
}
