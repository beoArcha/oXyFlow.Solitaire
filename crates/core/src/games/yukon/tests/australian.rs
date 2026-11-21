use super::*;
use crate::game::{Variant, deal};
use crate::session::Session;

const VARIANT: Variant = Variant::Yukon(YukonRules::Australian);

fn australian() -> Yukon {
    Yukon::empty(YukonRules::Australian)
}

fn waste() -> PileRef {
    PileRef::Waste
}

#[test]
fn the_deal_is_four_face_up_cards_in_seven_columns_and_a_stock_of_24() {
    let game = Yukon::deal(YukonRules::Australian, 7);
    for index in 0..7u8 {
        assert_eq!(
            face_up_of(&game, tab(index)),
            vec![true; 4],
            "column {index}"
        );
    }
    assert_eq!(cards_of(&game, PileRef::Stock).len(), 24);
    assert!(cards_of(&game, waste()).is_empty());
    assert!((0..4u8).all(|index| cards_of(&game, found(index)).is_empty()));
    let plan = YukonRules::Australian.deal_plan();
    let dealt: usize = plan.down.iter().chain(&plan.up).sum::<usize>() + plan.stock;
    assert_eq!(dealt, super::super::DECK_SIZE);
}

#[test]
fn the_same_seed_gives_the_same_australian_deal() {
    assert_eq!(deal(VARIANT, 99).piles(), deal(VARIANT, 99).piles());
    assert_ne!(deal(VARIANT, 99).piles(), deal(VARIANT, 100).piles());
}

#[test]
fn the_stock_counter_shows_the_single_pass() {
    let status = Yukon::deal(YukonRules::Australian, 3).stock_status();
    let status = status.unwrap();
    assert_eq!(status.recycles, 0);
    assert_eq!(status.recycles_left, Some(0));
    assert_eq!(status.deals_left, None);
    assert!(VARIANT.has_stock());
}

#[test]
fn turning_the_stock_moves_its_top_card_to_the_waste() {
    let mut game = australian();
    game.stock = cards("2C 3D 4H");
    assert_accepted(&mut game, Move::Stock);
    assert_eq!(cards_of(&game, waste()), cards("4H"));
    assert_eq!(cards_of(&game, PileRef::Stock), cards("2C 3D"));
    assert_accepted(&mut game, Move::Stock);
    assert_eq!(cards_of(&game, waste()), cards("4H 3D"));
}

#[test]
fn the_stock_is_dealt_through_once_and_never_redealt() {
    let mut game = australian();
    game.stock = cards("2C 3D");
    assert_accepted(&mut game, Move::Stock);
    assert_accepted(&mut game, Move::Stock);
    assert_rejected(&game, Move::Stock, MoveError::StockEmpty);
    assert_eq!(cards_of(&game, waste()), cards("3D 2C"));
}

#[test]
fn the_waste_top_goes_to_a_column_when_it_fits() {
    let mut game = australian();
    game.waste = cards("9C 6H");
    game.columns[0] = column("", "KD 7H");
    game.columns[1] = column("", "7S");
    assert_rejected(
        &game,
        transfer(waste(), tab(1), 1),
        MoveError::TargetRejects,
    );
    assert_accepted(&mut game, transfer(waste(), tab(0), 1));
    assert_eq!(cards_of(&game, tab(0)), cards("KD 7H 6H"));
    assert_eq!(cards_of(&game, waste()), cards("9C"));
}

#[test]
fn the_waste_top_goes_to_a_foundation() {
    let mut game = australian();
    game.waste = cards("AS");
    assert_accepted(&mut game, transfer(waste(), found(0), 1));
    assert_eq!(cards_of(&game, found(0)), cards("AS"));
    assert!(cards_of(&game, waste()).is_empty());
    assert_rejected(
        &game,
        transfer(waste(), found(1), 1),
        MoveError::EmptySource(waste()),
    );
}

#[test]
fn nothing_goes_onto_the_waste_or_the_stock() {
    let mut game = australian();
    game.waste = cards("9C");
    game.columns[0] = column("", "8C");
    assert_rejected(
        &game,
        transfer(tab(0), waste(), 1),
        MoveError::TargetRejects,
    );
    assert_rejected(
        &game,
        transfer(tab(0), PileRef::Stock, 1),
        MoveError::NoSuchPile(PileRef::Stock),
    );
}

#[test]
fn a_group_moves_when_its_bottom_card_fits_in_the_same_suit() {
    let mut game = australian();
    game.columns[0] = column("", "KC 5S 4D");
    game.columns[1] = column("", "6S");
    game.columns[2] = column("", "6H");
    assert_rejected(&game, transfer(tab(0), tab(2), 2), MoveError::TargetRejects);
    assert_accepted(&mut game, transfer(tab(0), tab(1), 2));
    assert_eq!(cards_of(&game, tab(1)), cards("6S 5S 4D"));
    assert_eq!(cards_of(&game, tab(0)), cards("KC"));
}

