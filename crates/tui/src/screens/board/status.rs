//! Localized messages of the last event, shown at the right end of the `Now:` row, and what a
//! position shows.

use oxysol_core::{Move, MoveError, PileSnapshot};

use super::BoardScreen;
use crate::announce::Look;
use crate::i18n::{t, tf};

/// Localization key of the message for a rejected move. Every [`MoveError`] has one; the match is
/// exhaustive on purpose, so a new variant cannot be forgotten.
pub(super) const fn error_key(error: &MoveError) -> &'static str {
    match error {
        MoveError::Unsupported => "board.error.unsupported",
        MoveError::NoSuchPile(_) => "board.error.no_such_pile",
        MoveError::EmptySource(_) => "board.error.empty_source",
        MoveError::FaceDown => "board.error.face_down",
        MoveError::InvalidSequence => "board.error.invalid_sequence",
        MoveError::TargetRejects => "board.error.target_rejects",
        MoveError::StockEmpty => "board.error.stock_empty",
        MoveError::EmptyColumnBlocksDeal => "board.error.empty_column_blocks_deal",
        MoveError::PassesExhausted => "board.error.passes_exhausted",
        MoveError::CountOutOfRange { .. } => "board.error.count_out_of_range",
        MoveError::NotEnoughSpace => "board.error.not_enough_space",
        MoveError::NotAPair => "board.error.not_a_pair",
        MoveError::Blocked => "board.error.blocked",
    }
}

pub(super) const NO_TARGETS: &str = "board.status.no_targets";
pub(super) const NOTHING_TO_UNDO: &str = "board.status.nothing_to_undo";
pub(super) const NOTHING_TO_REDO: &str = "board.status.nothing_to_redo";
pub(super) const NO_HINT: &str = "board.status.no_hint";
pub(super) const UNDO_OFF: &str = "board.status.undo_off";
pub(super) const SOLVED: &str = "board.status.solved";
pub(super) const MOVED: &str = "board.status.moved";
pub(super) const DEALT: &str = "board.status.dealt";
pub(super) const REMOVED: &str = "board.status.removed";
pub(super) const UNDONE: &str = "board.status.undone";
pub(super) const REDONE: &str = "board.status.redone";
pub(super) const RESTARTED: &str = "board.status.restarted";

/// Localization key of the event announced after a move was played.
pub(super) const fn done_of(mv: Move) -> &'static str {
    match mv {
        Move::Stock => DEALT,
        Move::Transfer { .. } => MOVED,
        Move::Remove { .. } => REMOVED,
    }
}

/// What a position shows: nothing, a face-down card, or the card itself. `depth` counts from the
/// top of the pile, as the cursor does.
pub(super) fn look_at(snapshot: &PileSnapshot, depth: usize) -> Look {
    let len = snapshot.cards.len();
    match snapshot.cards.get(len.saturating_sub(depth)) {
        Some(table) if table.face_up => Look::Up(table.card),
        Some(_) => Look::FaceDown,
        None => Look::Empty,
    }
}

/// How an event is drawn: a refusal in the Error token, anything else in the Success token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Tone {
    Success,
    Error,
}

const REFUSALS: [&str; 5] = [
    NO_TARGETS,
    NOTHING_TO_UNDO,
    NOTHING_TO_REDO,
    NO_HINT,
    UNDO_OFF,
];

/// The tone of the event with the localization key `key`.
pub(super) fn tone_of(key: &str) -> Tone {
    if key.starts_with("board.error.") || REFUSALS.contains(&key) {
        Tone::Error
    } else {
        Tone::Success
    }
}

impl BoardScreen {
    /// The last event, if there is one: the text of the copy result or of the status message, with
    /// its tone. Short on purpose, it shares the `Now:` row.
    pub(super) fn event(&self) -> Option<(String, Tone)> {
        match self.copy_note {
            Some((seed, true)) => Some((
                tf("board.status.seed_copied", &[("seed", &seed.to_string())]),
                Tone::Success,
            )),
            Some((_, false)) => {
                Some((t("board.status.seed_copy_failed").into_owned(), Tone::Error))
            }
            None => self.status.map(|key| (t(key).into_owned(), tone_of(key))),
        }
    }

    /// Records the event of the last action, unless the board already says why it stopped.
    pub(super) fn announce_event(&mut self, key: &'static str) {
        if self.status.is_none() {
            self.status = Some(key);
        }
    }
}
