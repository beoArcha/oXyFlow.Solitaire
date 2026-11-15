//! Picking, dropping and playing moves: the part of the controller that talks to the engine.

use oxysol_core::{Move, MoveError, PileRef, PileSnapshot, Variant};

use super::choices::{Choice, choices, quick};
use super::status::{self, error_key};
use super::{BoardScreen, Pick};
use crate::app::Modal;
use crate::audio::SoundCue;
use crate::screens::Effect;

/// Why the cards under the cursor cannot be picked up.
enum Refusal {
    Error(MoveError),
    NoTargets,
    Probe(Move),
}

impl BoardScreen {
    pub(super) fn select(&mut self) -> Effect {
        match self.pick.clone() {
            Some(pick) => self.drop_on(&pick, self.cursor.pile),
            None if self.cursor.pile == PileRef::Stock => self.play(Move::Stock),
            None => self.pick_up(),
        }
    }

    pub(super) fn quick_move(&mut self) -> Effect {
        let found = match &self.pick {
            Some(pick) => Ok(pick.choices.clone()),
            None => self
                .candidates(self.cursor.pile, self.cursor.depth)
                .map(|(_, found)| found),
        };
        match found.map(|found| quick(&found)) {
            Ok(Some(choice)) => self.play(choice.mv),
            Ok(None) => self.refuse(Refusal::NoTargets),
            Err(refusal) => self.refuse(refusal),
        }
    }

    fn pick_up(&mut self) -> Effect {
        match self.candidates(self.cursor.pile, self.cursor.depth) {
            Ok((count, found)) => self.start_pick(self.cursor.pile, count, found),
            Err(refusal) => self.refuse(refusal),
        }
    }

    fn start_pick(&mut self, source: PileRef, count: u8, found: Vec<Choice>) -> Effect {
        if let Some(immediate) = found.iter().find(|choice| choice.target.is_none()) {
            return self.play(immediate.mv);
        }
        self.pick = Some(Pick {
            source,
            count,
            choices: found,
        });
        self.status = Some(status::PICKED);
        self.cue(SoundCue::CardPick);
        Effect::None
    }

    fn drop_on(&mut self, pick: &Pick, target: PileRef) -> Effect {
        if target == pick.source {
            self.pick = None;
            self.status = None;
            return Effect::None;
        }
        let known = pick
            .choices
            .iter()
            .find(|choice| choice.target == Some(target));
        let variant = self.session.as_ref().map(|session| session.variant());
        match (known, variant) {
            (Some(choice), _) => self.play(choice.mv),
            (None, Some(variant)) => self.play(attempt(variant, pick, target)),
            (None, None) => Effect::None,
        }
    }

    fn candidates(&self, pile: PileRef, depth: usize) -> Result<(u8, Vec<Choice>), Refusal> {
        let session = self
            .session
            .as_ref()
            .ok_or(Refusal::Error(MoveError::NoSuchPile(pile)))?;
        let piles = session.game().piles();
        let snapshot = piles
            .iter()
            .find(|snapshot| snapshot.pile == pile)
            .ok_or(Refusal::Error(MoveError::NoSuchPile(pile)))?;
        let len = snapshot.cards.len();
        if len == 0 {
            return Err(Refusal::Error(MoveError::EmptySource(pile)));
        }
        let depth = depth.clamp(1, len);
        let face_up = snapshot
            .cards
            .get(len - depth)
            .is_some_and(|card| card.face_up);
        if !face_up {
            return Err(Refusal::Error(MoveError::FaceDown));
        }
        let count = u8::try_from(depth).unwrap_or(u8::MAX);
        let found = choices(&session.game().legal_moves(), pile, count);
        if !found.is_empty() {
            return Ok((count, found));
        }
        Err(match session.variant() {
            Variant::Pyramid(_) => Refusal::Probe(Move::Remove {
                first: pile,
                second: None,
            }),
            _ => Refusal::NoTargets,
        })
    }

    fn refuse(&mut self, refusal: Refusal) -> Effect {
        match refusal {
            Refusal::Error(error) => self.reject(&error),
            Refusal::NoTargets => {
                self.status = Some(status::NO_TARGETS);
                self.cue(SoundCue::Illegal);
            }
            Refusal::Probe(mv) => return self.play(mv),
        }
        Effect::None
    }

    fn reject(&mut self, error: &MoveError) {
        self.status = Some(if self.is_won() {
            status::SOLVED
        } else {
            error_key(error)
        });
        self.cue(SoundCue::Illegal);
    }

    pub(super) fn play(&mut self, mv: Move) -> Effect {
        let Some(session) = self.session.as_mut() else {
            return Effect::None;
        };
        let before = face_down(&session.game().piles());
        match session.apply(mv) {
            Ok(()) => {
                let flipped = face_down(&session.game().piles()) < before;
                self.after_move(mv, flipped)
            }
            Err(error) => {
                self.reject(&error);
                Effect::None
            }
        }
    }

    fn after_move(&mut self, mv: Move, flipped: bool) -> Effect {
        if mv == Move::Stock {
            self.cue(SoundCue::StockDeal);
        } else {
            self.cue(SoundCue::CardPlace);
            if flipped {
                self.cue(SoundCue::CardFlip);
            }
        }
        let effect = self.settle();
        self.announce_event(status::done_of(mv));
        effect
    }

    /// Runs after every change of the session: clears what the change made stale, keeps the
    /// cursor on a pile that still exists, saves, and reports a win or a dead end.
    pub(super) fn settle(&mut self) -> Effect {
        self.pick = None;
        self.hint = None;
        self.status = None;
        self.cursor.depth = 1;
        self.keep_cursor_valid();
        self.persist();
        if self.is_won() {
            self.cue(SoundCue::Win);
            return Effect::Open(Modal::Win);
        }
        let stuck = self
            .session
            .as_ref()
            .is_some_and(|session| session.game().legal_moves().is_empty());
        if stuck {
            self.status = Some(status::NO_MOVES);
        }
        Effect::None
    }

    /// Called after every change of the session; P4-03 saves the game here.
    fn persist(&mut self) {}
}

fn face_down(piles: &[PileSnapshot]) -> usize {
    piles
        .iter()
        .filter(|snapshot| snapshot.pile != PileRef::Stock)
        .flat_map(|snapshot| &snapshot.cards)
        .filter(|card| !card.face_up)
        .count()
}

fn attempt(variant: Variant, pick: &Pick, target: PileRef) -> Move {
    match variant {
        Variant::Pyramid(_) => Move::Remove {
            first: pick.source,
            second: Some(target),
        },
        _ => Move::Transfer {
            from: pick.source,
            to: target,
            count: pick.count,
        },
    }
}
