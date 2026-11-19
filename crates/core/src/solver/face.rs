//! Compact card faces for the solver models: `suit * 13 + rank` with the suit in the order of
//! [`Suit::ALL`] and the rank from 0 (ace) to 12 (king). The solver never needs card ids.

use crate::card::{Card, Suit};
use crate::pile::{PileRef, PileSnapshot, TableCard};

pub(super) type Face = u8;

pub(super) const RANKS: u8 = 13;
pub(super) const KING: u8 = 12;
pub(super) const NONE: Face = u8::MAX;

pub(super) fn face(card: Card) -> Face {
    let suit = match card.suit {
        Suit::Clubs => 0,
        Suit::Diamonds => 1,
        Suit::Hearts => 2,
        Suit::Spades => 3,
    };
    suit * RANKS + card.rank.index()
}

pub(super) fn suit(face: Face) -> u8 {
    face / RANKS
}

pub(super) fn rank(face: Face) -> u8 {
    face % RANKS
}

pub(super) fn is_red(face: Face) -> bool {
    matches!(suit(face), 1 | 2)
}

/// `lower` may be built onto `upper`: one rank below and the opposite color.
pub(super) fn fits_alternating(upper: Face, lower: Face) -> bool {
    rank(upper) == rank(lower) + 1 && is_red(upper) != is_red(lower)
}

/// The cards of the pile `pile`, or an empty slice when the snapshot has no such pile.
pub(super) fn cards_of(piles: &[PileSnapshot], pile: PileRef) -> &[TableCard] {
    piles
        .iter()
        .find(|snapshot| snapshot.pile == pile)
        .map_or(&[], |snapshot| snapshot.cards.as_slice())
}

/// Copies the faces of `cards` to the start of `target` and returns how many there are, or `None`
/// when they do not fit.
pub(super) fn fill<const N: usize>(target: &mut [Face; N], cards: &[TableCard]) -> Option<u8> {
    if cards.len() > N {
        return None;
    }
    for (slot, table_card) in target.iter_mut().zip(cards) {
        *slot = face(table_card.card);
    }
    u8::try_from(cards.len()).ok()
}

/// The number of face-down cards at the bottom of a pile listed bottom to top.
pub(super) fn hidden_count(cards: &[TableCard]) -> Option<u8> {
    u8::try_from(cards.iter().take_while(|card| !card.face_up).count()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::{CardId, Rank};

    fn card(suit: Suit, rank: Rank) -> Card {
        Card {
            id: CardId(0),
            suit,
            rank,
        }
    }

    #[test]
    fn faces_round_trip_suit_and_rank() {
        let queen = face(card(Suit::Hearts, Rank::Queen));
        assert_eq!((suit(queen), rank(queen)), (2, 11));
        assert!(is_red(queen));
        assert!(!is_red(face(card(Suit::Spades, Rank::Ace))));
    }

    #[test]
    fn alternating_needs_rank_and_color() {
        let black_seven = face(card(Suit::Spades, Rank::Seven));
        let red_six = face(card(Suit::Hearts, Rank::Six));
        let black_six = face(card(Suit::Clubs, Rank::Six));
        assert!(fits_alternating(black_seven, red_six));
        assert!(!fits_alternating(black_seven, black_six));
        assert!(!fits_alternating(red_six, black_seven));
    }
}
