use oxysol_core::{DrawCount, PyramidRules, StockStatus, SuitCount, Variant};
use ratatui::crossterm::event::KeyCode;

use super::*;
use crate::glyphs::is_wgl4;
use crate::i18n::{Locale, set_locale, t};
use crate::screens::board::counters::{stock_text, stock_text_of};
use crate::settings::Settings;
use crate::storage::memory::MemoryStorage;
use crate::test_support::PLAIN;

const KLONDIKE: Variant = Variant::Klondike(DrawCount::One);
const PYRAMID: Variant = Variant::Pyramid(PyramidRules::Relaxed);
const SEED: u64 = 5;

fn now_row(h: &mut Harness) -> String {
    h.lines()
        .into_iter()
        .map(|line| line.trim().to_owned())
        .find(|line| line.starts_with("Now:"))
        .unwrap_or_default()
}

fn header_row(h: &mut Harness) -> String {
    h.lines().into_iter().next().unwrap_or_default()
}

fn draw_stock(h: &mut Harness, times: usize) {
    for _ in 0..times {
        h.type_char('s');
    }
}

fn first_play(h: &mut Harness) {
    let Some((from, to, _)) = first_transfer(h) else {
        unreachable!("seed 5 has a transfer")
    };
    drag(h, from, to);
}

fn assert_counter(h: &mut Harness, expected: &str) {
    let header = header_row(h);
    assert!(
        header.contains(expected),
        "{header:?} should contain {expected:?}"
    );
    let now = now_row(h);
    assert!(
        !now.contains(expected),
        "{now:?} must not repeat {expected:?}"
    );
}

#[test]
fn the_header_counts_moves_through_play_undo_redo_and_restart() {
    let mut h = start(KLONDIKE, SEED);
    assert!(header_row(&mut h).contains("Moves: 0"));
    first_play(&mut h);
    assert!(header_row(&mut h).contains("Moves: 1"));
    h.type_char('u');
    assert!(header_row(&mut h).contains("Moves: 0"));
    h.type_char('r');
    assert!(header_row(&mut h).contains("Moves: 1"));
    first_play(&mut h);
    assert!(header_row(&mut h).contains("Moves: 2"));
    h.app.apply(Effect::Restart);
    assert!(header_row(&mut h).contains("Moves: 0"));
}

#[test]
fn the_win_dialog_shows_the_time_and_the_moves() {
    let mut h = start(KLONDIKE, SEED);
    first_play(&mut h);
    h.app.apply(Effect::Open(crate::app::Modal::win()));
    let text = h.text();
    assert!(text.contains("Time: 0:00"), "{text}");
    assert!(text.contains("Moves: 1"), "{text}");
}

#[test]
fn klondike_counts_how_often_the_waste_was_turned_over() {
    let mut h = start(KLONDIKE, SEED);
    assert_counter(&mut h, "Turned over: 0");
    draw_stock(&mut h, 24);
    assert_counter(&mut h, "Turned over: 0");
    draw_stock(&mut h, 1);
    assert_counter(&mut h, "Turned over: 1");
    h.type_char('u');
    assert_counter(&mut h, "Turned over: 0");
}

#[test]
fn pyramid_names_the_pass_and_the_last_pass() {
    let mut h = start(PYRAMID, 3);
    assert_counter(&mut h, "Pass 1 of 3");
    draw_stock(&mut h, 25);
    assert_counter(&mut h, "Pass 2 of 3");
    draw_stock(&mut h, 25);
    assert_counter(&mut h, "Last pass (3 of 3)");
}

#[test]
fn a_classic_pyramid_has_one_pass_and_says_so_when_it_is_used_up() {
    let mut h = start(Variant::Pyramid(PyramidRules::Classic), 3);
    assert_counter(&mut h, "Last pass (1 of 1)");
    draw_stock(&mut h, 24);
    assert_counter(&mut h, "No passes left");
}

