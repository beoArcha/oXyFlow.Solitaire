//! Building positions from text for the solver tests. Cards are written as `"AS"`, `"10H"`, `"KD"`.

use crate::card::{Card, CardId, Rank, Suit};
use crate::games::snapshot;
use crate::pile::{PileRef, PileSnapshot};

const RANKS: [&str; 13] = [
    "A", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K",
];
const SUITS: &str = "CDHS";

fn card(code: &str) -> Card {
    let (rank, suit) = code.split_at(code.len() - 1);
    let rank_index = RANKS.iter().position(|r| *r == rank).unwrap();
    let suit_index = SUITS.find(suit).unwrap();
    Card {
        id: CardId(u16::try_from(suit_index * 13 + rank_index).unwrap()),
        suit: Suit::ALL[suit_index],
        rank: Rank::ALL[rank_index],
    }
}

fn cards(codes: &str) -> Vec<Card> {
    codes.split_whitespace().map(card).collect()
}

/// A pile of the face-down cards `down` below the face-up cards `up`, both bottom to top.
pub(in crate::solver) fn pile(pile: PileRef, down: &str, up: &str) -> PileSnapshot {
    snapshot(pile, &cards(down), &cards(up))
}

/// The cards of `suit` from `from` up to the king, ace low, as text: `run_of('S', 3)` is
/// `"3S 4S ... KS"`.
pub(in crate::solver) fn run_of(suit: char, from: usize) -> String {
    RANKS[from - 1..]
        .iter()
        .map(|rank| format!("{rank}{suit}"))
        .collect::<Vec<_>>()
        .join(" ")
}
