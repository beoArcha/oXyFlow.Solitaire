use oxysol_core::Variant;
use ratatui::crossterm::event::KeyCode;

use super::support::{KLONDIKE, STATISTICS_ITEM, click_text, ctrl_c, open_stats, three_games};
use crate::app::Screen;
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, PLAIN, TRUE_COLOR};

#[test]
fn it_opens_from_the_main_menu_by_keyboard_and_by_mouse() {
    let h = open_stats(MemoryStorage::new(), 100, 30);
    assert_eq!(h.app.screen(), Screen::Stats);
    let mut h = Harness::with(MemoryStorage::new(), TRUE_COLOR, 100, 30);
    click_text(&mut h, "Statistics");
    assert_eq!(h.app.screen(), Screen::Stats);
    assert!(h.text().contains("Statistics"));
}

#[test]
fn an_empty_history_says_so_and_offers_only_back() {
    let mut h = open_stats(MemoryStorage::new(), 100, 30);
    let text = h.text();
    assert!(text.contains("No games played yet"), "{text}");
    assert!(!text.contains("Copy seed ]"));
    h.press(KeyCode::Enter);
    h.type_char('c');
    assert_eq!(h.app.screen(), Screen::Stats);
    assert!(h.clipboard.borrow().copies.is_empty());
    click_text(&mut h, "Back");
    assert_eq!(h.app.screen(), Screen::MainMenu);
}

#[test]
fn a_failing_history_read_shows_a_message_not_an_empty_history() {
    let mut h = Harness::with(three_games(), TRUE_COLOR, 100, 30);
    h.storage.borrow_mut().failing = true;
    for _ in 0..STATISTICS_ITEM {
        h.press(KeyCode::Down);
    }
    h.press(KeyCode::Enter);
    assert!(h.text().contains("could not be read"));
}

#[test]
fn the_history_is_listed_newest_first_with_its_columns_and_a_summary() {
    let mut h = open_stats(three_games(), 100, 30);
    let lines = h.lines();
    let position = |needle: &str| lines.iter().position(|line| line.contains(needle));
    let (newest, middle, oldest) = (
        position("1970-01-01 00:03").unwrap_or(usize::MAX),
        position("1970-01-01 00:02").unwrap_or(usize::MAX),
        position("1970-01-01 00:01").unwrap_or(usize::MAX),
    );
    assert!(newest < middle && middle < oldest, "{lines:#?}");
    let text = h.text();
    for part in [
        "Date",
        "Seed",
        "Abandoned",
        "1:15",
        "Klondike, Draw 1",
        "FreeCell",
        "Win rate",
        "100%",
        "Game 1 of 3",
    ] {
        assert!(text.contains(part), "missing {part:?}\n{text}");
    }
}

#[test]
fn the_selected_row_is_marked_without_color_in_both_glyph_sets() {
    for caps in [TRUE_COLOR, PLAIN] {
        let mut h = Harness::with(three_games(), caps, 100, 30);
        for _ in 0..STATISTICS_ITEM {
            h.press(KeyCode::Down);
        }
        h.press(KeyCode::Enter);
        let left = if caps.unicode { "►" } else { ">" };
        let marked: Vec<String> = h
            .lines()
            .into_iter()
            .filter(|line| line.contains(left))
            .collect();
        assert_eq!(marked.len(), 1, "{marked:?}");
        assert!(marked[0].contains("33"), "the newest game is selected");
        h.press(KeyCode::Down);
        let marked = h.lines().into_iter().find(|line| line.contains(left));
        assert!(marked.is_some_and(|line| line.contains("22")));
    }
}

#[test]
fn enter_replays_the_selected_seed_with_the_same_variant() {
    let mut h = open_stats(three_games(), 100, 30);
    h.press(KeyCode::Down);
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::Board);
    let session = h.app.board().session();
    assert_eq!(session.map(|s| s.seed()), Some(22));
    assert_eq!(session.map(|s| s.variant()), Some(Variant::FreeCell));
    let storage = h.storage.borrow();
    assert_eq!(storage.history.len(), 4);
    assert_eq!(storage.history[3].seed, 22);
    assert_eq!(storage.history[3].variant, Variant::FreeCell);
}

#[test]
fn the_play_button_replays_the_selected_seed() {
    let mut h = open_stats(three_games(), 100, 30);
    click_text(&mut h, "Play this seed again");
    assert_eq!(h.app.screen(), Screen::Board);
    assert_eq!(h.app.board().session().map(|s| s.seed()), Some(33));
    assert_eq!(h.app.board().session().map(|s| s.variant()), Some(KLONDIKE));
}

#[test]
fn c_and_the_copy_button_put_the_seed_on_the_clipboard_and_announce_it() {
    let mut h = open_stats(three_games(), 100, 30);
    h.type_char('c');
    assert_eq!(h.clipboard.borrow().copies, ["33"]);
    assert!(h.text().contains("Seed 33 copied"));
    h.press(KeyCode::Down);
    assert!(!h.text().contains("Seed 33 copied"));
    click_text(&mut h, "Copy seed ]");
    assert_eq!(h.clipboard.borrow().copies, ["33", "22"]);
    assert!(h.text().contains("Seed 22 copied"));
}

#[test]
fn ctrl_c_still_quits_and_does_not_copy() {
    let mut h = open_stats(three_games(), 100, 30);
    ctrl_c(&mut h);
    assert!(h.app.should_quit());
    assert!(h.clipboard.borrow().copies.is_empty());
}

#[test]
fn a_failed_copy_is_announced_with_the_seed_shown_as_text() {
    let mut h = open_stats(three_games(), 100, 30);
    h.clipboard.borrow_mut().failing = true;
    h.type_char('c');
    assert!(h.clipboard.borrow().copies.is_empty());
    let text = h.text();
    assert!(
        text.contains("Could not copy the seed — it is shown here: 33"),
        "{text}"
    );
}

#[test]
fn the_selection_stays_in_bounds_at_both_ends() {
    let mut h = open_stats(three_games(), 100, 30);
    h.press(KeyCode::Up);
    assert!(h.text().contains("Game 1 of 3"));
    for _ in 0..5 {
        h.press(KeyCode::Down);
    }
    assert!(h.text().contains("Game 3 of 3"));
}

#[test]
fn escape_and_the_back_button_return_to_the_main_menu() {
    let mut h = open_stats(three_games(), 100, 30);
    h.press(KeyCode::Esc);
    assert_eq!(h.app.screen(), Screen::MainMenu);
    h.press(KeyCode::Down);
    h.press(KeyCode::Down);
    h.press(KeyCode::Enter);
    click_text(&mut h, "Back ]");
    assert_eq!(h.app.screen(), Screen::MainMenu);
}
