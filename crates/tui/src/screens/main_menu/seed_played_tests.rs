//! The played seeds of the seed screen, by keyboard: the newest-first order of one variant, the
//! random replay pick, and the message when no game was played. The mouse and the whole flow are
//! in `seed_flow_tests`.

use ratatui::crossterm::event::{KeyCode, KeyEvent};

use super::seed_played::Played;
use super::seed_screen::{Outcome, SeedScreen};
use crate::action::Action;
use crate::app::Zone;
use crate::storage::{GameCounters, GameId, GameRecord, Outcome as GameOutcome, StorageError};
use oxysol_core::{DrawCount, GameOptions, Variant};

const GOLF: Variant = Variant::Golf;
const KLONDIKE: Variant = Variant::Klondike(DrawCount::Three);

fn game(id: i64, variant: Variant, seed: u64) -> GameRecord {
    GameRecord {
        id: GameId(id),
        variant,
        seed,
        options: oxysol_core::GameOptions::DEFAULT,
        started_at_ms: id * 60_000,
        finished_at_ms: None,
        outcome: Some(GameOutcome::Won),
        elapsed_ms: Some(1_000),
        moves: Some(10),
        undo_allowed: true,
        proof: None,
        counters: GameCounters::default(),
    }
}

/// Golf games 10 (first) and 30 (last), with a Klondike game between them.
fn history() -> Vec<GameRecord> {
    vec![game(1, GOLF, 10), game(2, KLONDIKE, 20), game(3, GOLF, 30)]
}

const fn replay(variant: Variant, seed: u64, options: GameOptions, undo_allowed: bool) -> Outcome {
    Outcome::Replay {
        variant,
        seed,
        options,
        undo_allowed,
    }
}

fn played_golf() -> Played {
    Played::read(Ok(history()), Zone::Utc, 7).of_variant(GOLF)
}

fn screen_with(played: Played) -> SeedScreen {
    SeedScreen::with_lookup(GOLF, no_curated, played)
}

fn no_curated(_: Variant, _: oxysol_core::curated::Difficulty) -> &'static [u64] {
    &[]
}

fn press(screen: &mut SeedScreen, action: Action) -> Outcome {
    screen.handle(
        &KeyEvent::from(KeyCode::Null),
        Some(action),
        oxysol_core::GameOptions::DEFAULT,
    )
}

fn enter(screen: &mut SeedScreen) -> Outcome {
    screen.handle(
        &KeyEvent::from(KeyCode::Enter),
        Some(Action::Select),
        oxysol_core::GameOptions::DEFAULT,
    )
}

/// The options page with the played seeds chosen: one step down, then Enter.
fn open_played(screen: &mut SeedScreen) -> Outcome {
    press(screen, Action::Down);
    enter(screen)
}

/// The options page with the random replay chosen: two steps down, then Enter.
fn random_replay(screen: &mut SeedScreen) -> Outcome {
    press(screen, Action::Down);
    press(screen, Action::Down);
    enter(screen)
}

#[test]
fn the_played_seeds_of_a_variant_are_listed_newest_first() {
    let played = played_golf();
    assert_eq!(played.len(), 2);
    assert_eq!(played.game_at(0).map(|game| game.seed), Some(30));
    assert_eq!(played.game_at(1).map(|game| game.seed), Some(10));
    assert_eq!(played.game_at(2).map(|game| game.seed), None);
}

#[test]
fn choosing_a_played_seed_replays_that_game() {
    let mut screen = screen_with(played_golf());
    assert_eq!(open_played(&mut screen), Outcome::Stay);
    assert_eq!(
        enter(&mut screen),
        replay(GOLF, 30, GameOptions::DEFAULT, true)
    );
}

#[test]
fn down_moves_to_the_second_played_seed_before_it_starts() {
    let mut screen = screen_with(played_golf());
    open_played(&mut screen);
    press(&mut screen, Action::Down);
    assert_eq!(
        enter(&mut screen),
        replay(GOLF, 10, GameOptions::DEFAULT, true)
    );
}

#[test]
fn back_from_the_played_seeds_returns_to_the_options() {
    let mut screen = screen_with(played_golf());
    open_played(&mut screen);
    assert_eq!(screen.back(), Outcome::Stay);
    assert_eq!(screen.back(), Outcome::Leave);
}

#[test]
fn without_a_played_game_the_played_seeds_start_nothing() {
    let mut screen = screen_with(Played::default());
    assert_eq!(open_played(&mut screen), Outcome::Stay);
    assert_eq!(enter(&mut screen), Outcome::Stay);
    assert_eq!(screen.back(), Outcome::Leave);
}

#[test]
fn the_random_replay_starts_one_of_the_played_seeds() {
    let played = played_golf();
    let mut screen = screen_with(played.clone());
    let Outcome::Replay { variant, seed, .. } = random_replay(&mut screen) else {
        panic!("the random replay should start a played seed");
    };
    assert_eq!(variant, GOLF);
    assert!(
        played.game_at(0).map(|game| game.seed) == Some(seed)
            || played.game_at(1).map(|game| game.seed) == Some(seed)
    );
}

#[test]
fn the_random_replay_follows_the_fresh_seed_of_the_visit() {
    let first = Played::read(Ok(history()), Zone::Utc, 7).of_variant(GOLF);
    let again = Played::read(Ok(history()), Zone::Utc, 7).of_variant(GOLF);
    assert_eq!(
        first.random_game().map(|game| game.seed),
        again.random_game().map(|game| game.seed)
    );
}

#[test]
fn the_random_replay_without_a_played_game_starts_nothing() {
    let mut screen = screen_with(Played::default());
    assert_eq!(random_replay(&mut screen), Outcome::Stay);
    assert!(!screen.captures_text());
    assert_eq!(screen.back(), Outcome::Leave);
}

#[test]
fn an_unreadable_history_starts_nothing_from_either_entry() {
    let unreadable = Played::read(
        Err(StorageError::Backend {
            message: "test".to_owned(),
        }),
        Zone::Utc,
        7,
    );
    assert!(unreadable.is_unreadable());
    assert!(unreadable.random_game().map(|game| game.seed).is_none());
    let mut screen = screen_with(unreadable.of_variant(GOLF));
    assert_eq!(random_replay(&mut screen), Outcome::Stay);
    assert_eq!(open_played(&mut screen), Outcome::Stay);
}

#[test]
fn a_variant_with_no_games_has_no_random_replay_pick() {
    let played = Played::read(Ok(vec![game(1, KLONDIKE, 5)]), Zone::Utc, 7).of_variant(GOLF);
    assert!(played.is_empty());
    assert_eq!(played.random_game().map(|game| game.seed), None);
}

#[test]
fn a_played_game_replays_with_its_own_options_and_undo_mode() {
    let options = GameOptions {
        golf_wrap: true,
        ..GameOptions::DEFAULT
    };
    let mut record = game(4, GOLF, 40);
    record.options = options;
    record.undo_allowed = false;
    let played = Played::read(Ok(vec![record]), Zone::Utc, 7).of_variant(GOLF);
    let mut screen = screen_with(played.clone());
    open_played(&mut screen);
    assert_eq!(enter(&mut screen), replay(GOLF, 40, options, false));
    let mut screen = screen_with(played);
    assert_eq!(random_replay(&mut screen), replay(GOLF, 40, options, false));
}
