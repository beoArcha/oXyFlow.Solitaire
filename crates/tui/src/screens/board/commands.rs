//! The commands of the board: what each [`Action`] does to the controller.

use oxysol_core::{Move, PileSnapshot};

use super::cursor::{Cursor, Dir, closest, cycle, focusable, max_depth, nearest};
use super::{BoardScreen, status};
use crate::action::Action;
use crate::app::{Modal, Screen};
use crate::audio::SoundCue;
use crate::layout::{self, BoardGeometry};
use crate::screens::Effect;

impl BoardScreen {
    pub(super) fn dispatch(&mut self, action: Action) -> Effect {
        if self.session.is_none() {
            return match action {
                Action::Back | Action::Menu => Effect::Goto(Screen::MainMenu),
                _ => Effect::None,
            };
        }
        self.copy_note = None;
        self.status = None;
        self.hint_note = false;
        match action {
            Action::Up | Action::Down | Action::Left | Action::Right => {
                self.navigate(action);
                Effect::None
            }
            Action::FocusNext | Action::FocusPrevious => {
                self.cycle_pile(action == Action::FocusNext);
                Effect::None
            }
            Action::RunMore | Action::RunLess => {
                self.adjust_depth(action == Action::RunMore);
                Effect::None
            }
            Action::Select => self.select(),
            Action::Back => self.back(),
            Action::Menu => Effect::Goto(Screen::MainMenu),
            Action::Stock => self.play(Move::Stock),
            Action::QuickMove => self.quick_move(),
            Action::Undo => self.undo(),
            Action::Redo => self.redo(),
            Action::Hint => self.hint(),
            Action::Finish => self.finish_key(),
            Action::Pause => Effect::Open(Modal::Pause),
            Action::Restart => {
                Effect::Open(Modal::confirm("board.confirm.restart", Effect::Restart))
            }
            Action::NewGame => self.confirm_new_game(),
            Action::CopySeed => self
                .session
                .as_ref()
                .map_or(Effect::None, |session| Effect::CopySeed(session.seed())),
            Action::Quit => Effect::Open(Modal::confirm("board.confirm.quit", Effect::Quit)),
            Action::ForceQuit | Action::Help | Action::Rules | Action::Guide => Effect::None,
        }
    }

    pub(super) fn piles(&self) -> Vec<PileSnapshot> {
        self.session
            .as_ref()
            .map(|session| session.game().piles())
            .unwrap_or_default()
    }

    pub(super) fn geometry(&self, piles: &[PileSnapshot]) -> Option<BoardGeometry> {
        let session = self.session.as_ref()?;
        Some(layout::compute(
            session.variant(),
            piles,
            layout::chrome_with(self.screen, self.guide_panel).body,
        ))
    }

    pub(super) fn cue(&mut self, cue: SoundCue) {
        self.cues.push(cue);
    }

    /// Whether the running game is solved.
    pub fn is_won(&self) -> bool {
        self.session
            .as_ref()
            .is_some_and(|session| session.game().is_won())
    }

    fn navigate(&mut self, action: Action) {
        let Some(dir) = Dir::from_action(action) else {
            return;
        };
        let piles = self.piles();
        let Some(current) = piles.iter().find(|s| s.pile == self.cursor.pile) else {
            return;
        };
        let inside = self.pick.is_none()
            && match dir {
                Dir::Up => self.cursor.deeper(current),
                Dir::Down => self.cursor.shallower(),
                Dir::Left | Dir::Right => false,
            };
        if inside {
            return;
        }
        let target = self
            .geometry(&piles)
            .and_then(|geometry| nearest(&piles, &geometry, self.cursor.pile, dir));
        if let Some(pile) = target {
            self.cursor = Cursor { pile, depth: 1 };
        }
    }

    /// Tab and Shift+Tab: the cursor goes to the next or the previous pile in reading order.
    fn cycle_pile(&mut self, forward: bool) {
        let piles = self.piles();
        if let Some(pile) = cycle(&piles, self.cursor.pile, forward) {
            self.cursor = Cursor { pile, depth: 1 };
        }
    }

    fn adjust_depth(&mut self, deeper: bool) {
        if self.pick.is_some() {
            return;
        }
        let piles = self.piles();
        if let Some(current) = piles.iter().find(|s| s.pile == self.cursor.pile) {
            if deeper {
                self.cursor.deeper(current);
            } else {
                self.cursor.shallower();
            }
        }
    }

    pub(super) fn keep_cursor_valid(&mut self) {
        let piles = self.piles();
        let current = piles.iter().find(|s| s.pile == self.cursor.pile);
        if let Some(snapshot) = current.filter(|s| focusable(s)) {
            self.cursor.depth = self.cursor.depth.min(max_depth(snapshot));
            return;
        }
        let moved = self
            .geometry(&piles)
            .and_then(|geometry| closest(&piles, &geometry, self.cursor.pile));
        if let Some(pile) = moved {
            self.cursor = Cursor { pile, depth: 1 };
        }
    }

    fn back(&mut self) -> Effect {
        if self.pick.take().is_some() {
            self.status = None;
            return Effect::None;
        }
        Effect::Goto(Screen::MainMenu)
    }

    fn undo_refused(&mut self) -> Effect {
        self.status = Some(status::UNDO_OFF);
        self.cue(SoundCue::Illegal);
        Effect::None
    }

    fn undo(&mut self) -> Effect {
        if !self.undo_allowed {
            return self.undo_refused();
        }
        let done = self
            .session
            .as_mut()
            .is_some_and(|session| session.can_undo() && session.undo());
        self.after_history(done, status::NOTHING_TO_UNDO, status::UNDONE)
    }

    fn redo(&mut self) -> Effect {
        if !self.undo_allowed {
            return self.undo_refused();
        }
        let done = self
            .session
            .as_mut()
            .is_some_and(|session| session.can_redo() && session.redo());
        self.after_history(done, status::NOTHING_TO_REDO, status::REDONE)
    }

    fn after_history(
        &mut self,
        done: bool,
        refusal: &'static str,
        success: &'static str,
    ) -> Effect {
        if !done {
            self.status = Some(refusal);
            self.cue(SoundCue::Illegal);
            return Effect::None;
        }
        self.cue(SoundCue::Undo);
        let effect = self.settle();
        self.announce_event(success);
        effect
    }

    fn hint(&mut self) -> Effect {
        let hint = self.session.as_ref().and_then(|s| s.game().hint());
        let Some(mv) = hint else {
            self.status = Some(status::NO_HINT);
            self.cue(SoundCue::Illegal);
            return Effect::None;
        };
        if self.hint != Some(mv) {
            self.hint = Some(mv);
            self.cue(SoundCue::Hint);
        }
        self.status = None;
        self.hint_note = true;
        Effect::None
    }

    fn confirm_new_game(&self) -> Effect {
        let Some(session) = &self.session else {
            return Effect::None;
        };
        Effect::Open(Modal::confirm(
            "board.confirm.new_game",
            Effect::Start {
                variant: session.variant(),
                seed: None,
                undo_allowed: self.undo_allowed,
            },
        ))
    }
}
