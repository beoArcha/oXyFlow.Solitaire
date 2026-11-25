//! Hint profiles (P5-07). Every game ranks its moves by its own rules; a profile says whether the
//! hint first looks ahead for the legal move that improves the table the most. The profile of each
//! game is the one that won the most deals played by hints only (decision T-48).

use crate::game::Game;
use crate::moves::Move;
use crate::pile::{PileRef, PileSnapshot, TableCard};

/// A card on a foundation.
const HOME: i32 = 100;
/// A face-down card still to turn over.
const HIDDEN: i32 = 25;
/// An empty tableau column.
const EMPTY_COLUMN: i32 = 6;
/// An empty free cell.
const FREE_CELL: i32 = 3;
/// A face-up card resting on the next higher rank.
const LINKED: i32 = 4;
/// A linked card that also follows the suit of the card beneath it.
const SAME_SUIT: i32 = 2;
/// Most positions one hint looks at, both depths together, so a hint stays near a millisecond
/// (the interface asks for it on every redraw).
const LOOK_BUDGET: usize = 400;

/// How far the table is towards a win; higher is better.
pub(crate) fn score(piles: &[PileSnapshot]) -> i32 {
    piles.iter().map(pile_score).sum()
}

fn pile_score(snapshot: &PileSnapshot) -> i32 {
    let count = i32::try_from(snapshot.cards.len()).unwrap_or(i32::MAX / HOME);
    match snapshot.pile {
        PileRef::Foundation(_) => count * HOME,
        PileRef::Cell(_) if snapshot.cards.is_empty() => FREE_CELL,
        PileRef::Tableau(_) if snapshot.cards.is_empty() => EMPTY_COLUMN,
        PileRef::Tableau(_) => column_score(&snapshot.cards),
        _ => 0,
    }
}

fn column_score(cards: &[TableCard]) -> i32 {
    let hidden = cards.iter().filter(|card| !card.face_up).count();
    let hidden = i32::try_from(hidden).unwrap_or(i32::MAX / HIDDEN);
    let links: i32 = cards.windows(2).map(link_score).sum();
    links - hidden * HIDDEN
}

fn link_score(pair: &[TableCard]) -> i32 {
    let [under, over] = pair else {
        return 0;
    };
    if !under.face_up || !over.face_up || over.card.rank.index() + 1 != under.card.rank.index() {
        return 0;
    }
    if over.card.suit == under.card.suit {
        LINKED + SAME_SUIT
    } else {
        LINKED
    }
}

/// How a game picks its hint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HintProfile {
    /// Only the game's own ranked rules.
    Rules,
    /// The move with the largest strict gain one move ahead, else the rules.
    #[allow(
        dead_code,
        reason = "one of the three profiles; no game measures best with it yet (T-48)"
    )]
    Step,
    /// As `Step`, and when no move gains, the move that sets up the largest gain two moves deep,
    /// else the rules.
    Lookahead,
}

/// What a hint may know about the cards the player cannot see.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HintKnowledge {
    /// Every card, face-down cards and the stock order included.
    #[allow(
        dead_code,
        reason = "the in-play option of the knowledge card; the hint uses Player until then"
    )]
    Full,
    /// Only what the player can see: the look ahead never goes past a move that turns a card
    /// over or draws from the stock. `stock_memory` is how many seen stock cards the player
    /// remembers; the rule-based hints do not use it yet.
    Player { stock_memory: u8 },
}

/// The knowledge of a hint in play until it becomes a setting: the player's, remembering three
/// stock cards.
pub(crate) const PLAY_KNOWLEDGE: HintKnowledge = HintKnowledge::Player { stock_memory: 3 };

/// The hint of `game` under `profile` and `knowledge`; `rules` is the game's own ranked
/// suggestion.
pub(crate) fn hint<G: Game + Clone>(
    game: &G,
    profile: HintProfile,
    knowledge: HintKnowledge,
    rules: impl FnOnce() -> Option<Move>,
) -> Option<Move> {
    let looked = match profile {
        HintProfile::Rules => None,
        HintProfile::Step => best_step(game, false, knowledge),
        HintProfile::Lookahead => best_step(game, true, knowledge),
    };
    looked.or_else(rules)
}

/// Face-down cards on the table and cards in the stock: a move that lowers either shows the
/// player a card they did not know.
fn unseen(piles: &[PileSnapshot]) -> usize {
    piles
        .iter()
        .map(|snapshot| match snapshot.pile {
            PileRef::Stock => snapshot.cards.len(),
            _ => snapshot.cards.iter().filter(|card| !card.face_up).count(),
        })
        .sum()
}

