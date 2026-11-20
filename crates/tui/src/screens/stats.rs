//! The statistics screen: a summary per variant and the list of played games, newest first.
//! Enter replays the seed of the selected game, `c` copies it, `w` checks whether it was winnable
//! and `W` checks every unchecked game; all of them are buttons too. Tab moves the focus through
//! the list and then the buttons, wrapping. The screen never reads storage or the solver itself:
//! `App` hands it the history when the screen opens, submits the checks it asks for and reports the
//! answers and how a copy ended.

mod check;
mod format;
mod list;
mod status;
mod summary;
mod view;

pub use self::check::Target;

#[cfg(test)]
mod tests;

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use ratatui::layout::Rect;

use self::check::Run;
use self::list::ListState;
use self::summary::{Summary, has_several_modes, summarize};
use super::Effect;
use super::focus::{step, tab_forward};
use crate::action::Action;
use crate::app::Screen;
use crate::mouse::{contains, left_press, wheel_action};
use crate::storage::GameRecord;
use crate::widgets::footer::FooterMap;
use crate::widgets::scrollbar::{self, Metrics};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Button {
    Replay,
    Copy,
    Check,
    CheckAll,
    Back,
}

/// What has the focus: the history list, or one of the buttons.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Stop {
    #[default]
    List,
    Button(Button),
}

/// What happened to the last copy, shown on the status line until the selection moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Notice {
    Copied(u64),
    CopyFailed(u64),
    AlreadyChecked,
    NothingToCheck,
    Finished(usize),
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
    run: Option<Run>,
    stop: Stop,
    buttons: Vec<(Button, Rect)>,
    rows: Vec<(usize, Rect)>,
    footer: FooterMap,
    drag: Option<u16>,
    track: Rect,
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
        self.stop = Stop::List;
        self.load_failed = false;
        self.notice = None;
        self.run = None;
        self.drag = None;
        self.track = Rect::default();
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

    /// The buttons in the order Tab visits them: the game buttons only when a game can be acted on.
    fn visible_buttons(&self) -> Vec<Button> {
        let mut buttons = Vec::with_capacity(5);
        if !self.records.is_empty() {
            buttons.extend([
                Button::Replay,
                Button::Copy,
                Button::Check,
                Button::CheckAll,
            ]);
        }
        buttons.push(Button::Back);
        buttons
    }

    fn stops(&self) -> Vec<Stop> {
        std::iter::once(Stop::List)
            .chain(self.visible_buttons().into_iter().map(Stop::Button))
            .collect()
    }

    fn step_focus(&mut self, forward: bool) {
        let stops = self.stops();
        let at = stops
            .iter()
            .position(|stop| *stop == self.stop)
            .unwrap_or(0);
        if let Some(stop) = stops.get(step(at, stops.len(), forward)) {
            self.stop = *stop;
        }
    }

    fn press(&mut self, button: Button) -> Effect {
        match button {
            Button::Replay => self.replay(),
            Button::Copy => self.copy(),
            Button::Check => self.check(false),
            Button::CheckAll => self.check(true),
            Button::Back => Effect::Goto(Screen::MainMenu),
        }
    }

    /// Enter: replays the selected game when the list has the focus, else presses the button.
    fn activate(&mut self) -> Effect {
        match self.stop {
            Stop::List => self.replay(),
            Stop::Button(button) => self.press(button),
        }
    }

    fn move_selection(&mut self, step: impl FnOnce(&mut ListState, usize)) {
        self.stop = Stop::List;
        let before = self.list.selected();
        step(&mut self.list, self.records.len());
        if self.list.selected() != before {
            self.notice = None;
        }
    }

    /// Handles a key press mapped to `action`: Tab and Shift+Tab move the focus, Up and Down (and
    /// Home, End, PgUp, PgDn) move through the list, Enter activates the focused element, `c`
    /// copies the seed, `w` and `W` check winnability, Esc goes back.
    pub fn handle(&mut self, key: &KeyEvent, action: Option<Action>) -> Effect {
        if let Some(forward) = action.and_then(tab_forward) {
            self.step_focus(forward);
            return Effect::None;
        }
        match (action, key.code) {
            (Some(Action::Back), _) => return Effect::Goto(Screen::MainMenu),
            (Some(Action::Select), _) => return self.activate(),
            (Some(Action::Up), _) => self.move_selection(|list, len| list.step(-1, len)),
            (Some(Action::Down), _) => self.move_selection(|list, len| list.step(1, len)),
            (_, KeyCode::PageUp) => self.move_selection(|list, len| list.page_by(true, len)),
            (_, KeyCode::PageDown) => self.move_selection(|list, len| list.page_by(false, len)),
            (_, KeyCode::Home) => self.move_selection(|list, len| list.select(0, len)),
            (_, KeyCode::End) => self.move_selection(|list, len| list.select(len, len)),
            (_, KeyCode::Char('c' | 'C')) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                return self.copy();
            }
            (_, KeyCode::Char('w')) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                return self.check(false);
            }
            (_, KeyCode::Char('W')) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                return self.check(true);
            }
            _ => {}
        }
        Effect::None
    }

    /// The footer button under `cell`, if any.
    pub fn footer_action_at(&self, cell: (u16, u16)) -> Option<Action> {
        self.footer.action_at(cell)
    }

    /// Handles a mouse event: the wheel moves the selection, a click selects a row or presses a
    /// button, which takes the focus.
    pub fn handle_mouse(&mut self, event: &MouseEvent) -> Effect {
        if let Some(action) = wheel_action(event) {
            let delta = if action == Action::Up { -1 } else { 1 };
            self.move_selection(|list, len| list.step(delta, len));
            return Effect::None;
        }
        let metrics = Metrics {
            total: self.records.len(),
            view: usize::from(self.track.height),
            offset: self.list.offset(),
        };
        if let Some(line) = scrollbar::handle_mouse(event, self.track, metrics, &mut self.drag) {
            self.move_selection(|list, len| list.scroll_to_offset(line, len));
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
            self.stop = Stop::Button(button);
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
