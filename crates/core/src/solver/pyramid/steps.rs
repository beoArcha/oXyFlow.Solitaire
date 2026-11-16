//! The steps of a Pyramid position: the stock move and every removal of a king or of a pair of
//! available cards that sum to 13, ordered by how many cards the removal frees.

use super::position::{COVERS, KING_VALUE, POSITIONS, Position, Slot, TARGET};
use std::cmp::Reverse;

/// One edge of the search.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::solver) enum Step {
    /// Draw a card to the waste or recycle the waste.
    Draw,
    /// Remove a lone king (`second` is `None`) or a pair.
    Remove { first: Slot, second: Option<Slot> },
}

const MAX_AVAILABLE: usize = POSITIONS + 2;

/// The cards that may leave the table, in the canonical order: stock top, waste top, then the
/// uncovered pyramid cards row by row.
pub(super) struct Available {
    items: [(Slot, u8); MAX_AVAILABLE],
    len: usize,
}

impl Available {
    pub(super) fn items(&self) -> &[(Slot, u8)] {
        &self.items[..self.len]
    }

    fn push(&mut self, slot: Slot, value: u8) {
        self.items[self.len] = (slot, value);
        self.len += 1;
    }
}

impl Position {
    pub(super) fn available(&self) -> Available {
        let mut available = Available {
            items: [(Slot::Stock, 0); MAX_AVAILABLE],
            len: 0,
        };
        if let Some(value) = self.stock_top() {
            available.push(Slot::Stock, value);
        }
        if let Some(value) = self.waste_top() {
            available.push(Slot::Waste, value);
        }
        for at in (0..POSITIONS).filter(|&at| self.is_free(at)) {
            available.push(Slot::Pyramid(u8::try_from(at).unwrap_or(0)), self.values[at]);
        }
        available
    }

    /// Every legal move, each pair once in the canonical order.
    pub(super) fn direct(&self, out: &mut Vec<Step>) {
        if self.can_draw() {
            out.push(Step::Draw);
        }
        let available = self.available();
        let items = available.items();
        for (i, &(first, a)) in items.iter().enumerate() {
            if a == KING_VALUE {
                out.push(Step::Remove {
                    first,
                    second: None,
                });
            }
            for &(second, b) in &items[i + 1..] {
                if a + b == TARGET {
                    out.push(Step::Remove {
                        first,
                        second: Some(second),
                    });
                }
            }
        }
    }

    /// The pyramid cards that a removal of `slots` would uncover.
    fn uncovered_by(&self, slots: [Option<Slot>; 2]) -> u32 {
        let removed = slots
            .iter()
            .flatten()
            .filter_map(|slot| match slot {
                Slot::Pyramid(at) => Some(1u32 << at),
                _ => None,
            })
            .fold(0, |bits, bit| bits | bit);
        let after = self.mask & !removed;
        let freed = (0..POSITIONS)
            .filter(|&at| self.mask & (1 << at) != 0 && after & (1 << at) != 0)
            .filter(|&at| self.mask & COVERS[at] != 0 && after & COVERS[at] == 0)
            .count();
        u32::try_from(freed).unwrap_or(0)
    }

    fn score_step(&self, step: Step) -> u32 {
        match step {
            Step::Draw => 100,
            Step::Remove { first, second } => {
                let from_pyramid = |slot: Slot| u32::from(matches!(slot, Slot::Pyramid(_)));
                let pyramid_cards = from_pyramid(first) + second.map_or(0, from_pyramid);
                200 * pyramid_cards + 30 * self.uncovered_by([Some(first), second])
            }
        }
    }

    /// The steps worth searching, best first. Nothing is dropped, except in a position where some
    /// pyramid card has no partner left at all.
    pub(super) fn ordered(&self, out: &mut Vec<Step>) {
        if self.is_hopeless() {
            return;
        }
        let mut all = Vec::new();
        self.direct(&mut all);
        all.sort_by_key(|&step| Reverse(self.score_step(step)));
        out.extend(all);
    }

    /// Whether some value has more cards on the pyramid than the table has partners (cards worth
    /// 13 minus it) in the pyramid, stock and waste together: those cards can never all be removed.
    fn is_hopeless(&self) -> bool {
        let mut pyramid = [0u8; 14];
        let mut table = [0u8; 14];
        for at in (0..POSITIONS).filter(|&at| self.mask & (1 << at) != 0) {
            pyramid[usize::from(self.values[at])] += 1;
            table[usize::from(self.values[at])] += 1;
        }
        for value in self.talon_values() {
            table[usize::from(value)] += 1;
        }
        (1..TARGET).any(|v| pyramid[usize::from(v)] > table[usize::from(TARGET - v)])
    }
}
