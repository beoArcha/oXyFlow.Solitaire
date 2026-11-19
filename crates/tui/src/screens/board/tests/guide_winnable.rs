//! The guide window shows the same game facts as the header and the `Game` box, the winnable
//! verdict of a checked deal included.

use oxysol_core::{DrawCount, Session, Variant};
use ratatui::crossterm::event::KeyCode;

use crate::input::Input;
use crate::screens::Effect;
use crate::solver::Proof;
use crate::storage::memory::{HistoryRow, MemoryStorage};
use crate::storage::{GameId, SavedGame};
use crate::test_support::{Harness, TRUE_COLOR};

const KLONDIKE: Variant = Variant::Klondike(DrawCount::One);

#[test]
fn the_winnable_verdict_shows_in_the_header_and_in_the_game_section_of_the_window() {
    let storage = MemoryStorage {
        game: Some(SavedGame {
            id: GameId(7),
            record: Session::new(KLONDIKE, 5).record(),
            elapsed_ms: 1_000,
            undo_allowed: true,
        }),
        history: vec![HistoryRow::open(7, KLONDIKE, 5)],
        ..MemoryStorage::default()
    };
    let mut h = Harness::with(storage, TRUE_COLOR, 100, 30);
    h.solver.borrow_mut().verdicts.insert(5, Proof::Winnable);
    h.app.apply(Effect::Resume);
    h.send(Input::Tick);
    assert!(h.lines()[0].contains("Winnable: yes"), "{}", h.lines()[0]);
    h.type_char('g');
    let lines = h.lines();
    let game = lines.iter().position(|line| line.contains("Game"));
    let facts: String = game.map_or(String::new(), |row| lines[row..row + 8].concat());
    assert!(facts.contains("Winnable: yes"), "{facts}");
    h.press(KeyCode::Esc);
    assert!(h.lines()[0].contains("Winnable: yes"));
}
