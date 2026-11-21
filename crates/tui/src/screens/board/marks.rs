//! Turning the controller state into the highlights the board draws.

use oxysol_core::{Game, Move, PileRef, PileSnapshot};

use super::Pick;
use super::cursor::Cursor;
use crate::widgets::pile::{CardPos, Marks};

fn len_of(piles: &[PileSnapshot], pile: PileRef) -> usize {
    piles
        .iter()
        .find(|snapshot| snapshot.pile == pile)
        .map_or(0, |snapshot| snapshot.cards.len())
}

fn run_start(piles: &[PileSnapshot], pile: PileRef, count: usize) -> usize {
    len_of(piles, pile).saturating_sub(count)
}

fn top(piles: &[PileSnapshot], pile: PileRef) -> CardPos {
    CardPos {
        pile,
        index: len_of(piles, pile).saturating_sub(1),
    }
}

fn hint_source(
    piles: &[PileSnapshot],
    pile: PileRef,
    count: usize,
) -> impl Iterator<Item = CardPos> {
    let len = len_of(piles, pile);
    (len.saturating_sub(count)..len).map(move |index| CardPos { pile, index })
}

fn hints(piles: &[PileSnapshot], hint: Move) -> Vec<CardPos> {
    match hint {
        Move::Stock => vec![top(piles, PileRef::Stock)],
        Move::Transfer { from, to, count } => hint_source(piles, from, usize::from(count))
            .chain([top(piles, to)])
            .collect(),
        Move::Remove { first, second } => [Some(first), second]
            .into_iter()
            .flatten()
            .map(|pile| top(piles, pile))
            .collect(),
    }
}

/// The piles whose card the game reports as covered: they are drawn in the blocked style.
pub(super) fn blocked(game: &dyn Game, piles: &[PileSnapshot]) -> Vec<PileRef> {
    piles
        .iter()
        .map(|snapshot| snapshot.pile)
        .filter(|&pile| game.is_covered(pile))
        .collect()
}

/// The highlights for the cursor, the picked run with its targets, and the hint.
pub(super) fn build(
    piles: &[PileSnapshot],
    cursor: &Cursor,
    pick: Option<&Pick>,
    hint: Option<Move>,
) -> Marks {
    Marks {
        focus: Some(CardPos {
            pile: cursor.pile,
            index: run_start(piles, cursor.pile, cursor.depth),
        }),
        selected: pick.map(|pick| CardPos {
            pile: pick.source,
            index: run_start(piles, pick.source, usize::from(pick.count)),
        }),
        targets: pick
            .map(|pick| pick.choices.iter().filter_map(|c| c.target).collect())
            .unwrap_or_default(),
        hints: hint.map(|mv| hints(piles, mv)).unwrap_or_default(),
        blocked: Vec::new(),
    }
}