#[test]
fn spider_counts_the_deals_left() {
    let mut h = start(Variant::Spider(SuitCount::One), 3);
    assert_counter(&mut h, "Deals left: 5");
    draw_stock(&mut h, 1);
    assert_counter(&mut h, "Deals left: 4");
    h.type_char('u');
    assert_counter(&mut h, "Deals left: 5");
}

#[test]
fn freecell_has_no_stock_counter() {
    let h = start(Variant::FreeCell, 3);
    assert_eq!(stock_text(session(&h)), None);
}

#[test]
fn the_stock_counter_is_shown_once_in_the_game_information_even_in_a_tall_window() {
    let mut h = Harness::with(MemoryStorage::new(), TRUE_COLOR, 120, 70);
    h.app.apply(Effect::Start {
        variant: PYRAMID,
        seed: Some(3),
        undo_allowed: true,
    });
    let lines = h.lines();
    let counted = lines
        .iter()
        .filter(|line| line.contains("Pass 1 of 3"))
        .count();
    assert_eq!(counted, 1, "{lines:?}");
    assert!(!now_row(&mut h).contains("Pass 1 of 3"));
}

#[test]
fn the_counter_texts_follow_the_status_of_the_stock() {
    let status = |recycles, recycles_left, deals_left| StockStatus {
        recycles,
        recycles_left,
        deals_left,
    };
    assert_eq!(stock_text_of(&status(4, None, None), 3), "Turned over: 4");
    assert_eq!(stock_text_of(&status(0, Some(2), None), 24), "Pass 1 of 3");
    assert_eq!(
        stock_text_of(&status(2, Some(0), None), 5),
        "Last pass (3 of 3)"
    );
    assert_eq!(
        stock_text_of(&status(2, Some(0), None), 0),
        "No passes left"
    );
    assert_eq!(
        stock_text_of(&status(0, Some(0), Some(3)), 30),
        "Deals left: 3"
    );
}

#[test]
fn no_undo_rejects_undo_and_redo_with_a_message() {
    let mut h = start_no_undo(KLONDIKE, SEED);
    first_play(&mut h);
    let played = session(&h).game().piles();
    h.type_char('u');
    assert_eq!(status(&h), Some("board.status.undo_off"));
    assert_eq!(session(&h).game().piles(), played);
    assert_eq!(cues(&h).last(), Some(&SoundCue::Illegal));
    assert!(h.text().contains("Undo is off in this game."));
    h.type_char('r');
    assert_eq!(status(&h), Some("board.status.undo_off"));
    assert_eq!(moves_made(&h), 1);
}

#[test]
fn no_undo_hides_undo_and_redo_from_every_hint_and_shows_a_badge() {
    let mut h = start_no_undo(KLONDIKE, SEED);
    first_play(&mut h);
    let text = h.text();
    assert!(header_row(&mut h).contains("No undo"));
    assert!(!text.contains(": undo"), "{text}");
    assert!(!text.contains("Undo "), "{text}");
    let mut with_undo = start(KLONDIKE, SEED);
    first_play(&mut with_undo);
    assert!(with_undo.text().contains("u: undo"));
    assert!(!header_row(&mut with_undo).contains("No undo"));
}

#[test]
fn the_guide_panel_drops_undo_and_offers_copying_the_seed() {
    let mut h = Harness::with(MemoryStorage::new(), TRUE_COLOR, 140, 40);
    h.app.apply(Effect::Start {
        variant: KLONDIKE,
        seed: Some(SEED),
        undo_allowed: false,
    });
    let text = h.text();
    assert!(!text.contains("undo / redo"), "{text}");
    assert!(text.contains("copy seed"), "{text}");
    let mut with_undo = Harness::with(MemoryStorage::new(), TRUE_COLOR, 140, 40);
    with_undo.app.apply(Effect::Start {
        variant: KLONDIKE,
        seed: Some(SEED),
        undo_allowed: true,
    });
    assert!(with_undo.text().contains("undo / redo"));
}

