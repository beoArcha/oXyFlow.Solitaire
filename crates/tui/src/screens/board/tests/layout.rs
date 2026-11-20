//! The board screen per size class: where the game information, the controls strip and the guide
//! panel are, that no fact is shown twice, and that every render stays inside WGL4 or ASCII.

use oxysol_core::{DrawCount, PyramidRules, SuitCount, Variant};
use ratatui::crossterm::event::KeyCode;

use super::*;
use crate::caps::TerminalCaps;
use crate::glyphs::is_wgl4;
use crate::i18n::{Locale, set_locale};
use crate::settings::Settings;
use crate::test_support::PLAIN;

const KLONDIKE: Variant = Variant::Klondike(DrawCount::Three);
const SEED: u64 = 5;
const SIZES: [(u16, u16); 5] = [(100, 30), (115, 35), (130, 30), (130, 40), (200, 60)];

const GAMES: [Variant; 4] = [
    KLONDIKE,
    Variant::Spider(SuitCount::Two),
    Variant::FreeCell,
    Variant::Pyramid(PyramidRules::Relaxed),
];

fn shown(
    variant: Variant,
    (width, height): (u16, u16),
    locale: Locale,
    caps: TerminalCaps,
) -> Harness {
    let storage = MemoryStorage {
        settings: Some(Settings {
            locale,
            moves_panel: false,
            ..Settings::for_caps(&caps)
        }),
        ..MemoryStorage::default()
    };
    let mut h = Harness::with(storage, caps, width, height);
    h.app.apply(Effect::Start {
        variant,
        seed: Some(SEED),
        undo_allowed: true,
    });
    h.send(crate::input::Input::Tick);
    h.lines();
    h
}

fn columns(line: &str, from: usize, to: usize) -> String {
    line.chars().skip(from).take(to - from).collect()
}

fn count(text: &str, needle: &str) -> usize {
    text.matches(needle).count()
}

fn panel_width(columns: u16) -> u16 {
    if columns < 180 { 30 } else { 40 }
}

fn every_screen() -> Vec<(Variant, (u16, u16), Locale, TerminalCaps)> {
    let mut all = Vec::new();
    for size in SIZES {
        for locale in Locale::offered() {
            for caps in [TRUE_COLOR, PLAIN] {
                all.extend(GAMES.map(|variant| (variant, size, locale, caps)));
            }
        }
    }
    all
}

#[test]
fn every_size_locale_and_glyph_mode_draws_inside_wgl4_or_ascii() {
    for (variant, size, locale, caps) in every_screen() {
        let mut h = shown(variant, size, locale, caps);
        let lines = h.lines();
        assert_eq!(lines.len(), usize::from(size.1));
        let plain_english = !caps.unicode && locale == Locale::EnUs;
        for ch in lines.concat().chars() {
            assert!(ch.is_ascii() || is_wgl4(ch), "{size:?} {variant:?}: {ch:?}");
            assert!(!plain_english || ch.is_ascii(), "{size:?}: {ch:?}");
        }
    }
    set_locale(Locale::EnUs);
}

#[test]
fn a_compact_screen_has_a_header_row_a_two_row_strip_and_no_panel() {
    for size in [(100, 30), (115, 35), (129, 40)] {
        let mut h = shown(KLONDIKE, size, Locale::EnUs, TRUE_COLOR);
        let lines = h.lines();
        let last = lines.len();
        assert!(
            lines[0].contains("Klondike, Draw 3"),
            "{size:?}: {}",
            lines[0]
        );
        assert!(lines[0].contains("seed 5") && lines[0].contains("Moves: 0"));
        assert!(lines[last - 2].starts_with("Now: "), "{size:?}");
        assert!(lines[last - 1].starts_with("Do: "), "{size:?}");
        assert!(!h.text().contains("Guide"), "{size:?}");
    }
}

