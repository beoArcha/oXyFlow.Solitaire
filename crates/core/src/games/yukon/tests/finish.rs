use super::*;

fn play(game: &mut Yukon, moves: &[Move]) {
    for &mv in moves {
        assert_eq!(game.apply(mv), Ok(()), "{mv:?}");
    }
}

#[test]
fn an_open_table_whose_tops_feed_the_foundations_finishes() {
    let mut game = table();
    foundations_up_to(&mut game, 10);
    game.columns[0] = column("", "KC QD JC");
    game.columns[1] = column("", "KD QC JD");
    game.columns[2] = column("", "KH QS JH");
    game.columns[3] = column("", "KS QH JS");
    let moves = game.forced_finish().unwrap();
    assert_eq!(moves.len(), 12);
    play(&mut game, &moves);
    assert!(game.is_won());
}

#[test]
fn a_face_down_card_means_no_forced_finish() {
    let mut game = table();
    foundations_up_to(&mut game, 11);
    game.foundations[3].pop();
    game.columns[0] = column("QS", "KC KD KH");
    game.columns[1] = column("", "KS");
    assert_eq!(game.forced_finish(), None);
}

#[test]
fn a_card_buried_under_a_higher_one_of_its_suit_means_no_forced_finish() {
    let mut game = table();
    foundations_up_to(&mut game, 11);
    game.columns[0] = column("", "QC KC");
    game.columns[1] = column("", "QD KD QH KH QS KS");
    assert_eq!(game.forced_finish(), None);
}

#[test]
fn a_fresh_deal_has_no_forced_finish() {
    for seed in 0..10 {
        assert_eq!(Yukon::deal(YukonRules::Classic, seed).forced_finish(), None);
    }
}

#[test]
fn hinted_play_survives_undo_and_a_restore_from_the_record() {
    use crate::game::Variant;
    use crate::session::Session;
    let mut session = Session::new(Variant::Yukon(YukonRules::Classic), 5);
    while let Some(hint) = session.game().hint() {
        assert_eq!(session.apply(hint), Ok(()));
    }
    assert!(session.moves_made() > 0);
    let restored = Session::restore(session.record()).unwrap();
    assert_eq!(restored.game().piles(), session.game().piles());
    let before = session.game().piles();
    let mut undone = Session::restore(session.record()).unwrap();
    assert!(undone.undo());
    assert!(undone.redo());
    assert_eq!(undone.game().piles(), before);
}