/// The look-ahead step: the legal move with the largest strict gain in [`score`]; when no move gains,
/// the move that sets up the largest gain two moves deep (a free cell used, a run split to reach a
/// card). The first move wins a tie. `None` when neither exists.
fn best_step<G: Game + Clone>(game: &G, setups: bool, knowledge: HintKnowledge) -> Option<Move> {
    let piles = game.piles();
    let base = score(&piles);
    let hidden = unseen(&piles);
    let firsts: Vec<(Move, G)> = successors(game).collect();
    let mut budget = LOOK_BUDGET.saturating_sub(firsts.len());
    let mut gain_now: Option<(i32, Move)> = None;
    let mut setup: Option<(i32, Move)> = None;
    for (mv, next) in firsts {
        let next_piles = next.piles();
        let gain = score(&next_piles) - base;
        let reveals = unseen(&next_piles) < hidden;
        let may_look = knowledge == HintKnowledge::Full || !reveals;
        if gain > 0 {
            keep_best(&mut gain_now, gain, mv);
        } else if setups && may_look && gain_now.is_none() && budget > 0 {
            let reach = best_gain(&next, &mut budget).map_or(gain, |follow| gain + follow);
            if reach > 0 {
                keep_best(&mut setup, reach, mv);
            }
        }
    }
    gain_now.or(setup).map(|(_, mv)| mv)
}

fn successors<G: Game + Clone>(game: &G) -> impl Iterator<Item = (Move, G)> + '_ {
    game.legal_moves().into_iter().filter_map(|mv| {
        let mut next = game.clone();
        next.apply(mv).ok().map(|()| (mv, next))
    })
}

fn best_gain<G: Game + Clone>(game: &G, budget: &mut usize) -> Option<i32> {
    let base = score(&game.piles());
    let mut best = None;
    for (_, next) in successors(game) {
        if *budget == 0 {
            break;
        }
        *budget -= 1;
        let gain = score(&next.piles()) - base;
        if gain > 0 && best.is_none_or(|top| gain > top) {
            best = Some(gain);
        }
    }
    best
}

fn keep_best(best: &mut Option<(i32, Move)>, gain: i32, mv: Move) {
    if best.is_none_or(|(top, _)| gain > top) {
        *best = Some((gain, mv));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::{Card, CardId, Rank, Suit};
    use crate::game::Variant;

    fn table_card(rank: Rank, suit: Suit, face_up: bool) -> TableCard {
        TableCard {
            card: Card {
                id: CardId(0),
                suit,
                rank,
            },
            face_up,
        }
    }

    fn pile(pile: PileRef, cards: Vec<TableCard>) -> PileSnapshot {
        PileSnapshot { pile, cards }
    }

    #[test]
    fn a_card_home_outweighs_everything_and_a_hidden_card_costs() {
        let home = [pile(
            PileRef::Foundation(0),
            vec![table_card(Rank::Ace, Suit::Spades, true)],
        )];
        let hidden = [pile(
            PileRef::Tableau(0),
            vec![table_card(Rank::Two, Suit::Spades, false)],
        )];
        assert_eq!(score(&home), HOME);
        assert_eq!(score(&hidden), -HIDDEN);
        assert_eq!(
            score(&[pile(PileRef::Tableau(0), Vec::new())]),
            EMPTY_COLUMN
        );
    }

    #[test]
    fn a_link_counts_more_in_suit() {
        let linked = |suit| {
            pile(
                PileRef::Tableau(0),
                vec![
                    table_card(Rank::Three, Suit::Spades, true),
                    table_card(Rank::Two, suit, true),
                ],
            )
        };
        assert_eq!(score(&[linked(Suit::Hearts)]), LINKED);
        assert_eq!(score(&[linked(Suit::Spades)]), LINKED + SAME_SUIT);
    }

    #[test]
    fn the_stock_and_face_down_cards_are_unseen() {
        let piles = [
            pile(
                PileRef::Stock,
                vec![table_card(Rank::Four, Suit::Clubs, false); 3],
            ),
            pile(
                PileRef::Tableau(0),
                vec![
                    table_card(Rank::Five, Suit::Clubs, false),
                    table_card(Rank::Six, Suit::Clubs, true),
                ],
            ),
        ];
        assert_eq!(unseen(&piles), 4);
    }

    #[test]
    fn every_hint_is_a_legal_move_in_every_variant() {
        for variant in Variant::ALL {
            for seed in 0..5 {
                let mut game = crate::deal(variant, seed);
                for _ in 0..40 {
                    let Some(mv) = game.hint() else { break };
                    assert!(
                        game.legal_moves().contains(&mv),
                        "{variant:?} {seed} {mv:?}"
                    );
                    assert_eq!(game.apply(mv), Ok(()), "{variant:?} {seed} {mv:?}");
                }
            }
        }
    }
}
