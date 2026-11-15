use oxysol_core::{DrawCount, Move, PyramidRules, SuitCount, Variant};
use ratatui::crossterm::event::KeyCode;

use super::*;
use crate::i18n::{Locale, set_locale};

const HINT: &str = "Arrows: move | Enter: pick up / drop | s: stock | ?: keys | i: how to play";
const NO_STOCK_HINT: &str = "Arrows: move | Enter: pick up / drop | ?: keys | i: how to play";
const WIDTH: usize = 100;

fn hint(h: &Harness) -> String {
    h.app.board().fitted_first_steps("", usize::MAX)
}

fn status_line(h: &mut Harness) -> String {
    let lines = h.lines();
    lines
        .iter()
        .find(|line| line.contains("how to play"))
        .cloned()
        .unwrap_or_default()
}

#[test]
fn a_new_game_shows_the_keys_built_from_the_key_table() {
    let mut h = start(Variant::Klondike(DrawCount::One), 5);
    assert_eq!(hint(&h), HINT);
    assert!(h.text().contains(HINT));
}

#[test]
fn a_game_without_a_stock_does_not_mention_it() {
    let mut h = start(Variant::FreeCell, 5);
    assert_eq!(hint(&h), NO_STOCK_HINT);
    assert!(h.text().contains(NO_STOCK_HINT));
    assert!(!h.text().contains("s: stock"));
}

#[test]
fn the_whole_status_line_fits_the_smallest_terminal_in_every_game() {
    for variant in [
        Variant::Klondike(DrawCount::Three),
        Variant::Spider(SuitCount::Four),
        Variant::FreeCell,
        Variant::Pyramid(PyramidRules::Classic),
    ] {
        let mut h = Harness::with(
            crate::storage::memory::MemoryStorage::new(),
            TRUE_COLOR,
            100,
            30,
        );
        h.app.apply(Effect::Start {
            variant,
            seed: Some(5),
        });
        let line = status_line(&mut h);
        let text = h.app.board().status_text();
        assert!(text.chars().count() <= WIDTH, "{variant:?}: {text}");
        assert!(line.contains(&text), "{variant:?}: {line}");
        assert!(
            text.contains("Enter: pick up / drop"),
            "{variant:?}: {text}"
        );
        assert!(text.contains("i: how to play"), "{variant:?}: {text}");
    }
}

#[test]
fn moving_the_cursor_keeps_the_hint() {
    let mut h = start(Variant::Klondike(DrawCount::One), 5);
    for code in [KeyCode::Right, KeyCode::Down, KeyCode::Up, KeyCode::Left] {
        h.press(code);
    }
    h.type_char('+');
    assert_eq!(hint(&h), HINT);
}

#[test]
fn the_first_action_removes_the_hint_for_good() {
    let mut h = start(Variant::Klondike(DrawCount::One), 5);
    h.type_char('u');
    assert_eq!(hint(&h), "");
    assert!(!h.text().contains("how to play"));
    h.type_char('h');
    h.press(KeyCode::Right);
    assert_eq!(hint(&h), "");
    assert!(!h.text().contains("how to play"));
}

#[test]
fn picking_up_and_cancelling_does_not_bring_the_hint_back() {
    let mut h = start(Variant::Klondike(DrawCount::One), 5);
    let Some((from, _, _)) = first_transfer(&h) else {
        unreachable!("seed 5 has a transfer")
    };
    put_cursor(&mut h, from);
    h.press(KeyCode::Enter);
    h.press(KeyCode::Esc);
    assert_eq!(h.app.screen(), crate::app::Screen::Board);
    assert_eq!(h.app.board().status_key(), None);
    assert_eq!(hint(&h), "");
    assert!(!h.text().contains("how to play"));
}

#[test]
fn opening_a_window_is_not_an_action() {
    let mut h = start(Variant::Klondike(DrawCount::One), 5);
    h.type_char('?');
    h.press(KeyCode::Esc);
    h.type_char('i');
    h.press(KeyCode::Esc);
    assert_eq!(hint(&h), HINT);
}

#[test]
fn a_paused_board_shows_no_hint() {
    let mut h = start(Variant::Klondike(DrawCount::One), 5);
    h.type_char('p');
    assert_eq!(hint(&h), "");
}

#[test]
fn a_dealt_game_shows_it_again_but_a_resumed_game_with_moves_does_not() {
    let mut h = start(Variant::Klondike(DrawCount::One), 5);
    h.type_char('u');
    assert_eq!(hint(&h), "");
    h.app.apply(Effect::Start {
        variant: Variant::Klondike(DrawCount::One),
        seed: Some(6),
    });
    assert_eq!(hint(&h), HINT);

    let fresh = resumed(Variant::Klondike(DrawCount::One), 5, &[]);
    assert_eq!(hint(&fresh), HINT);
    let moves = hint_moves(Variant::Klondike(DrawCount::One), 5);
    let played: Vec<Move> = moves.into_iter().take(1).collect();
    let continued = resumed(Variant::Klondike(DrawCount::One), 5, &played);
    assert_eq!(hint(&continued), "");
}

#[test]
fn the_polish_hint_uses_the_same_keys_in_its_own_words() {
    let h = start(Variant::Klondike(DrawCount::One), 5);
    set_locale(Locale::PlPl);
    let text = hint(&h);
    set_locale(Locale::EnUs);
    assert_eq!(
        text,
        "Strzałki: ruch | Enter: weź / połóż | s: talia | ?: klawisze | i: jak grać"
    );
    assert!(text.chars().count() < 90);
}

#[test]
fn the_focus_stays_in_front_of_the_hint_and_the_least_useful_keys_give_way() {
    let mut h = start(Variant::Klondike(DrawCount::One), 5);
    let focus = h.app.board().focus_text();
    let full = h.app.board().status_text();
    assert_eq!(full, format!("{focus}. {HINT}"));
    h.resize(100, 30);
    let narrow = h.app.board().status_text();
    assert!(
        narrow.starts_with(&format!("{focus}. Arrows: move")),
        "{narrow}"
    );
    h.resize(60, 30);
    let tight = h.app.board().status_text();
    assert!(tight.chars().count() <= 60, "{tight}");
    assert!(tight.starts_with(&focus), "{tight}");
    assert!(tight.ends_with("i: how to play"), "{tight}");
    h.resize(30, 30);
    assert_eq!(h.app.board().status_text(), focus);
}
