//! Move generation and the hint of FreeCell.

use super::{CELLS, COLUMNS, FOUNDATIONS, FreeCell, index_ref};
use crate::card::Card;
use crate::games::transfer_parts;
use crate::moves::Move;
use crate::pile::PileRef;
use crate::rules;

type HintRule = fn(&FreeCell, PileRef, PileRef, u8) -> bool;

/// Hint priorities, best first. Only transfers are ranked; there is no stock to fall back on.
const HINT_RULES: [HintRule; 5] = [
    FreeCell::to_foundation,
    FreeCell::column_extends_run,
    FreeCell::cell_to_column,
    FreeCell::column_to_empty_column,
    FreeCell::column_to_cell,
];

fn sources() -> impl Iterator<Item = PileRef> {
    let cells = (0..CELLS).map(|i| PileRef::Cell(index_ref(i)));
    let columns = (0..COLUMNS).map(|i| PileRef::Tableau(index_ref(i)));
    cells.chain(columns)
}

fn targets() -> impl Iterator<Item = PileRef> {
    let foundations = (0..FOUNDATIONS).map(|i| PileRef::Foundation(index_ref(i)));
    let cells = (0..CELLS).map(|i| PileRef::Cell(index_ref(i)));
    let columns = (0..COLUMNS).map(|i| PileRef::Tableau(index_ref(i)));
    foundations.chain(cells).chain(columns)
}

impl FreeCell {
    /// Every move [`FreeCell::execute`] accepts: each valid transfer of every count from a cell or
    /// column onto every foundation, cell and column.
    pub(super) fn enumerate_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        for from in sources() {
            moves.extend(self.transfers_from(from));
        }
        moves
    }

    fn transfers_from(&self, from: PileRef) -> Vec<Move> {
        let available = self.movable(from).map_or(0, <[Card]>::len);
        let mut moves = Vec::new();
        for count in 1..=u8::try_from(available).unwrap_or(u8::MAX) {
            let accepted = targets().filter(|&to| self.check_transfer(from, to, count).is_ok());
            moves.extend(accepted.map(|to| Move::Transfer { from, to, count }));
        }
        moves
    }

    /// The best legal move: a card to a foundation, a column move onto a non-empty column, a cell
    /// to a column, a column to an empty column, then a column to a cell. Never a whole column
    /// (a lone card included) to an empty column: that only swaps two columns.
    pub(super) fn suggest(&self) -> Option<Move> {
        let moves = self.enumerate_moves();
        HINT_RULES.into_iter().find_map(|rule| {
            moves.iter().copied().find(|&mv| {
                transfer_parts(mv).is_some_and(|(from, to, count)| rule(self, from, to, count))
            })
        })
    }

    fn column(&self, pile: PileRef) -> Option<&[Card]> {
        match pile {
            PileRef::Tableau(i) => self.columns.get(usize::from(i)).map(Vec::as_slice),
            _ => None,
        }
    }

    fn column_is_empty(&self, pile: PileRef) -> bool {
        self.column(pile).is_some_and(<[Card]>::is_empty)
    }

    fn moves_whole_column(&self, pile: PileRef, count: u8) -> bool {
        self.column(pile)
            .is_some_and(|cards| cards.len() == usize::from(count))
    }

    fn to_foundation(&self, from: PileRef, to: PileRef, _count: u8) -> bool {
        matches!(from, PileRef::Cell(_) | PileRef::Tableau(_))
            && matches!(to, PileRef::Foundation(_))
    }

    /// Whether the first of the `count` moved cards already rests on the card it builds on, so
    /// moving it only shuffles a finished link (and invites moving it straight back).
    fn rests_on_link(&self, from: PileRef, count: u8) -> bool {
        self.column(from).is_some_and(|cards| {
            let start = cards.len().saturating_sub(usize::from(count));
            start
                .checked_sub(1)
                .and_then(|under| cards.get(under).zip(cards.get(start)))
                .is_some_and(|(&under, &first)| rules::fits_alternating(under, first))
        })
    }

    fn column_extends_run(&self, from: PileRef, to: PileRef, count: u8) -> bool {
        matches!(from, PileRef::Tableau(_))
            && matches!(to, PileRef::Tableau(_))
            && !self.column_is_empty(to)
            && !self.rests_on_link(from, count)
    }

    fn cell_to_column(&self, from: PileRef, to: PileRef, _count: u8) -> bool {
        matches!(from, PileRef::Cell(_)) && matches!(to, PileRef::Tableau(_))
    }

    fn column_to_empty_column(&self, from: PileRef, to: PileRef, count: u8) -> bool {
        matches!(from, PileRef::Tableau(_))
            && self.column_is_empty(to)
            && !self.moves_whole_column(from, count)
            && !self.rests_on_link(from, count)
    }

    fn column_to_cell(&self, from: PileRef, to: PileRef, count: u8) -> bool {
        matches!(from, PileRef::Tableau(_))
            && matches!(to, PileRef::Cell(_))
            && !self.rests_on_link(from, count)
    }
}
