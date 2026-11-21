use super::*;
use crate::game::{Variant, deal};
use crate::moves::MoveError::TargetRejects;

fn russian_table() -> Yukon {
    Yukon::empty(YukonRules::Russian)
}

#[test]
fn a_card_builds_down_in_the_same_suit() {
    let mut game = russian_table();
    game.columns[0] = column("", "7H");
    game.columns[1] = column("", "8H");
    assert_accepted(&mut game, transfer(tab(0), tab(1), 1));
    assert_eq!(cards_of(&game, tab(1)), cards("8H 7H"));
    assert!(cards_of(&game, tab(0)).is_empty());
}

#[test]
fn a_card_of_the_opposite_colour_but_another_suit_is_refused() {
    let mut game = russian_table();
    game.columns[0] = column("", "7C");
    game.columns[1] = column("", "8H");
    game.columns[2] = column("", "7D");
    game.columns[3] = column("", "8S");
    assert_rejected(&game, transfer(tab(0), tab(1), 1), TargetRejects);
    assert_rejected(&game, transfer(tab(2), tab(1), 1), TargetRejects);
    assert_rejected(&game, transfer(tab(2), tab(3), 1), TargetRejects);
}

#[test]
fn a_card_of_the_same_suit_but_not_one_rank_lower_is_refused() {
    let mut game = russian_table();
    game.columns[0] = column("", "6H");
    game.columns[1] = column("", "8H");
    assert_rejected(&game, transfer(tab(0), tab(1), 1), TargetRejects);
}

#[test]
fn a_group_moves_when_its_bottom_card_is_one_rank_lower_in_the_same_suit() {
    let mut game = russian_table();
    game.columns[0] = column("", "6H 5S 2C");
    game.columns[1] = column("", "6S");
    game.columns[2] = column("", "6H");
    assert_rejected(&game, transfer(tab(0), tab(2), 2), TargetRejects);
    assert_accepted(&mut game, transfer(tab(0), tab(1), 2));
    assert_eq!(cards_of(&game, tab(1)), cards("6S 5S 2C"));
    assert_eq!(cards_of(&game, tab(0)), cards("6H"));
}

#[test]
fn an_empty_column_takes_only_a_king_or_a_group_with_a_king_at_the_bottom() {
    let mut game = russian_table();
    game.columns[0] = column("", "QH");
    game.columns[1] = column("", "KS QD");
    game.columns[2] = column("", "KH");
    assert_rejected(&game, transfer(tab(0), tab(3), 1), TargetRejects);
    assert_accepted(&mut game, transfer(tab(1), tab(3), 2));
    assert_eq!(cards_of(&game, tab(3)), cards("KS QD"));
    assert_accepted(&mut game, transfer(tab(2), tab(4), 1));
    assert_eq!(cards_of(&game, tab(4)), cards("KH"));
}

#[test]
fn the_russian_deal_is_the_classic_layout() {
    assert_eq!(
        YukonRules::Russian.deal_plan(),
        YukonRules::Classic.deal_plan()
    );
    let game = Yukon::deal(YukonRules::Russian, 7);
    for index in 0..7u8 {
        let faces = face_up_of(&game, tab(index));
        let down = usize::from(index);
        let up = if index == 0 { 1 } else { 5 };
        let expected: Vec<bool> = std::iter::repeat_n(false, down)
            .chain(std::iter::repeat_n(true, up))
            .collect();
        assert_eq!(faces, expected, "column {index}");
    }
}

#[test]
fn the_same_seed_gives_the_same_russian_deal() {
    let variant = Variant::Yukon(YukonRules::Russian);
    assert_eq!(deal(variant, 99).piles(), deal(variant, 99).piles());
    assert_ne!(deal(variant, 99).piles(), deal(variant, 100).piles());
}

#[test]
fn the_hint_of_a_fresh_russian_deal_is_a_legal_move() {
    for seed in 0..20 {
        let game = Yukon::deal(YukonRules::Russian, seed);
        if let Some(hint) = game.hint() {
            assert!(game.legal_moves().contains(&hint), "seed {seed}: {hint:?}");
        }
    }
}
