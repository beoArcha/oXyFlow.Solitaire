use super::*;
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, TRUE_COLOR};
use ratatui::crossterm::event::{
    KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use std::collections::VecDeque;

fn key(c: char) -> Input {
    Input::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE))
}

#[test]
fn events_are_translated_to_inputs() {
    let press = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(translate(Event::Key(press)), Input::Key(press));
    assert_eq!(translate(Event::Resize(80, 24)), Input::Resize(80, 24));
    assert_eq!(translate(Event::FocusLost), Input::Tick);
    let click = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 1,
        row: 2,
        modifiers: KeyModifiers::NONE,
    };
    assert_eq!(translate(Event::Mouse(click)), Input::Mouse(click));
}

#[test]
fn the_loop_draws_every_input_until_the_player_quits() {
    let Harness {
        mut app,
        mut terminal,
        ..
    } = Harness::with(MemoryStorage::new(), TRUE_COLOR, 120, 40);
    let mut script = VecDeque::from([
        Input::Tick,
        key('?'),
        Input::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
        Input::Resize(100, 30),
        key('q'),
    ]);
    let mut frames = 0;
    let result = drive(
        &mut app,
        |app| {
            frames += 1;
            if frames > 20 {
                return Err(io::Error::other("the loop did not stop"));
            }
            terminal
                .draw(|frame| app.draw(frame))
                .map(|_| ())
                .unwrap_or_else(|never| match never {});
            Ok(())
        },
        || Ok(script.pop_front().unwrap_or(Input::Tick)),
    );
    assert!(result.is_ok());
    assert!(app.should_quit());
    assert!(script.is_empty());
    assert_eq!(frames, 5);
}

#[test]
fn the_loop_never_draws_after_quit_and_stops_on_an_io_error() {
    let Harness { mut app, .. } = Harness::new();
    let mut draws = 0;
    let failed = drive(
        &mut app,
        |_| {
            draws += 1;
            Ok(())
        },
        || Err(io::Error::other("input lost")),
    );
    assert!(failed.is_err());
    assert_eq!(draws, 1);
    assert!(!app.should_quit());
}

#[test]
fn a_draw_failure_stops_the_loop() {
    let Harness { mut app, .. } = Harness::new();
    let failed = drive(
        &mut app,
        |_| Err(io::Error::other("terminal gone")),
        || Ok(Input::Tick),
    );
    assert!(failed.is_err());
}
