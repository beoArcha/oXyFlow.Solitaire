//! The state of the seed screen: which page each key opens, what each page starts, and how Esc steps
//! back. The levels read a fake table, so the outcome of a curated seed is known.

use ratatui::crossterm::event::{KeyCode, KeyEvent};

use super::seed_played::Played;
use super::seed_screen::{Outcome, SeedScreen};
use crate::action::Action;
use oxysol_core::curated::Difficulty;
use oxysol_core::{DrawCount, GameOptions, StockPasses, Variant};

const DEFAULT: GameOptions = GameOptions::DEFAULT;

/// Golf with the wrap rule, which changes the rules of Golf only.
const GOLF_WRAP: GameOptions = GameOptions {
    golf_wrap: true,
    ..GameOptions::DEFAULT
};

/// Klondike with one stock pass, which leaves Golf at the standard rules.
const KLONDIKE_ONE_PASS: GameOptions = GameOptions {
    klondike_passes: StockPasses::One,
    ..GameOptions::DEFAULT
};

fn fake(variant: Variant, level: Difficulty) -> &'static [u64] {
    match (variant, level) {
        (Variant::Golf, Difficulty::VeryEasy) => &[1, 2, 3],
        (Variant::Golf, Difficulty::Hard) => &[11, 12, 13],
        _ => &[],
    }
}

fn press_with(screen: &mut SeedScreen, action: Action, options: GameOptions) -> Outcome {
    screen.handle(&KeyEvent::from(KeyCode::Null), Some(action), options)
}

fn press(screen: &mut SeedScreen, action: Action) -> Outcome {
    press_with(screen, action, DEFAULT)
}

fn type_digits(screen: &mut SeedScreen, digits: &str) {
    for c in digits.chars() {
        screen.handle(&KeyEvent::from(KeyCode::Char(c)), None, DEFAULT);
    }
}

fn enter_with(screen: &mut SeedScreen, options: GameOptions) -> Outcome {
    let key = KeyEvent::from(KeyCode::Enter);
    screen.handle(&key, Some(Action::Select), options)
}

fn enter(screen: &mut SeedScreen) -> Outcome {
    enter_with(screen, DEFAULT)
}

fn choose_levels_with(screen: &mut SeedScreen, options: GameOptions) {
    for _ in 0..3 {
        press_with(screen, Action::Down, options);
    }
    enter_with(screen, options);
}

fn choose_levels(screen: &mut SeedScreen) {
    choose_levels_with(screen, DEFAULT);
}

#[test]
fn the_levels_do_not_open_under_options_changed_for_the_variant() {
    let mut screen = SeedScreen::with_lookup(Variant::Golf, fake, Played::default());
    choose_levels_with(&mut screen, GOLF_WRAP);
    assert_eq!(enter_with(&mut screen, GOLF_WRAP), Outcome::Stay);
    assert_eq!(screen.back(), Outcome::Leave);
}

#[test]
fn the_levels_stay_offered_under_options_of_another_game() {
    let mut screen = SeedScreen::with_lookup(Variant::Golf, fake, Played::default());
    choose_levels_with(&mut screen, KLONDIKE_ONE_PASS);
    assert_eq!(
        enter_with(&mut screen, KLONDIKE_ONE_PASS),
        Outcome::Start {
            variant: Variant::Golf,
            seed: 1,
        }
    );
}

#[test]
fn the_first_option_types_a_seed_and_submitting_it_starts_that_game() {
    let variant = Variant::Klondike(DrawCount::Three);
    let mut screen = SeedScreen::with_lookup(variant, fake, Played::default());
    assert_eq!(enter(&mut screen), Outcome::Stay);
    type_digits(&mut screen, "42");
    assert_eq!(enter(&mut screen), Outcome::Start { variant, seed: 42 });
}

#[test]
fn the_typing_page_captures_text_and_the_options_do_not() {
    let mut screen = SeedScreen::with_lookup(Variant::Golf, fake, Played::default());
    assert!(!screen.captures_text());
    enter(&mut screen);
    assert!(screen.captures_text());
}

#[test]
fn the_levels_start_the_chosen_seed_of_the_variant() {
    let mut screen = SeedScreen::with_lookup(Variant::Golf, fake, Played::default());
    choose_levels(&mut screen);
    assert_eq!(
        enter(&mut screen),
        Outcome::Start {
            variant: Variant::Golf,
            seed: 1,
        }
    );
}

#[test]
fn the_levels_list_every_curated_seed_in_order() {
    let mut screen = SeedScreen::with_lookup(Variant::Golf, fake, Played::default());
    choose_levels(&mut screen);
    press(&mut screen, Action::Down);
    press(&mut screen, Action::Down);
    press(&mut screen, Action::Down);
    assert_eq!(
        enter(&mut screen),
        Outcome::Start {
            variant: Variant::Golf,
            seed: 11,
        }
    );
}

#[test]
fn a_variant_without_curated_seeds_starts_nothing_from_its_levels() {
    let mut screen = SeedScreen::with_lookup(Variant::FreeCell, fake, Played::default());
    choose_levels(&mut screen);
    assert_eq!(enter(&mut screen), Outcome::Stay);
    assert_eq!(press(&mut screen, Action::Down), Outcome::Stay);
    assert_eq!(enter(&mut screen), Outcome::Stay);
}

#[test]
fn back_steps_out_one_page_then_leaves_from_the_options() {
    let mut screen = SeedScreen::with_lookup(Variant::Golf, fake, Played::default());
    choose_levels(&mut screen);
    assert_eq!(screen.back(), Outcome::Stay);
    assert_eq!(screen.back(), Outcome::Leave);
}

#[test]
fn back_from_the_typing_page_returns_to_the_options() {
    let mut screen = SeedScreen::with_lookup(Variant::Golf, fake, Played::default());
    enter(&mut screen);
    assert_eq!(screen.back(), Outcome::Stay);
    assert!(!screen.captures_text());
    assert_eq!(screen.back(), Outcome::Leave);
}

#[test]
fn a_refused_seed_stays_on_the_typing_page() {
    let mut screen = SeedScreen::with_lookup(Variant::Golf, fake, Played::default());
    enter(&mut screen);
    assert_eq!(enter(&mut screen), Outcome::Stay);
    assert!(screen.captures_text());
}
