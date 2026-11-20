//! Move generation and the hint of Yukon.

use super::{COLUMNS, FOUNDATIONS, Yukon, index_ref};
use crate::games::transfer_parts;
use crate::moves::Move;
use crate::pile::PileRef;

type HintRule = fn(&Yukon, PileRef, PileRef, u8) -> bool;

/// Hint priorities, best first.
const HINT_RULES: [HintRule; 3] = [
    Yukon::to_foundation,
    Yukon::reveals_card,
    Yukon::improves_column,
];

fn columns() -> impl Iterator<Item = PileRef> {
    (0..COLUMNS).map(|i| PileRef::Tableau(index_ref(i)))
}

fn foundations() -> impl Iterator<Item = PileRef> {
    (0..FOUNDATIONS).map(|i| PileRef::Foundation(index_ref(i)))
}

impl Yukon {
    /// Every move [`Yukon::execute`] accepts: each valid transfer of every count from every
    /// column and foundation onto every foundation and column.
    pub(super) fn enumerate_moves(&self) -> Vec<Move> {
        columns()
            .chain(foundations())
            .flat_map(|from| self.transfers_from(from))
            .collect()
    }

    fn transfers_from(&self, from: PileRef) -> Vec<Move> {
        let available = self.movable(from).map_or(0, <[_]>::len);
        let mut moves = Vec::new();
        for count in 1..=u8::try_from(available).unwrap_or(u8::MAX) {
            let accepted = foundations()
                .chain(columns())
                .filter(|&to| self.check_transfer(from, to, count).is_ok());
            moves.extend(accepted.map(|to| Move::Transfer { from, to, count }));
        }
        moves
    }

    /// The best legal move: a card to a foundation, then a group that uncovers a face-down card,
    /// then a group that leaves a card it did not fit on, or empties a column. Never a move out of
    /// a foundation and never a group that already rests on a card it fits, so following the hints
    /// cannot move a group back and forth. `None` when no such move exists.
    pub(super) fn suggest(&self) -> Option<Move> {
        let moves = self.enumerate_moves();
        HINT_RULES.into_iter().find_map(|rule| {
            moves.iter().copied().find(|&mv| {
                transfer_parts(mv).is_some_and(|(from, to, count)| rule(self, from, to, count))
            })
        })
    }

    fn to_foundation(&self, from: PileRef, to: PileRef, _count: u8) -> bool {
        matches!(from, PileRef::Tableau(_)) && matches!(to, PileRef::Foundation(_))
    }

    fn moves_all_face_up(&self, from: PileRef, count: u8) -> bool {
        self.column(from)
            .is_some_and(|column| column.up.len() == usize::from(count))
    }

    fn reveals_card(&self, from: PileRef, to: PileRef, count: u8) -> bool {
        let hides_card = self
            .column(from)
            .is_some_and(|column| !column.down.is_empty());
        self.column(to).is_some() && hides_card && self.moves_all_face_up(from, count)
    }

    /// Whether the group of the top `count` cards of `from` lies on a face-up card it fits.
    fn rests_on_fit(&self, from: PileRef, count: u8) -> bool {
        let Some(column) = self.column(from) else {
            return false;
        };
        let at = column.up.len().saturating_sub(usize::from(count));
        let below = at.checked_sub(1).and_then(|index| column.up.get(index));
        let bottom = column.up.get(at);
        below
            .zip(bottom)
            .is_some_and(|(&below, &bottom)| self.rules.build().fits(below, bottom))
    }

    fn is_whole_column(&self, from: PileRef, count: u8) -> bool {
        let no_face_down = self
            .column(from)
            .is_some_and(|column| column.down.is_empty());
        no_face_down && self.moves_all_face_up(from, count)
    }

    fn improves_column(&self, from: PileRef, to: PileRef, count: u8) -> bool {
        let Some(target) = self.column(to) else {
            return false;
        };
        let pointless = target.up.is_empty() && self.is_whole_column(from, count);
        self.column(from).is_some() && !pointless && !self.rests_on_fit(from, count)
    }
}
