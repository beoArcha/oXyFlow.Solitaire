//! Localized messages of the board status line, and the description of the focus the line starts
//! with.

use oxysol_core::{Move, MoveError, PileRef, PileSnapshot};

use super::BoardScreen;
use crate::action::Action;
use crate::announce::{self, Focus, Held, Look};
use crate::i18n::{t, tf};
use crate::input::primary_key;

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

pub(super) const PICKED: &str = "board.status.picked";
pub(super) const NO_TARGETS: &str = "board.status.no_targets";
pub(super) const NOTHING_TO_UNDO: &str = "board.status.nothing_to_undo";
pub(super) const NOTHING_TO_REDO: &str = "board.status.nothing_to_redo";
pub(super) const NO_HINT: &str = "board.status.no_hint";
pub(super) const HINT: &str = "board.status.hint";
pub(super) const HINT_STOCK: &str = "board.status.hint_stock";
pub(super) const NO_MOVES: &str = "board.status.no_moves";
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
fn look_at(snapshot: &PileSnapshot, depth: usize) -> Look {
    let len = snapshot.cards.len();
    match snapshot.cards.get(len.saturating_sub(depth)) {
        Some(table) if table.face_up => Look::Up(table.card),
        Some(_) => Look::FaceDown,
        None => Look::Empty,
    }
}

/// One item of the first-steps hint; the part with the highest `drop_order` goes first when the
/// line is too wide.
struct HintPart {
    text: String,
    drop_order: u8,
}

fn join(parts: &[HintPart]) -> String {
    parts
        .iter()
        .map(|part| part.text.as_str())
        .collect::<Vec<_>>()
        .join(&t("board.first_steps.separator"))
}

impl BoardScreen {
    /// The description of the focus that starts the status line. Empty without a game and while
    /// the board is covered, so a paused game shows no card.
    pub(super) fn focus_text(&self) -> String {
        let Some(session) = self.session.as_ref().filter(|_| !self.covered) else {
            return String::new();
        };
        let piles = session.game().piles();
        let look = piles
            .iter()
            .find(|snapshot| snapshot.pile == self.cursor.pile)
            .map_or(Look::Empty, |snapshot| look_at(snapshot, self.cursor.depth));
        let held = self.pick.as_ref().map(|pick| Held {
            source: pick.source,
            count: pick.count,
            targets: pick
                .choices
                .iter()
                .filter(|choice| choice.target.is_some())
                .count(),
        });
        announce::focus(&Focus {
            pile: self.cursor.pile,
            look,
            held,
        })
    }

    /// The parts of the hint that starts a new game, in display order: how to move, pick up and
    /// drop, and where to find the stock, the key list and the rules. Empty once the player acted,
    /// for a resumed game with moves, and while covered. The keys come from the key table; the
    /// stock is mentioned only when the game has one.
    fn first_steps_parts(&self) -> Vec<HintPart> {
        let Some(session) = self
            .session
            .as_ref()
            .filter(|_| self.first_steps && !self.covered)
        else {
            return Vec::new();
        };
        let has_stock = session
            .game()
            .piles()
            .iter()
            .any(|snapshot| snapshot.pile == PileRef::Stock);
        let keyed = [
            (Action::Select, "board.first_steps.select", 1, true),
            (Action::Stock, "board.first_steps.stock", 4, has_stock),
            (Action::Help, "board.first_steps.help", 3, true),
            (Action::Rules, "board.first_steps.rules", 0, true),
        ];
        let mut parts = vec![HintPart {
            text: t("board.first_steps.arrows").into_owned(),
            drop_order: 2,
        }];
        parts.extend(keyed.into_iter().filter(|(.., shown)| *shown).map(
            |(action, key, drop_order, _)| HintPart {
                text: tf(key, &[("key", &primary_key(action).unwrap_or_default())]),
                drop_order,
            },
        ));
        parts
    }

    /// The first-steps hint cut down to fit a status line of `width` columns after `focus`: parts
    /// are dropped, least useful first, until the whole line fits. Empty when none is due.
    pub(super) fn fitted_first_steps(&self, focus: &str, width: usize) -> String {
        let mut parts = self.first_steps_parts();
        while !parts.is_empty() {
            let hint = join(&parts);
            if announce::announce(focus, &hint).chars().count() <= width {
                return hint;
            }
            let least = parts
                .iter()
                .enumerate()
                .max_by_key(|(_, part)| part.drop_order)
                .map(|(index, _)| index);
            if let Some(index) = least {
                parts.remove(index);
            }
        }
        String::new()
    }

    /// Records the event of the last action, unless the board already says why it stopped.
    pub(super) fn announce_event(&mut self, key: &'static str) {
        if self.status.is_none() {
            self.status = Some(key);
        }
    }
}
