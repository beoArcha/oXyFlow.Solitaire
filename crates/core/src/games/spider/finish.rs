//! The forced win of Spider.

use super::rules::run_len;
use super::{Column, Spider};
use crate::games::finish::greedy_finish;
use crate::games::index_ref;
use crate::moves::Move;
use crate::pile::PileRef;
use crate::rules;

impl Spider {
    /// Only when the stock is empty and every column is a single face-up same-suit descending run.
    /// Each suit then has as many runs ending at rank *r* as starting at rank *r*-1, so every run
    /// that is not a complete king-to-ace run (which is collected automatically) has a run of its
    /// own suit that continues it. Moving a whole run onto the run it continues, again and again,
    /// builds the complete runs and wins.
    pub(super) fn finishing_moves(&self) -> Option<Vec<Move>> {
        if !self.stock.is_empty() || !self.columns.iter().all(Self::is_single_run) {
            return None;
        }
        greedy_finish(self, Self::next_join)
    }

    fn is_single_run(column: &Column) -> bool {
        column.down.is_empty() && run_len(&column.up) == column.up.len()
    }

    fn next_join(&self) -> Option<Move> {
        let columns = || self.columns.iter().enumerate();
        columns().find_map(|(from, source)| {
            let first = source.up.first()?;
            let count = u8::try_from(source.up.len()).ok()?;
            columns().find_map(|(to, target)| {
                let top = target.up.last()?;
                let joins = to != from && rules::fits_same_suit(*top, *first);
                joins.then_some(Move::Transfer {
                    from: PileRef::Tableau(index_ref(from)),
                    to: PileRef::Tableau(index_ref(to)),
                    count,
                })
            })
        })
    }
}
