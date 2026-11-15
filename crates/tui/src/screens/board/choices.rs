//! The moves that start at one pile, in the form the controller needs: which move to apply when
//! the player picks a target, and which target it is.

use oxysol_core::{Move, PileRef};

/// One way to continue from a picked source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Choice {
    /// The move to apply; a Pyramid pair is oriented so that the picked card comes first.
    pub mv: Move,
    /// The pile to drop on; `None` for a move that needs no target (a lone king).
    pub target: Option<PileRef>,
}

fn choice(mv: Move, source: PileRef, count: u8) -> Option<Choice> {
    match mv {
        Move::Stock => None,
        Move::Transfer { from, to, count: n } if from == source && n == count => Some(Choice {
            mv,
            target: Some(to),
        }),
        Move::Transfer { .. } => None,
        Move::Remove { first, second } if first == source => Some(Choice { mv, target: second }),
        Move::Remove {
            first,
            second: Some(second),
        } if second == source => Some(Choice {
            mv: Move::Remove {
                first: source,
                second: Some(first),
            },
            target: Some(first),
        }),
        Move::Remove { .. } => None,
    }
}

/// Every legal continuation of picking the top `count` cards of `source`.
pub(super) fn choices(legal: &[Move], source: PileRef, count: u8) -> Vec<Choice> {
    legal
        .iter()
        .filter_map(|&mv| choice(mv, source, count))
        .collect()
}

const fn quick_rank(target: Option<PileRef>) -> u8 {
    match target {
        None => 0,
        Some(PileRef::Foundation(_)) => 1,
        Some(PileRef::Tableau(_)) => 2,
        Some(PileRef::Cell(_)) => 4,
        Some(_) => 3,
    }
}

/// The move a quick move performs: no target needed first, then foundation, tableau, any other
/// pile and a free cell last; the first legal one among equals.
pub(super) fn quick(choices: &[Choice]) -> Option<Choice> {
    choices
        .iter()
        .copied()
        .min_by_key(|choice| quick_rank(choice.target))
}

#[cfg(test)]
mod tests {
    use super::*;

    const WASTE: PileRef = PileRef::Waste;
    const COLUMN: PileRef = PileRef::Tableau(2);
    const FOUNDATION: PileRef = PileRef::Foundation(0);

    fn transfer(from: PileRef, to: PileRef, count: u8) -> Move {
        Move::Transfer { from, to, count }
    }

    #[test]
    fn transfers_match_source_and_count() {
        let legal = [
            Move::Stock,
            transfer(WASTE, COLUMN, 1),
            transfer(WASTE, FOUNDATION, 1),
            transfer(COLUMN, FOUNDATION, 2),
        ];
        let found = choices(&legal, WASTE, 1);
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].target, Some(COLUMN));
        assert!(choices(&legal, COLUMN, 1).is_empty());
        assert_eq!(choices(&legal, COLUMN, 2).len(), 1);
    }

    #[test]
    fn a_pair_can_be_picked_from_either_end() {
        let pair = Move::Remove {
            first: WASTE,
            second: Some(COLUMN),
        };
        let from_first = choices(&[pair], WASTE, 1);
        assert_eq!(from_first[0].mv, pair);
        assert_eq!(from_first[0].target, Some(COLUMN));
        let from_second = choices(&[pair], COLUMN, 1);
        assert_eq!(from_second[0].target, Some(WASTE));
        assert_eq!(
            from_second[0].mv,
            Move::Remove {
                first: COLUMN,
                second: Some(WASTE)
            }
        );
    }

    #[test]
    fn a_lone_king_has_no_target() {
        let king = Move::Remove {
            first: WASTE,
            second: None,
        };
        assert_eq!(choices(&[king], WASTE, 1)[0].target, None);
    }

    #[test]
    fn quick_prefers_foundation_over_tableau_over_cell() {
        let legal = [
            transfer(WASTE, PileRef::Cell(0), 1),
            transfer(WASTE, COLUMN, 1),
            transfer(WASTE, FOUNDATION, 1),
        ];
        let best = quick(&choices(&legal, WASTE, 1));
        assert_eq!(best.and_then(|c| c.target), Some(FOUNDATION));
        let without = quick(&choices(&legal[..2], WASTE, 1));
        assert_eq!(without.and_then(|c| c.target), Some(COLUMN));
        assert_eq!(quick(&[]), None);
    }
}
