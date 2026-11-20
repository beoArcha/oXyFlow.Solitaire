use super::*;
use crate::game::{Variant, deal};

#[test]
fn the_classic_deal_uses_the_whole_deck() {
    let plan = YukonRules::Classic.deal_plan();
    let dealt: usize = plan.down.iter().chain(&plan.up).sum();
    assert_eq!(dealt, super::super::DECK_SIZE);
    assert_eq!(plan.down.iter().sum::<usize>(), 21);
    assert_eq!(plan.up.iter().sum::<usize>(), 31);
}

#[test]
fn columns_hold_face_down_cards_under_five_face_up_cards() {
    let game = Yukon::deal(YukonRules::Classic, 7);
    for index in 0..7u8 {
        let faces = face_up_of(&game, tab(index));
        let down = usize::from(index);
        let up = if index == 0 { 1 } else { 5 };
        let expected: Vec<bool> = std::iter::repeat_n(false, down)
            .chain(std::iter::repeat_n(true, up))
            .collect();
        assert_eq!(faces, expected, "column {index}");
    }
    for index in 0..4u8 {
        assert!(cards_of(&game, found(index)).is_empty());
    }
}

#[test]
fn the_same_seed_gives_the_same_deal() {
    let variant = Variant::Yukon(YukonRules::Classic);
    assert_eq!(deal(variant, 99).piles(), deal(variant, 99).piles());
    assert_ne!(deal(variant, 99).piles(), deal(variant, 100).piles());
}

#[test]
fn there_is_no_stock() {
    let mut game = Yukon::deal(YukonRules::Classic, 3);
    assert_eq!(game.stock_status(), None);
    assert!(!game.legal_moves().contains(&Move::Stock));
    assert_rejected(&game, Move::Stock, MoveError::Unsupported);
    let remove = Move::Remove {
        first: tab(0),
        second: None,
    };
    assert_eq!(game.apply(remove), Err(MoveError::Unsupported));
    assert!(
        game.piles()
            .iter()
            .all(|pile| matches!(pile.pile, PileRef::Foundation(_) | PileRef::Tableau(_)))
    );
}

#[test]
fn the_hint_of_a_fresh_deal_is_a_legal_move() {
    for seed in 0..20 {
        let game = Yukon::deal(YukonRules::Classic, seed);
        let moves = game.legal_moves();
        if let Some(hint) = game.hint() {
            assert!(moves.contains(&hint), "seed {seed}: {hint:?}");
        }
    }
}
