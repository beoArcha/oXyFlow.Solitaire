//! The board screen: the running game and its clock. Stub: P3-02 draws the board (`view.rs`) and
//! P3-03 adds the controller (cursor, picking, commands); until then Esc leads back to the menu.

mod view;

use std::time::{Duration, Instant};

use oxysol_core::Session;
use ratatui::Frame;
use ratatui::crossterm::event::KeyEvent;
use ratatui::layout::Rect;

use super::{Effect, ViewCtx};
use crate::action::Action;
use crate::app::Screen;
use crate::clock::GameClock;
use crate::storage::GameId;

/// The game being played, its history id and its clock.
pub struct BoardScreen {
    session: Option<Session>,
    game_id: Option<GameId>,
    clock: GameClock,
}

impl Default for BoardScreen {
    fn default() -> Self {
        Self {
            session: None,
            game_id: None,
            clock: GameClock::stopped(Duration::ZERO),
        }
    }
}

impl BoardScreen {
    /// A board without a game.
    pub fn new() -> Self {
        Self::default()
    }

    /// Takes over `session`, continuing from `elapsed`, and starts the clock at `now`.
    pub fn begin(&mut self, session: Session, id: Option<GameId>, elapsed: Duration, now: Instant) {
        self.session = Some(session);
        self.game_id = id;
        self.clock = GameClock::stopped(elapsed);
        self.clock.resume(now);
    }

    /// The running session, if a game was started.
    pub fn session(&self) -> Option<&Session> {
        self.session.as_ref()
    }

    /// The history id of the running game, if it could be recorded.
    pub fn game_id(&self) -> Option<GameId> {
        self.game_id
    }

    /// Playing time up to `now`.
    pub fn elapsed(&self, now: Instant) -> Duration {
        self.clock.elapsed(now)
    }

    /// Whether the clock is counting.
    pub fn clock_running(&self) -> bool {
        self.clock.is_running()
    }

    /// Stops the clock (a modal opened).
    pub fn pause_clock(&mut self, now: Instant) {
        self.clock.pause(now);
    }

    /// Continues the clock (the modal closed) when a game is running.
    pub fn resume_clock(&mut self, now: Instant) {
        if self.session.is_some() {
            self.clock.resume(now);
        }
    }

    /// Handles a key press mapped to `action`.
    pub fn handle(
        &mut self,
        _key: &KeyEvent,
        action: Option<Action>,
        _ctx: &ViewCtx<'_>,
    ) -> Effect {
        match action {
            Some(Action::Back | Action::Menu) => Effect::Goto(Screen::MainMenu),
            _ => Effect::None,
        }
    }

    /// Draws the board screen.
    pub fn draw(&mut self, frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>) {
        view::draw(frame, area, self, ctx);
    }
}