#[test]
fn a_standard_screen_puts_the_facts_in_the_game_box_and_has_no_header_row() {
    let mut h = shown(KLONDIKE, (130, 40), Locale::EnUs, TRUE_COLOR);
    let lines = h.lines();
    assert!(!columns(&lines[0], 0, 100).contains("Klondike"));
    let panel: Vec<String> = lines.iter().map(|line| columns(line, 100, 130)).collect();
    assert!(panel[0].starts_with("┌ Guide (g: hide)"), "{}", panel[0]);
    assert!(panel[1].contains("Game"), "{}", panel[1]);
    let facts = [
        "Klondike, Draw 3",
        "seed 5",
        "Time 0:00",
        "Moves: 0",
        "Turned over: 0",
    ];
    for (offset, fact) in facts.into_iter().enumerate() {
        let row = &panel[2 + offset];
        assert!(row.contains(fact), "{fact}: {row}");
    }
    assert!(lines[38].starts_with("Now: ") && lines[39].starts_with("Do: "));
    assert!(columns(&lines[38], 100, 101) == "│");
    assert!(columns(&lines[39], 100, 101) == "└");
}

#[test]
fn a_wide_screen_has_a_forty_column_panel() {
    let mut h = shown(KLONDIKE, (200, 60), Locale::EnUs, TRUE_COLOR);
    let lines = h.lines();
    let top = columns(&lines[0], 160, 200);
    assert!(top.starts_with("┌ Guide") && top.ends_with('┐'), "{top}");
    assert!(columns(&lines[59], 160, 200).starts_with('└'));
    assert!(lines[58].starts_with("Now: "));
    assert_eq!(columns(&lines[58], 160, 161), "│");
    let mut standard = shown(KLONDIKE, (179, 60), Locale::EnUs, TRUE_COLOR);
    assert!(columns(&standard.lines()[0], 149, 179).starts_with("┌ Guide"));
}

#[test]
fn the_panel_is_framed_in_both_glyph_sets_and_both_locales() {
    for locale in Locale::offered() {
        let sets = [
            (TRUE_COLOR, ['┌', '┐', '└', '┘'], ['│', '├', '┤']),
            (PLAIN, ['+', '+', '+', '+'], ['|', '+', '+']),
        ];
        for (caps, corners, [side, tee_left, tee_right]) in sets {
            for size in [(130, 30), (130, 40), (200, 60)] {
                let mut h = shown(KLONDIKE, size, locale, caps);
                let lines = h.lines();
                let left = usize::from(size.0 - panel_width(size.0));
                let cells = |line: &String| -> Vec<char> {
                    columns(line, left, usize::from(size.0)).chars().collect()
                };
                let top = cells(&lines[0]);
                let bottom = cells(&lines[lines.len() - 1]);
                assert_eq!([top[0], top[top.len() - 1]], [corners[0], corners[1]]);
                assert_eq!(
                    [bottom[0], bottom[bottom.len() - 1]],
                    [corners[2], corners[3]]
                );
                for line in &lines[1..lines.len() - 1] {
                    let row = cells(line);
                    assert!([side, tee_left].contains(&row[0]), "{size:?}: {line}");
                    assert!([side, tee_right].contains(&row[row.len() - 1]), "{size:?}");
                }
            }
        }
    }
    set_locale(Locale::EnUs);
}

#[test]
fn no_fact_is_shown_twice() {
    for size in SIZES {
        let mut h = shown(KLONDIKE, size, Locale::EnUs, TRUE_COLOR);
        h.type_char('s');
        h.type_char('s');
        h.type_char('u');
        let text = h.text();
        let now_row = text
            .lines()
            .find_map(|line| line.strip_prefix("Now: "))
            .unwrap_or_default();
        let focus = now_row.split("  ").next().unwrap_or_default().trim_end();
        assert!(focus.contains(": "), "{focus}");
        assert_eq!(count(&text, focus), 1, "{size:?}: {focus}\n{text}");
        let facts = [
            "Turned over: 0",
            "seed 5",
            "Moves: 1",
            "Time 0:00",
            "Klondike, Draw 3",
            "s: turn stock",
            "Shift+R: restart",
            "o: options",
        ];
        for fact in facts {
            assert_eq!(count(&text, fact), 1, "{size:?}: {fact}\n{text}");
        }
    }
}

