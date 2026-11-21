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
/// A new building rule (same suit, up or down in suit, any suit but its own) is a new variant
/// with its arm in [`BuildRule::fits`]; nothing else in the module changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BuildRule {
    /// One rank lower and of the opposite colour (classic Yukon).
    DownAlternating,
}

impl BuildRule {
    /// Whether `card` may be placed on `top`.
    pub(crate) fn fits(self, top: Card, card: Card) -> bool {
        match self {
            BuildRule::DownAlternating => rules::fits_alternating(top, card),
        }
    }
}

/// How the shuffled deck is dealt to the columns, column 1 first: `down[i]` face-down cards with
/// `up[i]` face-up cards on top. The deal must use the whole deck; a rule set with a stock
/// (Australian Patience, P4-10e) adds the stock pile and the cards it holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Deal {
    pub(crate) down: [usize; COLUMNS],
    pub(crate) up: [usize; COLUMNS],
}

/// Classic Yukon: column *i* has *i* face-down cards; column 1 one face-up card, the others five.
const CLASSIC_DEAL: Deal = Deal {
    down: [0, 1, 2, 3, 4, 5, 6],
    up: [1, 5, 5, 5, 5, 5, 5],
};

impl YukonRules {
    /// The building rule of this rule set.
    pub(crate) const fn build(self) -> BuildRule {
        match self {
            YukonRules::Classic => BuildRule::DownAlternating,
        }
    }

    /// The deal of this rule set.
    pub(crate) const fn deal_plan(self) -> Deal {
        match self {
            YukonRules::Classic => CLASSIC_DEAL,
        }
    }
}
