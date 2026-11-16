//! The statistics screen: a summary per variant and the list of played games, newest first.
//! Enter replays the seed of the selected game, `c` copies it; both are also buttons. The screen
//! never reads storage itself: `App` hands it the history when the screen opens and tells it how
//! a copy ended.

mod format;
mod list;
mod summary;
mod view;

#[cfg(test)]
mod tests;

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use ratatui::layout::Rect;

use self::list::ListState;
use self::summary::{Summary, has_several_modes, summarize};
use super::Effect;
use crate::action::Action;
use crate::app::Screen;
use crate::mouse::{contains, left_press, wheel_action};
use crate::storage::GameRecord;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Button {
    Replay,
    Copy,
    Back,
}

/// What happened to the last copy, shown on the status line until the selection moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Notice {
    Copied(u64),
    CopyFailed(u64),
}

/// State of the statistics screen.
#[derive(Debug, Default)]
pub struct StatsScreen {
    records: Vec<GameRecord>,
    summaries: Vec<Summary>,
    show_modes: bool,
    list: ListState,
    load_failed: bool,
    notice: Option<Notice>,
    buttons: Vec<(Button, Rect)>,
    rows: Vec<(usize, Rect)>,
}

impl StatsScreen {
    /// An empty screen; `App` fills it with [`StatsScreen::show`] when it opens.
    pub fn new() -> Self {
        Self::default()
    }

    /// Replaces the history (in any order) and selects the newest game.
    pub fn show(&mut self, mut records: Vec<GameRecord>) {
        records.sort_by_key(|record| std::cmp::Reverse(record.id.0));
        self.summaries = summarize(&records);
        self.show_modes = has_several_modes(&records);
        self.records = records;
        self.list = ListState::default();
        self.load_failed = false;
        self.notice = None;
    }

    /// Shows that the history could not be read, instead of an empty history.
    pub fn show_failure(&mut self) {
        self.show(Vec::new());
        self.load_failed = true;
    }

    /// Records how the last copy of `seed` ended, for the status line.
    pub fn announce_copy(&mut self, seed: u64, copied: bool) {
        self.notice = Some(if copied {
            Notice::Copied(seed)
        } else {
            Notice::CopyFailed(seed)
        });
    }

    /// The selected game, `None` while the history is empty.
    pub fn selected(&self) -> Option<&GameRecord> {
        self.records.get(self.list.selected())
    }

    fn replay(&self) -> Effect {
        self.selected()
            .map_or(Effect::None, |record| Effect::Start {
                variant: record.variant,
                seed: Some(record.seed),
                undo_allowed: record.undo_allowed,
            })
    }

    fn copy(&self) -> Effect {
        self.selected()
            .map_or(Effect::None, |record| Effect::CopySeed(record.seed))
    }

    fn press(&self, button: Button) -> Effect {
        match button {
            Button::Replay => self.replay(),
            Button::Copy => self.copy(),
            Button::Back => Effect::Goto(Screen::MainMenu),
        }
    }

    fn move_selection(&mut self, step: impl FnOnce(&mut ListState, usize)) {
        let before = self.list.selected();
        step(&mut self.list, self.records.len());
        if self.list.selected() != before {
            self.notice = None;
        }
    }

    /// Handles a key press mapped to `action`: Up and Down (and Home, End, PgUp, PgDn) move
    /// through the list, Enter replays the seed, `c` copies it, Esc goes back.
    pub fn handle(&mut self, key: &KeyEvent, action: Option<Action>) -> Effect {
        match (action, key.code) {
            (Some(Action::Back), _) => return Effect::Goto(Screen::MainMenu),
            (Some(Action::Select), _) => return self.replay(),
            (Some(Action::Up), _) => self.move_selection(|list, len| list.step(-1, len)),
            (Some(Action::Down), _) => self.move_selection(|list, len| list.step(1, len)),
            (_, KeyCode::PageUp) => self.move_selection(|list, len| list.page_by(true, len)),
            (_, KeyCode::PageDown) => self.move_selection(|list, len| list.page_by(false, len)),
            (_, KeyCode::Home) => self.move_selection(|list, len| list.select(0, len)),
            (_, KeyCode::End) => self.move_selection(|list, len| list.select(len, len)),
            (_, KeyCode::Char('c' | 'C')) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                return self.copy();
            }
            _ => {}
        }
        Effect::None
    }

    /// Handles a mouse event: the wheel moves the selection, a click selects a row or presses a
    /// button.
    pub fn handle_mouse(&mut self, event: &MouseEvent) -> Effect {
        if let Some(action) = wheel_action(event) {
            let delta = if action == Action::Up { -1 } else { 1 };
            self.move_selection(|list, len| list.step(delta, len));
            return Effect::None;
        }
        let Some(cell) = left_press(event) else {
            return Effect::None;
        };
        if let Some(button) = self
            .buttons
            .iter()
            .find(|(_, area)| contains(*area, cell))
            .map(|(button, _)| *button)
        {
            return self.press(button);
        }
        if let Some(index) = self
            .rows
            .iter()
            .find(|(_, area)| contains(*area, cell))
            .map(|(index, _)| *index)
        {
            self.move_selection(|list, len| list.select(index, len));
        }
        Effect::None
    }
}
