use std::cell::RefCell;
use std::rc::Rc;

use super::*;
use crate::settings::{ProcessPriority, Settings};
use crate::storage::memory::MemoryStorage;
use crate::test_support::Harness;

#[derive(Default)]
struct Asked {
    levels: Vec<ProcessPriority>,
}

struct Fake {
    asked: Rc<RefCell<Asked>>,
    outcome: PriorityOutcome,
}

impl PriorityControl for Fake {
    fn apply(&mut self, level: ProcessPriority) -> PriorityOutcome {
        self.asked.borrow_mut().levels.push(level);
        self.outcome
    }
}

fn harness(
    saved: ProcessPriority,
    flag: Option<ProcessPriority>,
    outcome: PriorityOutcome,
) -> (Harness, Rc<RefCell<Asked>>) {
    let asked = Rc::new(RefCell::new(Asked::default()));
    let storage = MemoryStorage {
        settings: Some(Settings {
            process_priority: saved,
            ..Settings::default()
        }),
        ..MemoryStorage::default()
    };
    let display = DisplayDeps {
        priority: PriorityDeps {
            control: Box::new(Fake {
                asked: Rc::clone(&asked),
                outcome,
            }),
            flag,
        },
        ..DisplayDeps::none()
    };
    (Harness::with_display(storage, display), asked)
}

fn levels(asked: &Rc<RefCell<Asked>>) -> Vec<ProcessPriority> {
    asked.borrow().levels.clone()
}

fn with_level(h: &Harness, level: ProcessPriority) -> Settings {
    Settings {
        process_priority: level,
        ..h.app.settings.clone()
    }
}

#[test]
fn the_saved_level_is_applied_once_at_start() {
    let (_h, asked) = harness(ProcessPriority::Low, None, PriorityOutcome::Applied);
    assert_eq!(levels(&asked), [ProcessPriority::Low]);
}

#[test]
fn the_flag_beats_the_saved_level_at_start() {
    let (h, asked) = harness(
        ProcessPriority::Low,
        Some(ProcessPriority::Highest),
        PriorityOutcome::Applied,
    );
    assert_eq!(levels(&asked), [ProcessPriority::Highest]);
    assert_eq!(h.app.settings.process_priority, ProcessPriority::Low);
}

#[test]
fn changing_the_setting_applies_at_once_and_ends_the_flag() {
    let (mut h, asked) = harness(
        ProcessPriority::Normal,
        Some(ProcessPriority::Highest),
        PriorityOutcome::Applied,
    );
    let next = with_level(&h, ProcessPriority::Lowest);
    h.app.apply_settings(next);
    assert_eq!(
        levels(&asked),
        [ProcessPriority::Highest, ProcessPriority::Lowest]
    );
}

#[test]
fn an_unchanged_level_is_not_asked_again_on_other_changes() {
    let (mut h, asked) = harness(ProcessPriority::Normal, None, PriorityOutcome::Applied);
    let next = Settings {
        volume: 20,
        ..h.app.settings.clone()
    };
    h.app.apply_settings(next);
    assert_eq!(levels(&asked), [ProcessPriority::Normal]);
}

#[test]
fn a_refusal_is_shown_on_the_row_and_asked_only_once() {
    let (mut h, asked) = harness(
        ProcessPriority::Normal,
        None,
        PriorityOutcome::NeedsPrivileges,
    );
    let next = with_level(&h, ProcessPriority::High);
    h.app.apply_settings(next);
    h.app.apply_settings(h.app.settings.clone());
    assert_eq!(
        levels(&asked),
        [ProcessPriority::Normal, ProcessPriority::High]
    );
    assert_eq!(
        h.app.font_facts().priority,
        PriorityOutcome::NeedsPrivileges
    );
}

#[test]
fn the_priority_is_saved_with_the_settings_and_loads_back() {
    let (mut h, _asked) = harness(ProcessPriority::Normal, None, PriorityOutcome::Applied);
    let next = with_level(&h, ProcessPriority::Lowest);
    h.app.apply_settings(next);
    let saved = h.app.storage.load_settings().ok().flatten();
    assert_eq!(
        saved.map(|settings| settings.process_priority),
        Some(ProcessPriority::Lowest)
    );
}

#[test]
fn a_preview_applies_at_once_and_undoing_it_applies_the_old_level_again() {
    let (mut h, asked) = harness(ProcessPriority::Normal, None, PriorityOutcome::Applied);
    let next = with_level(&h, ProcessPriority::Low);
    h.app.preview_settings(next);
    assert!(h.app.previewing());
    h.app.leave_settings(false, Screen::MainMenu);
    assert_eq!(
        levels(&asked),
        [
            ProcessPriority::Normal,
            ProcessPriority::Low,
            ProcessPriority::Normal
        ]
    );
    assert_eq!(h.app.settings.process_priority, ProcessPriority::Normal);
}
