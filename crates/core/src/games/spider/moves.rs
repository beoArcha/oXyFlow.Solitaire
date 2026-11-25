//! Move generation and the hint of Spider.

use super::rules::run_len;
use super::{COLUMNS, Column, Spider, index_ref};
use crate::card::Card;
use crate::games::transfer_parts;
use crate::moves::Move;
use crate::pile::PileRef;
use crate::rules;

type HintRule = fn(&Spider, PileRef, PileRef, u8) -> bool;

/// Hint priorities, best first. Only transfers are ranked; `Move::Stock` is the fallback.
const HINT_RULES: [HintRule; 5] = [
    Spider::extends_run,
    Spider::flips_card,
    Spider::empties_column,
    Spider::builds_on_other_suit,
    Spider::fills_empty_column,
];

fn columns() -> impl Iterator<Item = PileRef> {
    (0..COLUMNS).map(|i| PileRef::Tableau(index_ref(i)))
}

impl Spider {
    /// Every move [`Spider::execute`] accepts: the stock deal and each valid transfer of every
    /// count of the movable run of every column onto every column.
    pub(super) fn enumerate_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        if self.check_deal().is_ok() {
            moves.push(Move::Stock);
        }
        for from in columns() {
            moves.extend(self.transfers_from(from));
        }
        moves
    }

    fn transfers_from(&self, from: PileRef) -> Vec<Move> {
        let available = self.column(from).map_or(0, |column| run_len(&column.up));
        let mut moves = Vec::new();
        for count in 1..=u8::try_from(available).unwrap_or(u8::MAX) {
            let accepted = columns().filter(|&to| self.check_transfer(from, to, count).is_ok());
            moves.extend(accepted.map(|to| Move::Transfer { from, to, count }));
        }
        moves
    }

    /// The best legal move: one that extends a same-suit run, then one that flips a face-down
    /// card, then one that empties a column, then one that builds on another suit, then the stock.
    /// Only whole runs are moved, and a column with no face-down card is never moved to an empty
    /// column.
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

    /// The column and the index of the first moved card, when `count` cards are the whole movable
    /// run of `from`.
    fn whole_run(&self, from: PileRef, count: u8) -> Option<(&Column, usize)> {
        let column = self.column(from)?;
        let start = column.up.len().checked_sub(usize::from(count))?;
        (run_len(&column.up) == usize::from(count)).then_some((column, start))
    }

    fn target_top(&self, to: PileRef) -> Option<Card> {
        self.column(to)?.up.last().copied()
    }

    fn extends_run(&self, from: PileRef, to: PileRef, count: u8) -> bool {
        let moved = self
            .whole_run(from, count)
            .and_then(|(column, start)| column.up.get(start));
        let top = self.target_top(to);
        moved
            .zip(top)
            .is_some_and(|(first, top)| first.suit == top.suit)
    }

    fn flips_card(&self, from: PileRef, _to: PileRef, count: u8) -> bool {
        self.whole_run(from, count)
            .is_some_and(|(column, start)| start == 0 && !column.down.is_empty())
    }

    fn empties_column(&self, from: PileRef, to: PileRef, count: u8) -> bool {
        self.whole_run(from, count)
            .is_some_and(|(column, start)| start == 0 && column.down.is_empty())
            && self.target_top(to).is_some()
    }

    fn builds_on_other_suit(&self, from: PileRef, to: PileRef, count: u8) -> bool {
        let Some((column, start)) = self.whole_run(from, count) else {
            return false;
        };
        let beneath = start.checked_sub(1).and_then(|i| column.up.get(i));
        let moved = column.up.get(start);
        let already_linked = beneath
            .zip(moved)
            .is_some_and(|(&under, &first)| rules::fits_any_suit(under, first));
        self.target_top(to).is_some() && beneath.is_some() && !already_linked
    }

    /// A whole run that does not build on the card beneath it, moved into an empty column: the
    /// deal needs every column filled, and the run cannot be moved straight back.
    fn fills_empty_column(&self, from: PileRef, to: PileRef, count: u8) -> bool {
        let Some((column, start)) = self.whole_run(from, count) else {
            return false;
        };
        let beneath = start.checked_sub(1).and_then(|i| column.up.get(i));
        let linked = beneath
            .zip(column.up.get(start))
            .is_some_and(|(&under, &first)| rules::fits_any_suit(under, first));
        let leaves_cards = start > 0 || !column.down.is_empty();
        self.target_top(to).is_none() && self.column(to).is_some() && leaves_cards && !linked
    }
}
