//! The state of the seed screen: which page each key opens, what each page starts, and how Esc steps
//! back. The levels read a fake table, so the outcome of a curated seed is known.

use ratatui::crossterm::event::{KeyCode, KeyEvent};

use super::seed_played::Played;
use super::seed_screen::{Outcome, SeedScreen};
use crate::action::Action;
use oxysol_core::curated::Difficulty;
use oxysol_core::{DrawCount, Variant};

fn fake(variant: Variant, level: Difficulty) -> &'static [u64] {
    match (variant, level) {
        (Variant::Golf, Difficulty::VeryEasy) => &[1, 2, 3],
        (Variant::Golf, Difficulty::Hard) => &[11, 12, 13],
        _ => &[],
    }
}

fn press(screen: &mut SeedScreen, action: Action) -> Outcome {
    screen.handle(&KeyEvent::from(KeyCode::Null), Some(action))
}

fn type_digits(screen: &mut SeedScreen, digits: &str) {
    for c in digits.chars() {
        screen.handle(&KeyEvent::from(KeyCode::Char(c)), None);
    }
}

fn enter(screen: &mut SeedScreen) -> Outcome {
    let key = KeyEvent::from(KeyCode::Enter);
    screen.handle(&key, Some(Action::Select))
}

fn choose_levels(screen: &mut SeedScreen) {
    for _ in 0..3 {
        press(screen, Action::Down);
    }
    enter(screen);
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
