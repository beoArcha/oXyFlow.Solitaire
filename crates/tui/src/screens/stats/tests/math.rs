use oxysol_core::{SuitCount, Variant};

use super::{KLONDIKE, record};
use crate::screens::stats::summary::{has_several_modes, summarize};
use crate::storage::Outcome::{Abandoned, Won};

#[test]
fn counts_rate_best_time_fewest_moves_and_streaks_on_a_fixed_history() {
    let history = [
        record(1, KLONDIKE, Some(Won), Some(100_000), Some(80)),
        record(2, KLONDIKE, Some(Won), Some(60_000), Some(95)),
        record(3, KLONDIKE, Some(Abandoned), Some(10_000), Some(4)),
        record(4, KLONDIKE, Some(Won), Some(90_000), Some(70)),
        record(5, KLONDIKE, None, None, None),
    ];
    let summaries = summarize(&history);
    assert_eq!(summaries.len(), 1);
    let summary = &summaries[0];
    assert_eq!(summary.variant, KLONDIKE);
    assert_eq!((summary.played, summary.won), (4, 3));
    assert_eq!(summary.win_rate_percent(), 75);
    assert_eq!(summary.best_time_ms, Some(60_000));
    assert_eq!(summary.fewest_moves, Some(70));
    assert_eq!(summary.current_streak, 1);
    assert_eq!(summary.longest_streak, 2);
}

#[test]
fn the_win_rate_rounds_to_the_nearest_percent() {
    let one_of_three = [
        record(1, KLONDIKE, Some(Won), Some(1), Some(1)),
        record(2, KLONDIKE, Some(Abandoned), Some(1), Some(1)),
        record(3, KLONDIKE, Some(Abandoned), Some(1), Some(1)),
    ];
    assert_eq!(summarize(&one_of_three)[0].win_rate_percent(), 33);
    let two_of_three = [
        record(1, KLONDIKE, Some(Won), Some(1), Some(1)),
        record(2, KLONDIKE, Some(Won), Some(1), Some(1)),
        record(3, KLONDIKE, Some(Abandoned), Some(1), Some(1)),
    ];
    assert_eq!(summarize(&two_of_three)[0].win_rate_percent(), 67);
}

#[test]
fn an_abandoned_game_breaks_the_streak_and_a_lost_streak_is_zero() {
    let history = [
        record(1, KLONDIKE, Some(Won), Some(1), Some(1)),
        record(2, KLONDIKE, Some(Won), Some(1), Some(1)),
        record(3, KLONDIKE, Some(Won), Some(1), Some(1)),
        record(4, KLONDIKE, Some(Abandoned), Some(1), Some(1)),
    ];
    let summary = &summarize(&history)[0];
    assert_eq!(summary.current_streak, 0);
    assert_eq!(summary.longest_streak, 3);
}

#[test]
fn streaks_follow_the_deal_order_whatever_the_order_of_the_records() {
    let history = [
        record(4, KLONDIKE, Some(Won), Some(1), Some(1)),
        record(2, KLONDIKE, Some(Abandoned), Some(1), Some(1)),
        record(3, KLONDIKE, Some(Won), Some(1), Some(1)),
        record(1, KLONDIKE, Some(Won), Some(1), Some(1)),
    ];
    let summary = &summarize(&history)[0];
    assert_eq!(summary.current_streak, 2);
    assert_eq!(summary.longest_streak, 2);
}

#[test]
fn wins_without_a_move_count_are_skipped_for_the_fewest_moves() {
    let history = [
        record(1, KLONDIKE, Some(Won), Some(50_000), None),
        record(2, KLONDIKE, Some(Won), Some(70_000), Some(99)),
    ];
    let summary = &summarize(&history)[0];
    assert_eq!(summary.fewest_moves, Some(99));
    assert_eq!(summary.best_time_ms, Some(50_000));
    let only_old = [record(1, KLONDIKE, Some(Won), Some(50_000), None)];
    assert_eq!(summarize(&only_old)[0].fewest_moves, None);
}

#[test]
fn an_abandoned_game_never_sets_the_best_time_or_the_fewest_moves() {
    let history = [
        record(1, KLONDIKE, Some(Abandoned), Some(1_000), Some(1)),
        record(2, KLONDIKE, Some(Won), Some(70_000), Some(99)),
    ];
    let summary = &summarize(&history)[0];
    assert_eq!(summary.best_time_ms, Some(70_000));
    assert_eq!(summary.fewest_moves, Some(99));
}

#[test]
fn variants_are_listed_in_menu_order_and_open_games_alone_make_no_summary() {
    let spider = Variant::Spider(SuitCount::Two);
    let history = [
        record(1, Variant::FreeCell, Some(Won), Some(1), Some(1)),
        record(2, spider, Some(Abandoned), Some(1), Some(1)),
        record(3, KLONDIKE, Some(Won), Some(1), Some(1)),
        record(
            4,
            Variant::Pyramid(oxysol_core::PyramidRules::Classic),
            None,
            None,
            None,
        ),
    ];
    let order: Vec<Variant> = summarize(&history).iter().map(|s| s.variant).collect();
    assert_eq!(order, [KLONDIKE, spider, Variant::FreeCell]);
    assert!(summarize(&[]).is_empty());
}

#[test]
fn every_undo_mode_has_its_own_summary() {
    let mut no_undo = record(2, KLONDIKE, Some(Abandoned), Some(1), Some(1));
    no_undo.undo_allowed = false;
    let history = [record(1, KLONDIKE, Some(Won), Some(1), Some(1)), no_undo];
    assert!(has_several_modes(&history));
    let summaries = summarize(&history);
    assert_eq!(summaries.len(), 2);
    assert!(summaries[0].undo_allowed);
    assert_eq!(summaries[0].won, 1);
    assert!(!summaries[1].undo_allowed);
    assert_eq!(summaries[1].won, 0);
    assert!(!has_several_modes(&history[..1]));
}
