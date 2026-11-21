//! Finishing a game the engine has proven won (`Session::forced_finish`). With auto-finish on the
//! moves are played one per clock step; any key or mouse press interrupts. With it off the finish
//! key plays them at once. Every move goes through `play`, so the history and the statistics see it.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use oxysol_core::Move;

use super::BoardScreen;
use super::guidance::{self, Part};
use super::status;
use crate::action::Action;
use crate::audio::SoundCue;
use crate::screens::Effect;
use crate::storage::counters::MoveSource;

/// The time between two finishing moves: slow enough to follow, fast enough not to drag.
pub(super) const STEP: Duration = Duration::from_millis(400);

/// The moves still to play and the instant the next one is due.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Finishing {
    moves: VecDeque<Move>,
    due: Instant,
}

impl BoardScreen {
    /// Runs the auto-finish clock at `now`: starts a finish the engine offers and plays the next
    /// move when it is due. Called by the application after every event, but not while a modal is
    /// open.
    pub fn advance(&mut self, now: Instant, auto_finish: bool) -> Effect {
        if self.finishing.is_none() && auto_finish && !self.declined {
            self.start_finishing(now);
        }
        let due = self.finishing.as_ref().map(|finishing| finishing.due);
        if due.is_some_and(|due| due <= now) {
            return self.step(now);
        }
        Effect::None
    }

    /// Stops a finish in progress: any key or mouse press comes first and leaves the position to
    /// the player. The finish is not started again until the position changes.
    pub fn interrupt_finish(&mut self) {
        if self.finishing.take().is_some() {
            self.declined = true;
            self.status = None;
        }
    }

    /// Whether a finish is being played out.
    pub fn is_finishing(&self) -> bool {
        self.finishing.is_some()
    }

    /// The finish key. With auto-finish on it starts the paced finish again; with it off the
    /// moves are played at once and the win dialog follows the last one.
    pub(super) fn finish_key(&mut self) -> Effect {
        let Some(moves) = self.forced.clone() else {
            self.status = Some(status::NO_FINISH);
            self.cue(SoundCue::Illegal);
            return Effect::None;
        };
        if self.auto_finish {
            self.declined = false;
            return Effect::None;
        }
        self.play_all(moves)
    }

    /// Offers the finish in the action line when the player has to ask for it: a forced win that
    /// is not played out by itself.
    pub(super) fn push_finish(&self, possible: &mut Vec<Part>, usable: &mut Vec<Action>) {
        let offered = self.forced.is_some() && (!self.auto_finish || self.declined);
        if offered {
            possible.push(guidance::part(
                Action::Finish,
                "board.action.finish",
                super::priority::FINISH,
            ));
            usable.push(Action::Finish);
        }
    }

    fn start_finishing(&mut self, now: Instant) {
        let Some(moves) = self.forced.clone() else {
            return;
        };
        self.finishing = Some(Finishing {
            moves: moves.into(),
            due: now + STEP,
        });
        self.status = Some(status::FINISHING);
    }

    fn step(&mut self, now: Instant) -> Effect {
        let next = self
            .finishing
            .as_mut()
            .and_then(|finishing| finishing.moves.pop_front());
        let Some(mv) = next else {
            self.finishing = None;
            return Effect::None;
        };
        let before = self.moves_made();
        let effect = self.play_from(mv, MoveSource::Finish);
        if self.moves_made() == before {
            self.finishing = None;
            self.declined = true;
            return Effect::None;
        }
        if self.is_won() {
            self.finishing = None;
            return effect;
        }
        if let Some(finishing) = self.finishing.as_mut() {
            finishing.due = now + STEP;
        }
        self.status = Some(status::FINISHING);
        effect
    }

    fn play_all(&mut self, moves: Vec<Move>) -> Effect {
        let mut effect = Effect::None;
        for mv in moves {
            let before = self.moves_made();
            effect = self.play_from(mv, MoveSource::Finish);
            if self.moves_made() == before {
                break;
            }
        }
        effect
    }

    pub(super) fn moves_made(&self) -> usize {
        self.session
            .as_ref()
            .map_or(0, |session| session.moves_made())
    }
}
