use std::sync::Arc;

use super::fake::{Answer, FakeNative};
use super::*;

fn control(flavour: Flavour, answer: Answer) -> (ProcessControl, Arc<FakeNative>) {
    let fake = Arc::new(FakeNative::new(flavour, answer));
    (ProcessControl::new(fake.clone()), fake)
}

#[test]
fn windows_maps_each_level_to_its_priority_class() {
    let expected = [
        (ProcessPriority::Highest, 0x80),
        (ProcessPriority::High, 0x8000),
        (ProcessPriority::Normal, 0x20),
        (ProcessPriority::Low, 0x4000),
        (ProcessPriority::Lowest, 0x40),
    ];
    for (level, class) in expected {
        let (mut control, fake) = control(Flavour::Windows, Answer::Accept);
        assert_eq!(control.apply(level), PriorityOutcome::Applied);
        assert_eq!(fake.process_calls(), vec![class], "{level:?}");
    }
}

#[test]
fn windows_never_asks_for_the_real_time_class() {
    const REALTIME_PRIORITY_CLASS: i32 = 0x100;
    for level in ProcessPriority::ALL {
        assert_ne!(
            native_value(Flavour::Windows, level),
            REALTIME_PRIORITY_CLASS
        );
    }
}

#[test]
fn unix_maps_each_level_to_a_nice_value() {
    let expected = [
        (ProcessPriority::Highest, -10),
        (ProcessPriority::High, -5),
        (ProcessPriority::Normal, 0),
        (ProcessPriority::Low, 5),
        (ProcessPriority::Lowest, 10),
    ];
    for (level, nice) in expected {
        let (mut control, fake) = control(Flavour::Unix, Answer::Accept);
        assert_eq!(control.apply(level), PriorityOutcome::Applied);
        assert_eq!(fake.process_calls(), vec![nice], "{level:?}");
    }
}

#[test]
fn a_refusal_for_privileges_is_a_result_and_names_the_privilege_note() {
    let (mut control, fake) = control(Flavour::Unix, Answer::Deny);
    assert_eq!(
        control.apply(ProcessPriority::Highest),
        PriorityOutcome::NeedsPrivileges
    );
    assert_eq!(fake.process_calls(), vec![-10]);
    assert_eq!(
        PriorityOutcome::NeedsPrivileges.note_key(),
        Some("settings.value.needs_privileges")
    );
}

#[test]
fn any_other_refusal_is_a_failed_outcome() {
    let (mut control, _fake) = control(Flavour::Windows, Answer::Refuse);
    assert_eq!(control.apply(ProcessPriority::Low), PriorityOutcome::Failed);
}

#[test]
fn the_solver_thread_is_lowered_once_per_call_and_a_refusal_changes_nothing() {
    for answer in [Answer::Accept, Answer::Deny, Answer::Refuse] {
        let fake = FakeNative::new(Flavour::Unix, answer);
        lower_solver_thread(&fake);
        assert_eq!(fake.thread_calls(), 1);
        assert!(fake.process_calls().is_empty());
    }
}

#[test]
fn the_solver_nice_is_above_every_level_the_player_can_choose() {
    for level in ProcessPriority::ALL {
        assert!(SOLVER_NICE > native_value(Flavour::Unix, level));
    }
}
