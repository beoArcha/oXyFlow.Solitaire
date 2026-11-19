//! The board screen: the running game, its clock and the controller — the cursor, picking and
//! dropping, and the commands. The drawing is in `view.rs`; every rule comes from the engine
//! through `Session`, the screen only turns keys into moves.

mod choices;
mod commands;
mod controls;
mod counters;
mod cursor;
mod finish;
mod guidance;
mod guide_view;
mod header;
mod marks;
mod mouse;
mod moves;
mod moves_panel;
mod now;
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
use self::finish::Finishing;
use self::header::Facts;
use self::mouse::Pointer;
use super::{Effect, ViewCtx};
use crate::action::Action;
use crate::app::Modal;
use crate::audio::SoundCue;
use crate::clock::GameClock;
use crate::settings::Aid;
use crate::solver::DealCheck;
use crate::storage::GameId;
use crate::widgets::footer::FooterMap;
use crate::widgets::moves::PanelMap;
use crate::widgets::overlay::QuickOptions;
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
    hint_note: bool,
    undo_allowed: bool,
    restarts: u32,
    deal: DealCheck,
    guide_panel: bool,
    moves_panel: bool,
    target_marks: bool,
    panel_focus: Option<usize>,
    field_edge: bool,
    auto_finish: bool,
    forced: Option<Vec<Move>>,
    finishing: Option<Finishing>,
    declined: bool,
    cues: Vec<SoundCue>,
    dirty: bool,
    screen: Rect,
    covered: bool,
    pointer: Pointer,
    /// The buttons of the `Do:` row as drawn last.
    do_map: FooterMap,
    /// The active entries of the possible moves panel as drawn last.
    moves_map: PanelMap,
}

/// The request to show the guide panel when it is hidden and to hide it when it is shown. It is a
/// quick choice of the running game; "Save as default" in the quick options keeps it.
fn toggle_guide(ctx: &ViewCtx<'_>) -> Effect {
    Effect::Quick(Aid::GuidePanel.toggled(&ctx.settings.aids()))
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
            hint_note: false,
            undo_allowed: true,
            restarts: 0,
            deal: DealCheck::Unchecked,
            guide_panel: true,
            moves_panel: true,
            target_marks: true,
            panel_focus: None,
            field_edge: false,
            auto_finish: true,
            forced: None,
            finishing: None,
            declined: false,
            cues: Vec::new(),
            dirty: false,
            screen: DEFAULT_SCREEN,
            covered: false,
            pointer: Pointer::default(),
            do_map: FooterMap::default(),
            moves_map: PanelMap::default(),
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
        self.finishing = None;
        self.cues.clear();
        self.pointer = Pointer::default();
        self.copy_note = None;
        self.undo_allowed = undo_allowed;
        self.restarts = 0;
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

    /// Restarts counted in the running game: only a game without undo counts them.
    pub fn restarts(&self) -> u32 {
        self.restarts
    }

    /// Sets the restart count of a game taken over from the saved one.
    pub fn set_restarts(&mut self, restarts: u32) {
        self.restarts = restarts;
    }

    /// Records how copying the seed ended, for the status line, until the next action.
    pub fn announce_copy(&mut self, seed: u64, copied: bool) {
        self.copy_note = Some((seed, copied));
    }

    /// Back to the dealt position of the same seed with an empty history and a zeroed clock.
    /// Returns whether the restart was counted, which it is in a game without undo only.
    pub fn restart(&mut self, now: Instant) -> bool {
        let Some(session) = self.session.as_mut() else {
            return false;
        };
        session.restart();
        self.cursor = Cursor::initial(&session.game().piles());
        self.pointer = Pointer::default();
        self.finishing = None;
        self.clock = GameClock::stopped(Duration::ZERO);
        self.clock.resume(now);
        self.settle();
        self.announce_event(status::RESTARTED);
        let counted = !self.undo_allowed;
        if counted {
            self.restarts = self.restarts.saturating_add(1);
        }
        counted
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
        let mut marks = marks::build(
            &session.game().piles(),
            &self.cursor,
            self.pick.as_ref(),
            self.hint,
        );
        if !self.target_marks {
            marks.targets.clear();
        }
        marks
    }

    fn facts(&self, now: Instant) -> Option<Facts> {
        let session = self.session.as_ref()?;
        Some(Facts::new(
            session,
            self.elapsed(now),
            self.undo_allowed,
            self.restarts,
            self.deal,
        ))
    }

    /// Takes over the aids of the running game, which the application hands over as the settings,
    /// and the frame of the theme. They decide the screen split, so that drawing, the mouse and the
    /// commands all use the same [`layout::Chrome`].
    pub(super) fn follow(&mut self, ctx: &ViewCtx<'_>) {
        self.guide_panel = ctx.settings.guide_panel;
        self.moves_panel = ctx.settings.moves_panel;
        self.target_marks = ctx.settings.target_marks;
        self.field_edge = ctx.theme.field_edge();
        self.auto_finish = ctx.settings.auto_finish;
    }

    /// Handles a key press mapped to `action`; the effect is for the application to apply.
    pub fn handle(&mut self, _key: &KeyEvent, action: Option<Action>, ctx: &ViewCtx<'_>) -> Effect {
        self.follow(ctx);
        match action {
            Some(Action::Guide) if self.panel_unavailable() => self.open_guide_window(ctx),
            Some(Action::Guide) => toggle_guide(ctx),
            Some(Action::Options) => {
                Effect::Open(Modal::Quick(QuickOptions::new(ctx.settings.aids())))
            }
            Some(Action::Moves) if self.panel_unavailable() => self.open_moves_window(),
            Some(Action::Moves) => moves_panel::toggle_moves(ctx),
            Some(action) => self
                .panel_key(action)
                .unwrap_or_else(|| self.dispatch(action)),
            None => Effect::None,
        }
    }

    /// Draws the board screen.
    pub fn draw(&mut self, frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>) {
        self.screen = area;
        self.follow(ctx);
        let (do_map, moves_map) = view::draw(frame, self, ctx);
        self.do_map = do_map;
        self.moves_map = moves_map;
    }

    /// The action of the `Do:` button under `cell`, if any.
    pub fn footer_action_at(&self, cell: (u16, u16)) -> Option<Action> {
        self.do_map.action_at(cell)
    }
}
