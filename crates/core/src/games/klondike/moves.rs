//! Move generation and the hint of Klondike.

use super::{COLUMNS, FOUNDATIONS, Klondike, index_ref};
use crate::moves::Move;
use crate::pile::PileRef;

type HintRule = fn(&Klondike, PileRef, PileRef, u8) -> bool;

/// Hint priorities, best first. Only transfers are ranked; `Move::Stock` is the fallback.
const HINT_RULES: [HintRule; 4] = [
    Klondike::to_foundation,
    Klondike::run_onto_column,
    Klondike::waste_to_column,
    Klondike::king_reveals_card,
];

fn sources() -> impl Iterator<Item = PileRef> {
    let columns = (0..COLUMNS).map(|i| PileRef::Tableau(index_ref(i)));
    let foundations = (0..FOUNDATIONS).map(|i| PileRef::Foundation(index_ref(i)));
    std::iter::once(PileRef::Waste)
        .chain(columns)
        .chain(foundations)
}

fn targets() -> impl Iterator<Item = PileRef> {
    let foundations = (0..FOUNDATIONS).map(|i| PileRef::Foundation(index_ref(i)));
    let columns = (0..COLUMNS).map(|i| PileRef::Tableau(index_ref(i)));
    foundations.chain(columns)
}

fn transfer_parts(mv: Move) -> Option<(PileRef, PileRef, u8)> {
    match mv {
        Move::Transfer { from, to, count } => Some((from, to, count)),
        Move::Stock | Move::Remove { .. } => None,
    }
}

impl Klondike {
    /// Every move [`Klondike::execute`] accepts: the stock move and each valid transfer of every
    /// count from every source onto every target.
    pub(super) fn enumerate_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        if !self.stock.is_empty() || !self.waste.is_empty() {
            moves.push(Move::Stock);
        }
        for from in sources() {
            moves.extend(self.transfers_from(from));
        }
        moves
    }

    fn transfers_from(&self, from: PileRef) -> Vec<Move> {
        let available = self.movable(from).map_or(0, <[_]>::len);
        let mut moves = Vec::new();
        for count in 1..=u8::try_from(available).unwrap_or(u8::MAX) {
            let accepted = targets().filter(|&to| self.check_transfer(from, to, count).is_ok());
            moves.extend(accepted.map(|to| Move::Transfer { from, to, count }));
        }
        moves
    }

    /// The best legal move: a foundation move, then a whole run onto a column, then the waste onto
    /// a column, then a king that uncovers a face-down card, then the stock. Never a move out of a
    /// foundation, a king from the bottom of a column to an empty column, or part of a run.
    pub(super) fn suggest(&self) -> Option<Move> {
        let moves = self.enumerate_moves();
        for rule in HINT_RULES {
            let ranked = moves.iter().copied().find(|&mv| {
                transfer_parts(mv).is_some_and(|(from, to, count)| rule(self, from, to, count))
            });
            if ranked.is_some() {
                return ranked;
            }
        }
        moves.contains(&Move::Stock).then_some(Move::Stock)
    }

    fn to_foundation(&self, from: PileRef, to: PileRef, _count: u8) -> bool {
        matches!(from, PileRef::Waste | PileRef::Tableau(_)) && matches!(to, PileRef::Foundation(_))
    }

    fn moves_whole_run(&self, from: PileRef, count: u8) -> bool {
        self.column(from)
            .is_some_and(|column| column.up.len() == usize::from(count))
    }

    fn column_is_empty(&self, pile: PileRef) -> bool {
        self.column(pile).is_some_and(|column| column.up.is_empty())
    }

    fn run_onto_column(&self, from: PileRef, to: PileRef, count: u8) -> bool {
        self.column(to).is_some() && !self.column_is_empty(to) && self.moves_whole_run(from, count)
    }

    fn waste_to_column(&self, from: PileRef, to: PileRef, _count: u8) -> bool {
        from == PileRef::Waste && matches!(to, PileRef::Tableau(_))
    }

    fn king_reveals_card(&self, from: PileRef, to: PileRef, count: u8) -> bool {
        let hides_card = self
            .column(from)
            .is_some_and(|column| !column.down.is_empty());
        self.column_is_empty(to) && self.moves_whole_run(from, count) && hides_card
    }
}
