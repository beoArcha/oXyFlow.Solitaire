//! Input events routing and handling: key presses, mouse clicks, text capture, and modal dispatch.

use std::time::Instant;

use ratatui::crossterm::event::{KeyEvent, KeyEventKind, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

use super::{App, Modal, Screen};
use crate::action::Action;
use crate::input::{Input, action_for_key, is_text_key, primary_key_event};
use crate::mouse::left_press;
use crate::screens::{Effect, ViewCtx};
use crate::widgets::overlay;
use crate::widgets::too_small::is_too_small;

impl App {
    pub(super) fn too_small(&self) -> bool {
        self.area
            .is_some_and(|(width, height)| is_too_small(Rect::new(0, 0, width, height)))
    }

    /// Handles one input event. `now` drives the clock; the application never reads the time.
    pub fn handle(&mut self, input: Input, now: Instant) {
        self.now = now;
        let effect = match input {
            Input::Key(key) => self.on_key(key),
            Input::Mouse(event) => {
                self.pointer = Some((event.column, event.row));
                self.on_mouse(&event)
            }
            Input::Resize(width, height) => {
                self.area = Some((width, height));
                self.board.resize(width, height);
                Effect::None
            }
            Input::Tick => Effect::None,
        };
        self.apply(effect);
        self.poll_solver();
        let effect = self.advance_finish();
        self.apply(effect);
        self.play_board_cues();
    }

    fn play_board_cues(&mut self) {
        for cue in self.board.take_cues() {
            self.audio.play(cue);
        }
    }

    fn on_key(&mut self, key: KeyEvent) -> Effect {
        if key.kind == KeyEventKind::Release {
            return Effect::None;
        }
        if let Some(step) = super::font_step_for_key(&key)
            && self.full_screen.font.step(step)
        {
            return Effect::None;
        }
        let action = action_for_key(&key);
        self.run(key, action)
    }

    /// Runs one key press, or one footer click (which sends the primary key of its action): the
    /// same path for both, so a button does exactly what its key does.
    pub(super) fn run(&mut self, key: KeyEvent, action: Option<Action>) -> Effect {
        if action == Some(Action::ForceQuit) {
            return Effect::Quit;
        }
        self.board.interrupt_finish();
        if self.too_small() {
            return if action == Some(Action::Quit) {
                Effect::Quit
            } else {
                Effect::None
            };
        }
        if let Some(effect) = self.dock_route(&key, action) {
            return effect;
        }
        if !self.types_text(&key) {
            match action {
                Some(Action::Help) => return Effect::Open(Modal::help()),
                Some(Action::Rules) => return Effect::HowToPlay,
                _ => {}
            }
        }
        let ctx = view_ctx!(self);
        match self.screen {
            Screen::MainMenu => self.menu.handle(&key, action, &ctx),
            Screen::GameSelect => self.select.handle(&key, action, &ctx),
            Screen::Board => self.board.handle(&key, action, &ctx),
            Screen::Settings => self.settings_screen.handle(&key, action, &ctx),
            Screen::Stats => self.stats.handle(&key, action),
        }
    }

    /// The footer button under `cell` on the active screen, if any; no modal may be open.
    fn footer_action_at(&self, cell: (u16, u16)) -> Option<Action> {
        if self.modal.is_some() && self.modal_takes_keys() {
            return self.modal_footer.action_at(cell);
        }
        match self.screen {
            Screen::MainMenu => self.menu.footer_action_at(cell),
            Screen::GameSelect => self.select.footer_action_at(cell),
            Screen::Board => self.board.footer_action_at(cell),
            Screen::Settings => self.settings_screen.footer_action_at(cell),
            Screen::Stats => self.stats.footer_action_at(cell),
        }
    }

    /// Whether the active screen is typing into a text field.
    fn captures_text(&self) -> bool {
        match self.screen {
            Screen::MainMenu => self.menu.captures_text(),
            Screen::GameSelect | Screen::Board | Screen::Settings | Screen::Stats => false,
        }
    }

    /// Whether `key` is typed text for the active screen, so that it is not a global shortcut.
    fn types_text(&self, key: &KeyEvent) -> bool {
        self.captures_text() && is_text_key(key)
    }

    pub(super) fn on_modal_mouse(&mut self, event: &MouseEvent) -> Effect {
        let area = self.screen_rect();
        let outcome = self
            .modal
            .as_mut()
            .and_then(|modal| overlay::handle_mouse(modal, area, &self.board, self.now, event));
        self.sync_quick();
        self.finish_modal(outcome)
    }

    pub(super) fn finish_modal(&mut self, outcome: Option<Effect>) -> Effect {
        match outcome {
            Some(effect) => {
                self.close_modal();
                effect
            }
            None => Effect::None,
        }
    }

    pub(super) fn screen_rect(&self) -> Rect {
        self.area.map_or(Rect::default(), |(width, height)| {
            Rect::new(0, 0, width, height)
        })
    }

    fn on_mouse(&mut self, event: &MouseEvent) -> Effect {
        if self.too_small() {
            return Effect::None;
        }
        if let Some(action) = left_press(event).and_then(|cell| self.footer_action_at(cell)) {
            return self.on_footer_click(action);
        }
        if matches!(event.kind, MouseEventKind::Down(_)) {
            self.board.interrupt_finish();
        }
        if let Some(effect) = self.dock_mouse(event) {
            return effect;
        }
        let ctx = view_ctx!(self);
        match self.screen {
            Screen::MainMenu => self.menu.handle_mouse(event, &ctx),
            Screen::GameSelect => self.select.handle_mouse(event, &ctx),
            Screen::Board => self.board.handle_mouse(event, &ctx),
            Screen::Settings => self.settings_screen.handle_mouse(event, &ctx),
            Screen::Stats => self.stats.handle_mouse(event),
        }
    }

    fn on_footer_click(&mut self, action: Action) -> Effect {
        match primary_key_event(action) {
            Some(key) => self.run(key, Some(action)),
            None => Effect::None,
        }
    }
}
