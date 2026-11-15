//! Deck construction with unique card ids.

use crate::card::{Card, CardId, Rank, Suit};
use crate::rng::SplitMix64;

/// Builds `copies` sets of `suits`, each with 13 ranks. Ids are `0..` in build order:
/// copy by copy, suit by suit, ace to king. Stops early if ids would exceed `u16`.
pub fn build(suits: &[Suit], copies: u8) -> Vec<Card> {
    let mut cards = Vec::new();
    let mut next_id: u16 = 0;
    for _ in 0..copies {
        for &suit in suits {
            for rank in Rank::ALL {
                cards.push(Card {
                    id: CardId(next_id),
                    suit,
                    rank,
                });
                let Some(following) = next_id.checked_add(1) else {
                    return cards;
                };
                next_id = following;
            }
        }
    }
    cards
}

/// The standard 52-card deck in build order.
pub fn standard() -> Vec<Card> {
    build(&Suit::ALL, 1)
}

/// Shuffles `cards` in place using a generator seeded with `seed`.
pub fn shuffle_with_seed(cards: &mut [Card], seed: u64) {
    SplitMix64::new(seed).shuffle(cards);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn ids(cards: &[Card]) -> Vec<u16> {
        cards.iter().map(|c| c.id.0).collect()
    }

    #[test]
    fn standard_has_52_unique_faces_and_ids() {
        let deck = standard();
        assert_eq!(deck.len(), 52);
        assert_eq!(ids(&deck), (0..52).collect::<Vec<u16>>());
        let faces: HashSet<_> = deck.iter().map(|c| (c.suit, c.rank)).collect();
        assert_eq!(faces.len(), 52);
    }

    #[test]
    fn spider_build_has_duplicate_faces_and_unique_ids() {
        let deck = build(&[Suit::Spades], 8);
        assert_eq!(deck.len(), 104);
        let unique: HashSet<_> = ids(&deck).into_iter().collect();
        assert_eq!(unique.len(), 104);
        let faces: HashSet<_> = deck.iter().map(|c| (c.suit, c.rank)).collect();
        assert_eq!(faces.len(), 13);
    }

    #[test]
    fn build_edge_cases() {
        assert!(build(&[], 3).is_empty());
        assert!(build(&Suit::ALL, 0).is_empty());
    }

    const GOLDEN_SEED_0: [u16; 52] = [
        46, 36, 8, 40, 14, 24, 44, 9, 15, 0, 10, 6, 34, 7, 50, 38, 30, 1, 28, 4, 26, 5, 42, 22, 17,
        33, 20, 13, 37, 48, 11, 23, 41, 16, 27, 21, 2, 25, 39, 45, 47, 19, 49, 31, 51, 3, 32, 43,
        18, 29, 12, 35,
    ];
    const GOLDEN_SEED_42: [u16; 52] = [
        6, 28, 12, 39, 40, 0, 26, 25, 22, 4, 46, 36, 27, 42, 31, 17, 14, 43, 18, 13, 19, 1, 33, 7,
        16, 32, 51, 48, 29, 15, 35, 49, 3, 44, 45, 41, 20, 24, 47, 38, 30, 5, 50, 21, 23, 11, 2,
        34, 37, 8, 10, 9,
    ];

    #[test]
    fn golden_shuffle_seed_0() {
        let mut deck = standard();
        shuffle_with_seed(&mut deck, 0);
        assert_eq!(ids(&deck), GOLDEN_SEED_0);
    }

    #[test]
    fn golden_shuffle_seed_42() {
        let mut deck = standard();
        shuffle_with_seed(&mut deck, 42);
        assert_eq!(ids(&deck), GOLDEN_SEED_42);
    }
}