#[test]
fn a_game_without_undo_stays_that_way_in_storage_and_after_a_resume() {
    let mut h = start_no_undo(KLONDIKE, SEED);
    assert!(!h.storage.borrow().history[0].undo_allowed);
    first_play(&mut h);
    let saved = h.storage.borrow().game.clone();
    assert!(saved.as_ref().is_some_and(|game| !game.undo_allowed));
    let storage = MemoryStorage {
        game: saved,
        ..MemoryStorage::default()
    };
    let mut resumed = Harness::with(storage, TRUE_COLOR, 120, 40);
    resumed.app.apply(Effect::Resume);
    assert!(!resumed.app.board().undo_allowed());
    resumed.type_char('u');
    assert_eq!(status(&resumed), Some("board.status.undo_off"));
    assert_eq!(moves_made(&resumed), 1);
}

#[test]
fn a_new_game_from_the_board_keeps_the_undo_mode_of_the_game() {
    let mut h = start_no_undo(KLONDIKE, SEED);
    h.type_char('n');
    h.press(KeyCode::Left);
    h.press(KeyCode::Enter);
    assert!(!h.app.board().undo_allowed());
    assert_eq!(h.storage.borrow().history.len(), 2);
    assert!(!h.storage.borrow().history[1].undo_allowed);
}

#[test]
fn the_copy_key_puts_the_seed_on_the_clipboard_and_announces_it() {
    let mut h = start(KLONDIKE, SEED);
    h.type_char('c');
    assert_eq!(h.clipboard.borrow().copies, ["5"]);
    assert!(h.text().contains("Seed 5 copied"));
    h.type_char('s');
    assert!(!h.text().contains("Seed 5 copied"));
}

#[test]
fn a_failed_copy_still_shows_the_seed() {
    let mut h = start(KLONDIKE, SEED);
    h.clipboard.borrow_mut().failing = true;
    h.type_char('c');
    assert!(h.clipboard.borrow().copies.is_empty());
    assert!(h.text().contains("Could not copy the seed"));
    assert!(header_row(&mut h).contains("seed 5"));
}

#[test]
fn a_click_on_the_seed_in_the_header_copies_it() {
    let mut h = start(KLONDIKE, SEED);
    let header = header_row(&mut h);
    let Some(byte) = header.find("seed 5") else {
        unreachable!("the header names the seed: {header}")
    };
    let column = u16::try_from(header[..byte].chars().count() + "seed ".len()).unwrap_or(0);
    h.click(column, 0);
    assert_eq!(h.clipboard.borrow().copies, ["5"]);
    h.click(column + 20, 0);
    assert_eq!(h.clipboard.borrow().copies.len(), 1);
}

#[test]
fn the_board_draws_at_100_by_30_in_both_locales_with_only_wgl4_or_ascii_glyphs() {
    for locale in Locale::offered() {
        for caps in [TRUE_COLOR, PLAIN] {
            for variant in [KLONDIKE, PYRAMID] {
                let settings = Settings {
                    locale,
                    ..Settings::for_caps(&caps)
                };
                let storage = MemoryStorage {
                    settings: Some(settings),
                    ..MemoryStorage::default()
                };
                let mut h = Harness::with(storage, caps, 100, 30);
                h.app.apply(Effect::Start {
                    variant,
                    seed: Some(SEED),
                    undo_allowed: false,
                });
                let text = h.text();
                for ch in text.chars().filter(|ch| *ch != '\n') {
                    assert!(ch.is_ascii() || is_wgl4(ch), "{ch:?} in {locale:?}");
                    assert!(
                        caps.unicode || locale != Locale::EnUs || ch.is_ascii(),
                        "{ch:?} in ASCII mode"
                    );
                }
                let header = header_row(&mut h);
                assert!(header.contains(&SEED.to_string()), "{header}");
                set_locale(locale);
                let badge = t("board.header.no_undo").into_owned();
                assert!(header.contains(&badge), "{header}");
            }
        }
    }
    set_locale(Locale::EnUs);
}