#[test]
fn an_empty_column_takes_only_a_king_or_a_group_with_a_king_at_the_bottom() {
    let mut game = australian();
    game.columns[0] = column("", "QS KH 3C");
    game.columns[1] = column("", "9D");
    assert_rejected(&game, transfer(tab(1), tab(2), 1), MoveError::TargetRejects);
    assert_rejected(&game, transfer(tab(0), tab(2), 1), MoveError::TargetRejects);
    assert_accepted(&mut game, transfer(tab(0), tab(2), 2));
    assert_eq!(cards_of(&game, tab(2)), cards("KH 3C"));
    game.waste = cards("KS");
    assert_accepted(&mut game, transfer(waste(), tab(3), 1));
}

#[test]
fn the_other_variants_still_have_no_waste_pile() {
    let game = Yukon::deal(YukonRules::Russian, 3);
    assert!(game.piles().iter().all(|p| p.pile != waste()));
    assert_rejected(
        &game,
        transfer(waste(), tab(0), 1),
        MoveError::NoSuchPile(waste()),
    );
}

#[test]
fn undo_takes_a_stock_turn_back() {
    let mut session = Session::new(VARIANT, 5);
    let dealt = session.game().piles();
    assert_eq!(session.apply(Move::Stock), Ok(()));
    assert_ne!(session.game().piles(), dealt);
    assert!(session.undo());
    assert_eq!(session.game().piles(), dealt);
    assert!(session.redo());
}

#[test]
fn a_game_in_progress_survives_a_storage_round_trip_with_stock_and_waste() {
    let mut session = Session::new(VARIANT, 11);
    for _ in 0..5 {
        assert_eq!(session.apply(Move::Stock), Ok(()));
    }
    let hint = session.game().hint().unwrap();
    assert_eq!(session.apply(hint), Ok(()));
    let json = serde_json::to_string(&session.record()).unwrap();
    let record = serde_json::from_str(&json).unwrap();
    let restored = Session::restore(record).unwrap();
    assert_eq!(restored.game().piles(), session.game().piles());
    assert!(restored.game().stock_status().is_some());
    assert!(!cards_of_session(&restored, PileRef::Stock).is_empty());
}

fn cards_of_session(session: &Session, pile: PileRef) -> Vec<Card> {
    let piles = session.game().piles();
    let snapshot = piles.into_iter().find(|p| p.pile == pile).unwrap();
    snapshot.cards.iter().map(|c| c.card).collect()
}

#[test]
fn the_forced_finish_waits_for_an_empty_stock_and_waste() {
    let mut game = australian();
    foundations_up_to(&mut game, 12);
    game.columns[0] = column("", "KC");
    game.columns[1] = column("", "KD");
    game.columns[2] = column("", "KH");
    game.columns[3] = column("", "KS");
    assert!(game.forced_finish().is_some());
    game.waste = cards("2C");
    game.foundations[0].pop();
    assert_eq!(game.forced_finish(), None);
    game.waste.clear();
    game.stock = cards("2C");
    assert_eq!(game.forced_finish(), None);
}

#[test]
fn the_stock_turn_is_the_hint_when_nothing_better_exists() {
    let mut game = australian();
    game.stock = cards("2C AS");
    assert_eq!(game.hint(), Some(Move::Stock));
    assert_eq!(game.apply(Move::Stock), Ok(()));
    assert_eq!(game.hint(), Some(transfer(waste(), found(0), 1)));
}

#[test]
fn the_waste_onto_a_column_ranks_after_the_column_moves() {
    let mut game = australian();
    game.stock = cards("2C");
    game.waste = cards("6H");
    game.columns[0] = column("", "7H");
    assert_eq!(game.hint(), Some(transfer(waste(), tab(0), 1)));
    game.columns[1] = column("", "3S");
    game.columns[2] = column("", "4S");
    assert_eq!(game.hint(), Some(transfer(tab(1), tab(2), 1)));
}

#[test]
fn following_the_hints_never_repeats_a_position() {
    for seed in 0..60 {
        let mut game = Yukon::deal(YukonRules::Australian, seed);
        let mut seen = vec![game.piles()];
        while let Some(hint) = game.hint() {
            assert_eq!(game.apply(hint), Ok(()), "seed {seed}: {hint:?}");
            let piles = game.piles();
            assert!(!seen.contains(&piles), "seed {seed}: a position came back");
            seen.push(piles);
        }
    }
}
