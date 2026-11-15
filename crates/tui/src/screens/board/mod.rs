//! The board screen: the running game, its clock and the controller — the cursor, picking and
//! dropping, and the commands. The drawing is in `view.rs`; every rule comes from the engine
//! through `Session`, the screen only turns keys into moves.

mod choices;
mod commands;
mod cursor;
mod marks;
mod mouse;
mod moves;
mod status;
#[cfg(test)]
mod tests;
mod view;

use std::time::{Duration, Instant};

use oxysol_core::{Move, PileRef, Session};
use ratatui::Frame;
use ratatui::crossterm::event::KeyEvent;
use ratatui::layout::Rect;

use self::choices::Choice;
use self::cursor::Cursor;
use self::mouse::Pointer;
use super::{Effect, ViewCtx};
use crate::action::Action;
use crate::announce;
use crate::audio::SoundCue;
use crate::clock::GameClock;
use crate::i18n::t;
use crate::storage::GameId;
use crate::widgets::pile::Marks;

const DEFAULT_SCREEN: Rect = Rect::new(0, 0, 120, 40);

/// Cards picked up, waiting for a target.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Pick {
    source: PileRef,
    count: u8,
    choices: Vec<Choice>,
}

/// The game being played, its history id, its clock and the state of the controller.
pub struct BoardScreen {
    session: Option<Session>,
    game_id: Option<GameId>,
    clock: GameClock,
    cursor: Cursor,
    pick: Option<Pick>,
    hint: Option<Move>,
    status: Option<&'static str>,
    first_steps: bool,
    cues: Vec<SoundCue>,
    dirty: bool,
    screen: Rect,
    covered: bool,
    pointer: Pointer,
}

impl Default for BoardScreen {
    fn default() -> Self {
        Self {
            session: None,
            game_id: None,
            clock: GameClock::stopped(Duration::ZERO),
            cursor: Cursor {
                pile: PileRef::Stock,
                depth: 1,
            },
            pick: None,
            hint: None,
            status: None,
            first_steps: false,
            cues: Vec::new(),
            dirty: false,
            screen: DEFAULT_SCREEN,
            covered: false,
            pointer: Pointer::default(),
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
        self.cursor = Cursor::initial(&session.game().piles());
        self.first_steps = !session.can_undo();
        self.session = Some(session);
        self.game_id = id;
        self.clock = GameClock::stopped(elapsed);
        self.clock.resume(now);
        self.cues.clear();
        self.pointer = Pointer::default();
        self.settle();
    }

    /// Back to the dealt position of the same seed with an empty history and a zeroed clock.
    pub fn restart(&mut self, now: Instant) {
        let Some(session) = self.session.as_mut() else {
            return;
        };
        session.restart();
        self.cursor = Cursor::initial(&session.game().piles());
        self.pointer = Pointer::default();
        self.clock = GameClock::stopped(Duration::ZERO);
        self.clock.resume(now);
        self.settle();
        self.announce_event(status::RESTARTED);
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

    /// Continues the clock (the modal closed) while a game is running and not yet won.
    pub fn resume_clock(&mut self, now: Instant) {
        if self.session.is_some() && !self.is_won() {
            self.clock.resume(now);
        }
    }

    /// Tells the board how large the terminal is; cursor movement follows the drawn geometry.
    pub fn resize(&mut self, width: u16, height: u16) {
        self.screen = Rect::new(0, 0, width, height);
    }

    /// Hides the cards while the game is paused.
    pub fn set_covered(&mut self, covered: bool) {
        self.covered = covered;
    }

    /// The sound cues emitted since the last call, oldest first.
    pub fn take_cues(&mut self) -> Vec<SoundCue> {
        std::mem::take(&mut self.cues)
    }

    /// Whether the session changed since the last call; the application then saves the game.
    pub fn take_dirty(&mut self) -> bool {
        std::mem::take(&mut self.dirty)
    }

    /// The localization key of the status message, if one is shown.
    pub fn status_key(&self) -> Option<&'static str> {
        self.status
    }

    /// The highlights of the cursor, the picked run and its targets, and the hint.
    pub fn marks(&self) -> Marks {
        let Some(session) = &self.session else {
            return Marks::default();
        };
        marks::build(
            &session.game().piles(),
            &self.cursor,
            self.pick.as_ref(),
            self.hint,
        )
    }

    fn status_text(&self) -> String {
        let focus = self.focus_text();
        let event = match self.status {
            Some(key) => t(key).into_owned(),
            None => self.fitted_first_steps(&focus, usize::from(self.screen.width)),
        };
        announce::announce(&focus, &event)
    }

    /// Handles a key press mapped to `action`; the effect is for the application to apply.
    pub fn handle(
        &mut self,
        _key: &KeyEvent,
        action: Option<Action>,
        _ctx: &ViewCtx<'_>,
    ) -> Effect {
        action.map_or(Effect::None, |action| self.dispatch(action))
    }

    /// Draws the board screen.
    pub fn draw(&mut self, frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>) {
        self.screen = area;
        view::draw(frame, area, self, ctx);
    }
}
