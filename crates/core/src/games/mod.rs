//! Rules of the games. Internal: callers reach them only through [`crate::deal`].

pub(crate) mod finish;
pub(crate) mod freecell;
pub(crate) mod golf;
pub(crate) mod klondike;
pub(crate) mod progress;
pub(crate) mod pyramid;
pub(crate) mod spider;
pub(crate) mod yukon;

use crate::card::Card;
use crate::moves::Move;
use crate::pile::{PileRef, PileSnapshot, TableCard};

/// A pile reference index as `u8`; a count that cannot occur saturates instead of panicking.
pub(crate) fn index_ref(index: usize) -> u8 {
    u8::try_from(index).unwrap_or(u8::MAX)
}

/// A pile of face-down cards `down` below face-up cards `up`, both bottom to top.
pub(crate) fn snapshot(pile: PileRef, down: &[Card], up: &[Card]) -> PileSnapshot {
    let face = |face_up| move |&card| TableCard { card, face_up };
    PileSnapshot {
        pile,
        cards: down
            .iter()
            .map(face(false))
            .chain(up.iter().map(face(true)))
            .collect(),
    }
}

/// The parts of a [`Move::Transfer`]; other moves have none.
pub(crate) fn transfer_parts(mv: Move) -> Option<(PileRef, PileRef, u8)> {
    match mv {
        Move::Transfer { from, to, count } => Some((from, to, count)),
        Move::Stock | Move::Remove { .. } => None,
    }
}
