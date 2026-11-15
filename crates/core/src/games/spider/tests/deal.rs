use super::*;
use crate::game::SuitCount;
use crate::rng::SplitMix64;
use std::collections::HashSet;

const SUIT_COUNTS: [SuitCount; 3] = [SuitCount::One, SuitCount::Two, SuitCount::Four];
const GOLDEN_TOPS_ONE: [&str; 10] = ["10S", "JS", "7S", "AS", "JS", "QS", "JS", "KS", "KS", "2S"];
const GOLDEN_TOPS_TWO: [&str; 10] = ["10H", "JS", "7S", "AH", "JH", "QS", "JH", "KH", "KH", "2S"];
const GOLDEN_TOPS_FOUR: [&str; 10] = ["10D", "JC", "7H", "AD", "JS", "QH", "JS", "KS", "KD", "2H"];

fn code(card: &Card) -> String {
    let suit = SUITS
        .chars()
        .nth(Suit::ALL.iter().position(|s| *s == card.suit).unwrap());
    format!("{}{}", RANKS[usize::from(card.rank.index())], suit.unwrap())
}

fn tops(game: &Spider) -> Vec<String> {
    game.columns
        .iter()
        .map(|c| code(c.up.last().unwrap()))
        .collect()
}

#[test]
fn the_deal_has_the_spider_layout() {
    for suits in SUIT_COUNTS {
        let piles = Spider::deal(suits, 7).piles();
        let order: Vec<PileRef> = piles.iter().map(|p| p.pile).collect();
        let expected: Vec<PileRef> = std::iter::once(PileRef::Stock)
            .chain((0..8).map(found))
            .chain((0..10).map(tab))
            .collect();
        assert_eq!(order, expected);
        assert_eq!(piles[0].cards.len(), 50);
        assert!(piles[0].cards.iter().all(|c| !c.face_up));
        assert!(piles[1..9].iter().all(|p| p.cards.is_empty()));
        for (index, pile) in piles[9..].iter().enumerate() {
            assert_eq!(pile.cards.len(), if index < 4 { 6 } else { 5 });
            let face_up: Vec<bool> = pile.cards.iter().map(|c| c.face_up).collect();
            assert_eq!(face_up.iter().filter(|up| **up).count(), 1);
            assert_eq!(face_up.last(), Some(&true));
        }
        let ids: HashSet<u16> = piles
            .iter()
            .flat_map(|p| p.cards.iter().map(|c| c.card.id.0))
            .collect();
        assert_eq!(ids.len(), 104);
    }
}

#[test]
fn the_deal_holds_the_right_suits() {
    let expected = [
        (SuitCount::One, 1),
        (SuitCount::Two, 2),
        (SuitCount::Four, 4),
    ];
    for (suits, kinds) in expected {
        let game = Spider::deal(suits, 3);
        let all: Vec<Card> = game
            .stock
            .iter()
            .chain(game.columns.iter().flat_map(|c| c.down.iter().chain(&c.up)))
            .copied()
            .collect();
        let distinct: HashSet<Suit> = all.iter().map(|c| c.suit).collect();
        assert_eq!(distinct.len(), kinds);
        for suit in distinct {
            let count = all.iter().filter(|c| c.suit == suit).count();
            assert_eq!(count * kinds, 104, "{suits:?} {suit:?}");
        }
    }
}

#[test]
fn the_same_seed_gives_the_same_deal() {
    for suits in SUIT_COUNTS {
        for seed in [0, 1, 42, u64::MAX] {
            let first = Spider::deal(suits, seed).piles();
            assert_eq!(first, Spider::deal(suits, seed).piles());
        }
        let different = Spider::deal(suits, 43).piles();
        assert_ne!(different, Spider::deal(suits, 42).piles());
    }
}

#[test]
fn golden_deal_for_seed_42() {
    let golden = [
        (SuitCount::One, GOLDEN_TOPS_ONE),
        (SuitCount::Two, GOLDEN_TOPS_TWO),
        (SuitCount::Four, GOLDEN_TOPS_FOUR),
    ];
    for (suits, expected) in golden {
        assert_eq!(tops(&Spider::deal(suits, 42)), expected, "{suits:?}");
    }
}

fn candidate_piles() -> Vec<PileRef> {
    let mut piles = vec![PileRef::Stock, PileRef::Waste];
    piles.extend((0..8).map(found));
    piles.extend((0..10).map(tab));
    piles.extend([found(8), tab(10), PileRef::Cell(0)]);
    piles
}

fn assert_invariants(game: &Spider) {
    for column in &game.columns {
        assert!(!column.up.is_empty() || column.down.is_empty());
    }
    for foundation in game.foundations.iter().filter(|f| !f.is_empty()) {
        assert_eq!(foundation.len(), 13);
        assert_eq!(foundation[0].rank, Rank::King);
        assert!(crate::rules::descending_same_suit(foundation));
    }
}

fn assert_legal_moves_equal_accepted_moves(game: &Spider) -> usize {
    assert_invariants(game);
    let legal = game.legal_moves();
    let unique: HashSet<Move> = legal.iter().copied().collect();
    assert_eq!(unique.len(), legal.len(), "duplicate legal moves");
    let piles = candidate_piles();
    let mut probe = game.clone();
    let mut accepted = 0;
    for &from in &piles {
        for &to in &piles {
            for count in 0..=14 {
                let mv = transfer(from, to, count);
                let result = probe.apply(mv);
                assert_eq!(result.is_ok(), unique.contains(&mv), "{mv:?}: {result:?}");
                if result.is_ok() {
                    accepted += 1;
                    probe = game.clone();
                }
            }
        }
    }
    assert_eq!(probe.piles(), game.piles(), "a rejected move changed state");
    let stock_ok = probe.apply(Move::Stock).is_ok();
    assert_eq!(stock_ok, unique.contains(&Move::Stock));
    assert_eq!(accepted + usize::from(stock_ok), legal.len());
    accepted
}

#[test]
fn legal_moves_are_exactly_the_moves_apply_accepts() {
    let mut transfers = 0;
    for suits in SUIT_COUNTS {
        for seed in 0..4 {
            let mut game = Spider::deal(suits, seed);
            let mut rng = SplitMix64::new(seed);
            for _ in 0..80 {
                transfers += assert_legal_moves_equal_accepted_moves(&game);
                let legal = game.legal_moves();
                let Some(last) = u64::try_from(legal.len()).ok().filter(|n| *n > 0) else {
                    break;
                };
                let pick = usize::try_from(rng.below(last)).unwrap();
                assert_eq!(game.apply(legal[pick]), Ok(()));
            }
        }
    }
    assert!(transfers > 0);
}
