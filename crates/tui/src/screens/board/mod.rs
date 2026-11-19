//! The board screen: the running game, its clock and the controller — the cursor, picking and
//! dropping, and the commands. The drawing is in `view.rs`; every rule comes from the engine
//! through `Session`, the screen only turns keys into moves.

mod choices;
mod commands;
mod counters;
mod cursor;
mod guidance;
mod header;
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
use self::header::Header;
use self::mouse::Pointer;
use super::{Effect, ViewCtx};
use crate::action::Action;
use crate::announce;
use crate::audio::SoundCue;
use crate::clock::GameClock;
use crate::i18n::{t, tf};
use crate::settings::Settings;
use crate::solver::DealCheck;
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

/// How a game is taken over by [`BoardScreen::begin`]: its history id, the playing time so far and
/// its undo mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Start {
    pub id: Option<GameId>,
    pub elapsed: Duration,
    pub undo_allowed: bool,
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
    copy_note: Option<(u64, bool)>,
    undo_allowed: bool,
    deal: DealCheck,
    guide_panel: bool,
    cues: Vec<SoundCue>,
    dirty: bool,
    screen: Rect,
    covered: bool,
    pointer: Pointer,
}

/// The request to show the guide panel when it is hidden and to hide it when it is shown.
fn toggle_guide(ctx: &ViewCtx<'_>) -> Effect {
    Effect::Apply(Settings {
        guide_panel: !ctx.settings.guide_panel,
        ..ctx.settings.clone()
    })
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
            copy_note: None,
            undo_allowed: true,
            deal: DealCheck::Unchecked,
            guide_panel: true,
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
    /// `undo_allowed` is the undo mode of the game: `false` rejects undo and redo.
    pub fn begin(&mut self, session: Session, start: Start, now: Instant) {
        let Start {
            id,
            elapsed,
            undo_allowed,
        } = start;
        self.cursor = Cursor::initial(&session.game().piles());
        self.session = Some(session);
        self.game_id = id;
        self.clock = GameClock::stopped(elapsed);
        self.clock.resume(now);
        self.cues.clear();
        self.pointer = Pointer::default();
        self.copy_note = None;
        self.undo_allowed = undo_allowed;
        self.deal = DealCheck::Unchecked;
        self.settle();
    }

    /// Tells the header what is known about the running deal.
    pub fn set_deal(&mut self, deal: DealCheck) {
        self.deal = deal;
    }

    /// What the header says about the running deal.
    pub fn deal(&self) -> DealCheck {
        self.deal
    }

    /// Whether undo and redo are allowed in the running game.
    pub fn undo_allowed(&self) -> bool {
        self.undo_allowed
    }

    /// Records how copying the seed ended, for the status line, until the next action.
    pub fn announce_copy(&mut self, seed: u64, copied: bool) {
        self.copy_note = Some((seed, copied));
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

    fn header(&self, now: Instant) -> Option<Header> {
        let session = self.session.as_ref()?;
        Some(Header::new(
            session,
            self.elapsed(now),
            self.undo_allowed,
            self.deal,
        ))
    }

    fn status_text(&self) -> String {
        let focus = self.focus_text();
        let event = match self.copy_note {
            Some((seed, true)) => Some(tf(
                "board.status.seed_copied",
                &[("seed", &seed.to_string())],
            )),
            Some((seed, false)) => Some(tf(
                "board.status.seed_copy_failed",
                &[("seed", &seed.to_string())],
            )),
            None => self.status.map(|key| t(key).into_owned()),
        };
        announce::announce(&focus, event.as_deref().unwrap_or_default())
    }

    /// Handles a key press mapped to `action`; the effect is for the application to apply.
    pub fn handle(&mut self, _key: &KeyEvent, action: Option<Action>, ctx: &ViewCtx<'_>) -> Effect {
        self.guide_panel = ctx.settings.guide_panel;
        match action {
            Some(Action::Guide) => toggle_guide(ctx),
            Some(action) => self.dispatch(action),
            None => Effect::None,
        }
    }

    /// Draws the board screen.
    pub fn draw(&mut self, frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>) {
        self.screen = area;
        self.guide_panel = ctx.settings.guide_panel;
        view::draw(frame, area, self, ctx);
    }
}
