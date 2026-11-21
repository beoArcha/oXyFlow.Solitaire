use super::*;
use crate::game::{Variant, deal};
use crate::moves::MoveError::TargetRejects;

fn moosehide_table() -> Yukon {
    Yukon::empty(YukonRules::Moosehide)
}

#[test]
fn a_card_one_rank_lower_of_the_same_colour_but_another_suit_is_accepted() {
    let mut game = moosehide_table();
    game.columns[0] = column("", "6C");
    game.columns[1] = column("", "7S");
    assert_accepted(&mut game, transfer(tab(0), tab(1), 1));
    assert_eq!(cards_of(&game, tab(1)), cards("7S 6C"));
    assert!(cards_of(&game, tab(0)).is_empty());
}

#[test]
fn a_card_one_rank_lower_of_the_other_colour_and_suit_is_accepted() {
    let mut game = moosehide_table();
    game.columns[0] = column("", "6H");
    game.columns[1] = column("", "7S");
    assert_accepted(&mut game, transfer(tab(0), tab(1), 1));
    assert_eq!(cards_of(&game, tab(1)), cards("7S 6H"));
}

#[test]
fn a_card_of_the_same_suit_is_refused_even_one_rank_lower() {
    let mut game = moosehide_table();
    game.columns[0] = column("", "6S");
    game.columns[1] = column("", "7S");
    assert_rejected(&game, transfer(tab(0), tab(1), 1), TargetRejects);
}

#[test]
fn a_card_two_ranks_away_or_one_rank_higher_is_refused() {
    let mut game = moosehide_table();
    game.columns[0] = column("", "5H");
    game.columns[1] = column("", "7S");
    game.columns[2] = column("", "8D");
    assert_rejected(&game, transfer(tab(0), tab(1), 1), TargetRejects);
    assert_rejected(&game, transfer(tab(2), tab(1), 1), TargetRejects);
}

#[test]
fn a_group_moves_when_its_bottom_card_fits_whatever_lies_above_it() {
    let mut game = moosehide_table();
    game.columns[0] = column("", "6C 3S");
    game.columns[1] = column("", "7S");
    assert_accepted(&mut game, transfer(tab(0), tab(1), 2));
    assert_eq!(cards_of(&game, tab(1)), cards("7S 6C 3S"));
    assert!(cards_of(&game, tab(0)).is_empty());
}

#[test]
fn a_group_is_refused_when_its_bottom_card_has_the_suit_of_the_target() {
    let mut game = moosehide_table();
    game.columns[0] = column("", "6S 3C");
    game.columns[1] = column("", "7S");
    assert_rejected(&game, transfer(tab(0), tab(1), 2), TargetRejects);
}

#[test]
fn an_empty_column_takes_only_a_king_or_a_group_with_a_king_at_the_bottom() {
    let mut game = moosehide_table();
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
fn the_moosehide_deal_is_the_classic_layout() {
    assert_eq!(
        YukonRules::Moosehide.deal_plan(),
        YukonRules::Classic.deal_plan()
    );
    let game = Yukon::deal(YukonRules::Moosehide, 7);
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
fn the_same_seed_gives_the_same_moosehide_deal() {
    let variant = Variant::Yukon(YukonRules::Moosehide);
    assert_eq!(deal(variant, 99).piles(), deal(variant, 99).piles());
    assert_ne!(deal(variant, 99).piles(), deal(variant, 100).piles());
}

#[test]
fn a_card_that_already_rests_on_a_lower_card_of_another_suit_is_not_moved_back() {
    let mut game = moosehide_table();
    game.columns[0] = column("", "7S 6C");
    assert_eq!(game.hint(), None);
    game.columns[0] = column("", "6C");
    game.columns[1] = column("", "7S");
    assert_eq!(game.hint(), Some(transfer(tab(0), tab(1), 1)));
    assert_eq!(game.apply(transfer(tab(0), tab(1), 1)), Ok(()));
    assert_eq!(game.hint(), None);
}

#[test]
fn following_the_hints_never_repeats_a_position() {
    for seed in 0..60 {
        let mut game = Yukon::deal(YukonRules::Moosehide, seed);
        let mut seen = vec![game.piles()];
        while let Some(hint) = game.hint() {
            assert_eq!(game.apply(hint), Ok(()), "seed {seed}: {hint:?}");
            let piles = game.piles();
            assert!(!seen.contains(&piles), "seed {seed}: a position came back");
            seen.push(piles);
        }
    }
}
