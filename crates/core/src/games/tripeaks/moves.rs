//! Move generation and the hint of TriPeaks.

use super::{ROW_LENS, ROWS, TriPeaks, children_of, ranks_adjacent};
use crate::games::index_ref;
use crate::moves::Move;
use crate::pile::PileRef;

pub(super) fn positions() -> impl Iterator<Item = PileRef> {
    (0..ROWS).flat_map(|row| {
        (0..ROW_LENS[row]).map(move |index| PileRef::Pyramid {
            row: index_ref(row),
            index: index_ref(index),
        })
    })
}

pub(super) fn play(from: PileRef) -> Move {
    Move::Transfer {
        from,
        to: PileRef::Waste,
        count: 1,
    }
}

impl TriPeaks {
    /// Every move [`TriPeaks::execute`] accepts: the stock turn, then each playable peak card.
    pub(super) fn enumerate_moves(&self) -> Vec<Move> {
        let stock = (!self.stock.is_empty()).then_some(Move::Stock);
        let plays = positions().filter(|&from| self.can_play(from)).map(play);
        stock.into_iter().chain(plays).collect()
    }

    fn hint_score(&self, from: PileRef) -> (usize, usize) {
        let PileRef::Pyramid { row, index } = from else {
            return (0, 0);
        };
        let (r, i) = (usize::from(row), usize::from(index));
        let Some(played) = self.slot(r, i).map(|s| s.card.rank.index()) else {
            return (0, 0);
        };
        let uncovers = self.uncovers_parent_count(r, i);
        let next_plays = positions()
            .filter(|&other| other != from && self.can_follow_with(other, played))
            .count();
        (uncovers, next_plays)
    }

    fn uncovers_parent_count(&self, row: usize, index: usize) -> usize {
        let Some(parent_row) = row.checked_sub(1) else {
            return 0;
        };
        (0..ROW_LENS[parent_row])
            .filter(|&parent_idx| {
                self.slot(parent_row, parent_idx).is_some()
                    && children_of(parent_row, parent_idx).is_some_and(|[a, b]| {
                        (a == index && self.slot(row, b).is_none())
                            || (b == index && self.slot(row, a).is_none())
                    })
            })
            .count()
    }

    fn can_follow_with(&self, pile: PileRef, waste_rank: u8) -> bool {
        let PileRef::Pyramid { row, index } = pile else {
            return false;
        };
        let (r, i) = (usize::from(row), usize::from(index));
        self.slot(r, i).is_some_and(|slot| {
            slot.face_up
                && !self.covered(r, i)
                && ranks_adjacent(slot.card.rank.index(), waste_rank)
        })
    }

    /// Prefers a play that uncovers a face-down card or opens follow-up plays; falls back to the
    /// stock turn when no peak card fits.
    pub(super) fn suggest(&self) -> Option<Move> {
        let best = positions().filter(|&from| self.can_play(from)).fold(
            None::<((usize, usize), PileRef)>,
            |best, from| {
                let score = self.hint_score(from);
                match best {
                    Some((top, _)) if top >= score => best,
                    _ => Some((score, from)),
                }
            },
        );
        best.map(|(_, from)| play(from))
            .or_else(|| (!self.stock.is_empty()).then_some(Move::Stock))
    }
}