#[test]
fn the_seed_is_a_click_target_in_the_header_and_in_the_game_box() {
    for size in SIZES {
        let mut h = shown(KLONDIKE, size, Locale::EnUs, TRUE_COLOR);
        let Some((column, row)) = h.find("seed 5") else {
            unreachable!("the seed is shown at {size:?}")
        };
        h.click(column, row);
        assert!(h.clipboard.borrow().copies.is_empty(), "{size:?}: the word");
        h.click(column + 5, row);
        assert_eq!(h.clipboard.borrow().copies, ["5"], "{size:?}");
    }
}

#[test]
fn hiding_the_panel_brings_the_header_row_back() {
    let mut h = shown(KLONDIKE, (130, 40), Locale::EnUs, TRUE_COLOR);
    h.type_char('g');
    let lines = h.lines();
    assert!(lines[0].contains("seed 5"), "{}", lines[0]);
    assert!(lines[0].contains("Klondike, Draw 3"), "{}", lines[0]);
    assert!(!h.text().contains("Guide"));
    let Some((column, row)) = h.find("seed 5") else {
        unreachable!("the header shows the seed")
    };
    h.click(column + 5, row);
    assert_eq!(h.clipboard.borrow().copies, ["5"]);
}

#[test]
fn the_last_event_is_right_aligned_and_the_next_action_clears_it() {
    let mut h = shown(KLONDIKE, (100, 30), Locale::EnUs, TRUE_COLOR);
    h.type_char('s');
    let lines = h.lines();
    assert!(lines[28].starts_with("Now: "), "{}", lines[28]);
    assert!(
        lines[28]
            .trim_end()
            .ends_with("Dealt from the stock.   v: moves"),
        "the event sits right before the Now row's fallback buttons: {}",
        lines[28]
    );
    h.press(KeyCode::Right);
    assert!(!h.text().contains("Dealt from the stock."));
    h.type_char('u');
    h.type_char('u');
    assert!(h.lines()[28].ends_with("Nothing to undo."));
}

#[test]
fn an_event_and_a_refusal_use_different_tokens() {
    let mut h = shown(KLONDIKE, (100, 30), Locale::EnUs, TRUE_COLOR);
    let color_near_the_end = |h: &mut Harness| {
        let end = h.lines()[28].chars().count();
        let column = u16::try_from(end - 3).unwrap_or(0);
        h.terminal.backend().buffer()[(column, 28)].fg
    };
    h.type_char('s');
    let success = color_near_the_end(&mut h);
    h.type_char('u');
    h.type_char('u');
    assert_ne!(success, color_near_the_end(&mut h));
}

#[test]
fn polish_now_rows_use_the_colon_form() {
    let mut h = shown(KLONDIKE, (100, 30), Locale::PlPl, TRUE_COLOR);
    let Some((from, _, _)) = first_transfer(&h) else {
        unreachable!("seed 5 has a transfer")
    };
    put_cursor(&mut h, from);
    let browsing = h.lines()[28].clone();
    assert!(browsing.starts_with("Teraz: Kolumna "), "{browsing}");
    h.press(KeyCode::Enter);
    let carrying = h.lines()[28].clone();
    assert!(carrying.starts_with("Teraz: Trzymasz: "), "{carrying}");
    assert!(carrying.contains("(z: Kolumna "), "{carrying}");
    set_locale(Locale::EnUs);
}

#[test]
fn the_pyramid_names_its_cards_by_row_and_card() {
    let variant = Variant::Pyramid(PyramidRules::Relaxed);
    let mut h = shown(variant, (100, 30), Locale::EnUs, TRUE_COLOR);
    let now = h.lines()[28].clone();
    assert!(now.starts_with("Now: Row "), "{now}");
    assert!(now.contains(", card "), "{now}");
}
