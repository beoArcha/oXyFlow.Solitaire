use super::*;
use crate::rng::SplitMix64;
use std::collections::HashSet;

const STOCK: PileRef = PileRef::Stock;
const WASTE: PileRef = PileRef::Waste;
const DRAWS: [DrawCount; 2] = [DrawCount::One, DrawCount::Three];
const GOLDEN_TABLEAU_TOPS: [&str; 7] = ["7C", "KC", "AC", "5C", "6H", "7D", "10S"];
const GOLDEN_STOCK_TOP: [&str; 3] = ["10C", "JC", "9C"];

#[test]
fn the_deal_has_the_klondike_layout() {
    let piles = Klondike::deal(DrawCount::One, 7).piles();
    let order: Vec<PileRef> = piles.iter().map(|p| p.pile).collect();
    let expected: Vec<PileRef> = [STOCK, WASTE]
        .into_iter()
        .chain((0..4).map(found))
        .chain((0..7).map(tab))
        .collect();
    assert_eq!(order, expected);
    assert_eq!(piles[0].cards.len(), 24);
    assert!(piles[0].cards.iter().all(|c| !c.face_up));
    assert!(piles[1].cards.is_empty());
    assert!(piles[2..6].iter().all(|p| p.cards.is_empty()));
    for (index, pile) in piles[6..].iter().enumerate() {
        assert_eq!(pile.cards.len(), index + 1);
        let face_up: Vec<bool> = pile.cards.iter().map(|c| c.face_up).collect();
        assert_eq!(face_up.iter().filter(|up| **up).count(), 1);
        assert_eq!(face_up.last(), Some(&true));
    }
    let ids: HashSet<u16> = piles
        .iter()
        .flat_map(|p| p.cards.iter().map(|c| c.card.id.0))
        .collect();
    assert_eq!(ids.len(), 52);
}

#[test]
fn the_same_seed_gives_the_same_layout_for_both_draw_counts() {
    for seed in [0, 1, 42, u64::MAX] {
        let one = Klondike::deal(DrawCount::One, seed).piles();
        assert_eq!(one, Klondike::deal(DrawCount::Three, seed).piles());
        assert_eq!(one, Klondike::deal(DrawCount::One, seed).piles());
    }
    let different = Klondike::deal(DrawCount::One, 43).piles();
    assert_ne!(different, Klondike::deal(DrawCount::One, 42).piles());
}

fn code(card: &Card) -> String {
    let suit = SUITS
        .chars()
        .nth(Suit::ALL.iter().position(|s| *s == card.suit).unwrap());
    format!("{}{}", RANKS[usize::from(card.rank.index())], suit.unwrap())
}

#[test]
fn golden_deal_for_seed_42() {
    let game = Klondike::deal(DrawCount::One, 42);
    let tops: Vec<String> = game
        .columns
        .iter()
        .map(|c| code(c.up.last().unwrap()))
        .collect();
    assert_eq!(tops, GOLDEN_TABLEAU_TOPS);
    let stock: Vec<String> = game.stock.iter().rev().take(3).map(code).collect();
    assert_eq!(stock, GOLDEN_STOCK_TOP);
}

#[test]
fn draw_one_moves_the_top_stock_card_to_the_waste() {
    let mut game = table();
    game.stock = cards("AS 2S 3S");
    assert_accepted(&mut game, Move::Stock);
    assert_eq!(game.stock, cards("AS 2S"));
    assert_eq!(game.waste, cards("3S"));
}

#[test]
fn draw_three_moves_three_cards_and_fewer_when_fewer_remain() {
    let mut game = Klondike::empty(DrawCount::Three);
    game.stock = cards("AS 2S 3S 4S 5S");
    assert_accepted(&mut game, Move::Stock);
    assert_eq!(game.stock, cards("AS 2S"));
    assert_eq!(game.waste, cards("5S 4S 3S"));
    assert_accepted(&mut game, Move::Stock);
    assert!(game.stock.is_empty());
    assert_eq!(game.waste, cards("5S 4S 3S 2S AS"));
}

#[test]
fn only_the_top_waste_card_is_playable_after_drawing_three() {
    let mut game = Klondike::empty(DrawCount::Three);
    game.stock = cards("7S 4C 6D");
    game.columns[0] = column("", "8D");
    game.columns[1] = column("", "7C");
    assert_accepted(&mut game, Move::Stock);
    assert_eq!(game.waste, cards("6D 4C 7S"));
    let from_waste: Vec<Move> = game
        .legal_moves()
        .into_iter()
        .filter(|mv| mv.source() == Some(WASTE))
        .collect();
    assert_eq!(from_waste, vec![transfer(WASTE, tab(0), 1)]);
    assert_accepted(&mut game, transfer(WASTE, tab(0), 1));
    assert_eq!(game.waste, cards("6D 4C"));
}

#[test]
fn recycling_restores_the_original_draw_order() {
    for draw in DRAWS {
        let mut game = Klondike::deal(draw, 42);
        let initial = game.piles();
        let first_draw = {
            let mut probe = game.clone();
            assert_eq!(probe.apply(Move::Stock), Ok(()));
            cards_of(&probe, WASTE)
        };
        while !game.stock.is_empty() {
            assert_eq!(game.apply(Move::Stock), Ok(()));
        }
        assert_eq!(game.apply(Move::Stock), Ok(()));
        assert_eq!(game.piles(), initial, "{draw:?}");
        assert_eq!(game.apply(Move::Stock), Ok(()));
        assert_eq!(cards_of(&game, WASTE), first_draw, "{draw:?}");
    }
}

#[test]
fn recycling_keeps_the_order_of_the_remaining_cards() {
    let mut game = table();
    game.stock = cards("AS 2S 3S");
    for _ in 0..3 {
        assert_eq!(game.apply(Move::Stock), Ok(()));
    }
    game.waste.remove(1);
    assert_accepted(&mut game, Move::Stock);
    assert_eq!(game.stock, cards("AS 3S"));
    assert!(game.waste.is_empty());
}

#[test]
fn an_empty_stock_and_waste_give_stock_empty() {
    let game = table();
    assert_rejected(&game, Move::Stock, MoveError::StockEmpty);
}

fn candidate_piles() -> Vec<PileRef> {
    let mut piles = vec![STOCK, WASTE];
    piles.extend((0..4).map(found));
    piles.extend((0..7).map(tab));
    piles.extend([found(4), tab(7), PileRef::Cell(0)]);
    piles
}

fn assert_legal_moves_equal_accepted_moves(game: &Klondike) -> usize {
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
    assert_eq!(
        probe.piles(),
        game.piles(),
        "a rejected move changed the state"
    );
    let stock_ok = probe.apply(Move::Stock).is_ok();
    assert_eq!(stock_ok, unique.contains(&Move::Stock));
    assert_eq!(accepted + usize::from(stock_ok), legal.len());
    accepted
}

#[test]
fn legal_moves_are_exactly_the_moves_apply_accepts() {
    let mut transfers = 0;
    for draw in DRAWS {
        for seed in 0..6 {
            let mut game = Klondike::deal(draw, seed);
            let mut rng = SplitMix64::new(seed);
            for _ in 0..60 {
                transfers += assert_legal_moves_equal_accepted_moves(&game);
                let legal = game.legal_moves();
                let pick = usize::try_from(rng.below(u64::try_from(legal.len()).unwrap())).unwrap();
                assert_eq!(game.apply(legal[pick]), Ok(()));
            }
        }
    }
    assert!(transfers > 0);
}
