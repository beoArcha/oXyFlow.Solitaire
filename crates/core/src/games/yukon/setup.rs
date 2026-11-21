//! The parameters of a Yukon rule set: how a column is built ([`BuildRule`]) and how the deck is
//! dealt ([`Deal`]). A new member of the family is one [`YukonRules`] value mapped here; the move
//! code in `rules` and `moves` reads only these two descriptions.

use super::COLUMNS;
use crate::card::Card;
use crate::game::YukonRules;
use crate::rules;

/// Which card may go onto the top card of a non-empty column. Only the bottom card of a moved
/// group is checked; the cards above it may be in any order.
///
/// A new building rule is a new variant with its arm in [`BuildRule::fits`]; nothing else in the
/// module changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BuildRule {
    /// One rank lower and of the opposite colour (classic Yukon).
    DownAlternating,
    /// One rank lower and of the same suit (Russian Solitaire).
    DownSameSuit,
    /// One rank lower or higher and of the same suit (Alaska).
    UpOrDownSameSuit,
    /// One rank lower and of any suit but the top card's own (Moosehide).
    DownOtherSuit,
}

impl BuildRule {
    /// Whether `card` may be placed on `top`.
    pub(crate) fn fits(self, top: Card, card: Card) -> bool {
        match self {
            BuildRule::DownAlternating => rules::fits_alternating(top, card),
            BuildRule::DownSameSuit => rules::fits_same_suit(top, card),
            BuildRule::UpOrDownSameSuit => {
                rules::fits_same_suit(top, card) || rules::fits_same_suit(card, top)
            }
            BuildRule::DownOtherSuit => rules::fits_any_suit(top, card) && top.suit != card.suit,
        }
    }
}

/// How the shuffled deck is dealt to the columns, column 1 first: `down[i]` face-down cards with
/// `up[i]` face-up cards on top; the `stock` cards that remain go to the stock pile (Australian
/// Patience). The deal must use the whole deck.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Deal {
    pub(crate) down: [usize; COLUMNS],
    pub(crate) up: [usize; COLUMNS],
    pub(crate) stock: usize,
}

/// Classic Yukon: column *i* has *i* face-down cards; column 1 one face-up card, the others five.
const CLASSIC_DEAL: Deal = Deal {
    down: [0, 1, 2, 3, 4, 5, 6],
    up: [1, 5, 5, 5, 5, 5, 5],
    stock: 0,
};

/// Australian Patience: four face-up cards in every column, the other 24 cards in the stock.
const AUSTRALIAN_DEAL: Deal = Deal {
    down: [0; COLUMNS],
    up: [4; COLUMNS],
    stock: 24,
};

impl YukonRules {
    /// The building rule of this rule set.
    pub(crate) const fn build(self) -> BuildRule {
        match self {
            YukonRules::Classic => BuildRule::DownAlternating,
            YukonRules::Russian => BuildRule::DownSameSuit,
            YukonRules::Alaska => BuildRule::UpOrDownSameSuit,
            YukonRules::Moosehide => BuildRule::DownOtherSuit,
            YukonRules::Australian => BuildRule::DownSameSuit,
        }
    }

    /// The deal of this rule set.
    pub(crate) const fn deal_plan(self) -> Deal {
        match self {
            YukonRules::Classic
            | YukonRules::Russian
            | YukonRules::Alaska
            | YukonRules::Moosehide => CLASSIC_DEAL,
            YukonRules::Australian => AUSTRALIAN_DEAL,
        }
    }

    /// Whether the deal leaves cards in a stock, which is drawn once through to the waste.
    pub(crate) const fn has_stock(self) -> bool {
        self.deal_plan().stock > 0
    }
}
