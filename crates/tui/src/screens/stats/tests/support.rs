use oxysol_core::{DrawCount, Variant};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

use crate::app::Screen;
use crate::input::Input;
use crate::storage::Outcome::{Abandoned, Won};
use crate::storage::memory::{HistoryRow, MemoryStorage};
use crate::test_support::{Harness, TRUE_COLOR};

pub(super) const KLONDIKE: Variant = Variant::Klondike(DrawCount::One);
pub(super) const STATISTICS_ITEM: usize = 2;

pub(super) fn row(
    id: i64,
    variant: Variant,
    outcome: crate::storage::Outcome,
    ms: u64,
) -> HistoryRow {
    HistoryRow {
        moves: Some(40 + u32::try_from(id).unwrap_or(0)),
        ..HistoryRow::finished(
            id,
            variant,
            u64::try_from(id).unwrap_or(0) * 11,
            outcome,
            ms,
        )
    }
}

pub(super) fn three_games() -> MemoryStorage {
    MemoryStorage {
        history: vec![
            row(1, KLONDIKE, Won, 100_000),
            row(2, Variant::FreeCell, Abandoned, 30_000),
            row(3, KLONDIKE, Won, 75_000),
        ],
        ..MemoryStorage::new()
    }
}

pub(super) fn many_games(count: i64) -> MemoryStorage {
    MemoryStorage {
        history: (1..=count)
            .map(|id| {
                row(
                    id,
                    KLONDIKE,
                    Won,
                    60_000 + 1_000 * u64::try_from(id).unwrap_or(0),
                )
            })
            .collect(),
        ..MemoryStorage::new()
    }
}

pub(super) fn open_stats(storage: MemoryStorage, width: u16, height: u16) -> Harness {
    let mut h = Harness::with(storage, TRUE_COLOR, width, height);
    for _ in 0..STATISTICS_ITEM {
        h.press(KeyCode::Down);
    }
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::Stats);
    h
}

pub(super) fn click_text(h: &mut Harness, text: &str) {
    let (column, row) = h.find(text).unwrap_or_default();
    h.click(column, row);
}

pub(super) fn wheel(h: &mut Harness, kind: MouseEventKind) {
    h.send(Input::Mouse(MouseEvent {
        kind,
        column: 50,
        row: 15,
        modifiers: KeyModifiers::NONE,
    }));
}

pub(super) fn ctrl_c(h: &mut Harness) {
    h.send(Input::Key(KeyEvent::new(
        KeyCode::Char('c'),
        KeyModifiers::CONTROL,
    )));
}
