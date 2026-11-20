use std::cell::RefCell;
use std::rc::Rc;

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::*;
use crate::input::Input;
use crate::storage::memory::MemoryStorage;
use crate::test_support::Harness;

struct Recording {
    steps: Rc<RefCell<Vec<FontStep>>>,
    obeys: bool,
}

impl ConsoleFont for Recording {
    fn step(&mut self, step: FontStep) -> bool {
        self.steps.borrow_mut().push(step);
        self.obeys
    }
}

fn harness(obeys: bool) -> (Harness, Rc<RefCell<Vec<FontStep>>>) {
    let steps = Rc::new(RefCell::new(Vec::new()));
    let display = DisplayDeps {
        font: Box::new(Recording {
            steps: Rc::clone(&steps),
            obeys,
        }),
        ..DisplayDeps::none()
    };
    (Harness::with_display(MemoryStorage::new(), display), steps)
}

fn ctrl(h: &mut Harness, code: char) {
    h.send(Input::Key(KeyEvent::new(
        KeyCode::Char(code),
        KeyModifiers::CONTROL,
    )));
}

#[test]
fn ctrl_plus_minus_and_zero_ask_the_console_font() {
    let (mut h, steps) = harness(true);
    ctrl(&mut h, '=');
    ctrl(&mut h, '-');
    ctrl(&mut h, '0');
    assert_eq!(
        *steps.borrow(),
        [FontStep::Larger, FontStep::Smaller, FontStep::Reset]
    );
}

#[test]
fn the_font_keys_work_in_a_terminal_that_is_too_small() {
    let (mut h, steps) = harness(true);
    h.resize(80, 24);
    assert!(h.text().contains("Terminal too small"));
    ctrl(&mut h, '-');
    assert_eq!(*steps.borrow(), [FontStep::Smaller]);
}

#[test]
fn plain_keys_never_reach_the_console_font() {
    let (mut h, steps) = harness(true);
    for code in ['=', '-', '0', '+', '[', ']'] {
        h.type_char(code);
    }
    assert!(steps.borrow().is_empty());
}

#[test]
fn a_console_that_refuses_leaves_the_key_to_the_application() {
    let (mut h, steps) = harness(false);
    ctrl(&mut h, '0');
    assert_eq!(*steps.borrow(), [FontStep::Reset]);
    assert_eq!(h.app.screen(), Screen::MainMenu);
    assert!(!h.app.should_quit());
}
