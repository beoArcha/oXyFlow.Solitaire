//! Shared pure predicates over cards. Games compose them; none holds state.

use crate::card::Card;

fn steps_down(upper: Card, lower: Card) -> bool {
    upper.rank.lower() == Some(lower.rank)
}

fn every_pair(cards: &[Card], fits: fn(Card, Card) -> bool) -> bool {
    cards.windows(2).all(|pair| fits(pair[0], pair[1]))
}

/// `lower` is one rank below `upper` and of the opposite color.
pub fn fits_alternating(upper: Card, lower: Card) -> bool {
    steps_down(upper, lower) && upper.suit.color() != lower.suit.color()
}

/// `lower` is one rank below `upper` and of the same suit.
pub fn fits_same_suit(upper: Card, lower: Card) -> bool {
    steps_down(upper, lower) && upper.suit == lower.suit
}

/// `lower` is one rank below `upper`, any suit.
pub fn fits_any_suit(upper: Card, lower: Card) -> bool {
    steps_down(upper, lower)
}

/// Every neighbouring pair descends by one rank and alternates color. Empty and single runs pass.
pub fn descending_alternating(cards: &[Card]) -> bool {
    every_pair(cards, fits_alternating)
}

/// Every neighbouring pair descends by one rank within one suit. Empty and single runs pass.
pub fn descending_same_suit(cards: &[Card]) -> bool {
    every_pair(cards, fits_same_suit)
}

/// Every neighbouring pair descends by one rank, any suit. Empty and single runs pass.
pub fn descending_any_suit(cards: &[Card]) -> bool {
    every_pair(cards, fits_any_suit)
}

/// `card` may go on a foundation whose cards (bottom to top) are `pile`: an ace on an empty
/// foundation, otherwise the next rank up in the suit of the top card.
pub fn builds_foundation_up(pile: &[Card], card: Card) -> bool {
    match pile.last() {
        None => card.rank.index() == 0,
        Some(top) => top.suit == card.suit && top.rank.higher() == Some(card.rank),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::{CardId, Rank, Suit};

    fn c(suit: Suit, rank: Rank) -> Card {
        Card {
            id: CardId(0),
            suit,
            rank,
        }
    }

    #[test]
    fn alternating() {
        let run = [
            c(Suit::Spades, Rank::King),
            c(Suit::Hearts, Rank::Queen),
            c(Suit::Clubs, Rank::Jack),
        ];
        assert!(descending_alternating(&run));
        assert!(!descending_alternating(&[
            c(Suit::Spades, Rank::King),
            c(Suit::Clubs, Rank::Queen)
        ]));
        assert!(!descending_alternating(&[
            c(Suit::Spades, Rank::King),
            c(Suit::Hearts, Rank::Jack)
        ]));
        assert!(!descending_alternating(&[
            c(Suit::Hearts, Rank::Two),
            c(Suit::Spades, Rank::Three)
        ]));
        assert!(descending_alternating(&[]));
        assert!(descending_alternating(&[c(Suit::Hearts, Rank::Ace)]));
        assert!(descending_alternating(&[
            c(Suit::Hearts, Rank::Two),
            c(Suit::Clubs, Rank::Ace)
        ]));
    }

    #[test]
    fn same_suit() {
        let run = [
            c(Suit::Hearts, Rank::Three),
            c(Suit::Hearts, Rank::Two),
            c(Suit::Hearts, Rank::Ace),
        ];
        assert!(descending_same_suit(&run));
        assert!(!descending_same_suit(&[
            c(Suit::Hearts, Rank::Three),
            c(Suit::Spades, Rank::Two)
        ]));
        assert!(!descending_same_suit(&[
            c(Suit::Hearts, Rank::Three),
            c(Suit::Hearts, Rank::Five)
        ]));
        assert!(descending_same_suit(&[]));
        assert!(descending_same_suit(&[c(Suit::Clubs, Rank::King)]));
    }

    #[test]
    fn any_suit() {
        let run = [
            c(Suit::Hearts, Rank::Three),
            c(Suit::Spades, Rank::Two),
            c(Suit::Hearts, Rank::Ace),
        ];
        assert!(descending_any_suit(&run));
        assert!(!descending_any_suit(&[
            c(Suit::Hearts, Rank::Two),
            c(Suit::Hearts, Rank::Three)
        ]));
        assert!(!descending_any_suit(&[
            c(Suit::Hearts, Rank::Two),
            c(Suit::Hearts, Rank::Two)
        ]));
        assert!(!descending_any_suit(&[
            c(Suit::Hearts, Rank::Ace),
            c(Suit::Hearts, Rank::King)
        ]));
        assert!(descending_any_suit(&[]));
        assert!(descending_any_suit(&[c(Suit::Clubs, Rank::Ace)]));
    }

    #[test]
    fn foundation() {
        assert!(builds_foundation_up(&[], c(Suit::Clubs, Rank::Ace)));
        assert!(!builds_foundation_up(&[], c(Suit::Clubs, Rank::Two)));
        let pile = [c(Suit::Clubs, Rank::Ace)];
        assert!(builds_foundation_up(&pile, c(Suit::Clubs, Rank::Two)));
        assert!(!builds_foundation_up(&pile, c(Suit::Spades, Rank::Two)));
        assert!(!builds_foundation_up(&pile, c(Suit::Clubs, Rank::Three)));
        assert!(!builds_foundation_up(&pile, c(Suit::Clubs, Rank::Ace)));
        let almost = [c(Suit::Clubs, Rank::Queen)];
        assert!(builds_foundation_up(&almost, c(Suit::Clubs, Rank::King)));
        let full = [c(Suit::Clubs, Rank::King)];
        assert!(!builds_foundation_up(&full, c(Suit::Clubs, Rank::Ace)));
    }
}
