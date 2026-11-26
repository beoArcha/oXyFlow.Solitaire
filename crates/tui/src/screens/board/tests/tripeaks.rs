//! Board controller and rendering tests for TriPeaks: layout visibility of all three peaks,
//! stock, and waste at 100×30 and 130×40; non-color distinction between covered and playable
//! cards; cursor navigation; and accurate footer/guide labels in every focus state.

use oxysol_core::{Move, PileRef, Variant};
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::style::Modifier;

use super::*;
use crate::glyphs::{Glyphs, is_wgl4};
use crate::i18n::{Locale, set_locale};
use crate::layout;
use crate::settings::{GlyphMode, Settings};
use crate::test_support::{PLAIN, TRUE_COLOR};

const SEED: u64 = 5;

fn peak(row: u8, index: u8) -> PileRef {
    PileRef::Pyramid { row, index }
}

fn board_at(size: (u16, u16), caps: crate::caps::TerminalCaps) -> Harness {
    let storage = MemoryStorage {
        settings: Some(Settings {
            locale: Locale::EnUs,
            moves_panel: false,
            ..Settings::for_caps(&caps)
        }),
        ..MemoryStorage::default()
    };
    let mut h = Harness::with(storage, caps, size.0, size.1);
    h.app.apply(Effect::Start {
        variant: Variant::TriPeaks,
        seed: Some(SEED),
        undo_allowed: true,
    });
    h.lines();
    h
}

fn text_at(lines: &[String], x: u16, y: u16, width: u16) -> String {
    lines[usize::from(y)]
        .chars()
        .skip(usize::from(x))
        .take(usize::from(width))
        .collect()
}

#[test]
fn the_cursor_starts_at_the_bottom_left_of_the_first_peak() {
    let h = start(Variant::TriPeaks, SEED);
    assert_eq!(cursor(&h), (peak(3, 0), 1));
}

#[test]
fn board_renders_all_three_peaks_stock_and_waste_at_100x30_and_130x40() {
    for size in [(100, 30), (130, 40)] {
        for (caps, mode) in [(TRUE_COLOR, GlyphMode::Unicode), (PLAIN, GlyphMode::Ascii)] {
            let mut h = board_at(size, caps);
            let lines = h.lines();
            assert_eq!(lines.len(), usize::from(size.1));
            for line in &lines {
                assert!(
                    line.chars().count() <= usize::from(size.0),
                    "{size:?}: {line}"
                );
                for ch in line.chars() {
                    assert!(ch.is_ascii() || is_wgl4(ch), "{size:?}: {ch:?}");
                    if !caps.unicode {
                        assert!(ch.is_ascii(), "{size:?} ASCII: {ch:?}");
                    }
                }
            }
            let body = layout::chrome_with(Rect::new(0, 0, size.0, size.1), true, false).body;
            let piles = session(&h).game().piles();
            let geometry = layout::compute(Variant::TriPeaks, &piles, body);
            let glyphs = Glyphs::new(mode);

            for tip in [peak(0, 0), peak(0, 1), peak(0, 2)] {
                let rect = geometry.pile(tip).expect("peak tip exists").slot;
                let back_row = text_at(&lines, rect.x, rect.y + 1, rect.width);
                assert!(
                    back_row.contains(glyphs.face_down),
                    "{size:?} {mode:?} {tip:?} should show back pattern on interior row: {back_row:?}"
                );
            }
            for index in 0..10u8 {
                let pile = peak(3, index);
                let rect = geometry.pile(pile).expect("bottom row exists").slot;
                let top = text_at(&lines, rect.x, rect.y, rect.width);
                assert!(
                    !top.contains(glyphs.face_down),
                    "{size:?} {mode:?} {pile:?} should be face up: {top:?}"
                );
            }
            let stock_rect = geometry.pile(PileRef::Stock).expect("stock exists").slot;
            let stock_bottom = text_at(
                &lines,
                stock_rect.x,
                stock_rect.bottom().saturating_sub(1),
                stock_rect.width,
            );
            assert!(
                stock_bottom.contains("x23"),
                "{size:?} {mode:?} stock badge missing: {stock_bottom:?}"
            );
            let waste_rect = geometry.pile(PileRef::Waste).expect("waste exists").slot;
            let waste_top = text_at(&lines, waste_rect.x, waste_rect.y, waste_rect.width);
            assert!(
                !waste_top.trim().is_empty() && !waste_top.contains(glyphs.face_down),
                "{size:?} {mode:?} waste card missing: {waste_top:?}"
            );
        }
    }
    set_locale(Locale::EnUs);
}

#[test]
fn covered_cards_are_distinguishable_from_playable_cards_without_color() {
    for (caps, mode) in [(TRUE_COLOR, GlyphMode::Unicode), (PLAIN, GlyphMode::Ascii)] {
        let (playable_edge, blocked_edge) = if mode == GlyphMode::Ascii {
            ("|", ":")
        } else {
            ("│", "·")
        };
        let mut h = board_at((130, 40), caps);
        let lines = h.lines();
        let body = layout::chrome_with(Rect::new(0, 0, 130, 40), true, false).body;
        let piles = session(&h).game().piles();
        let geometry = layout::compute(Variant::TriPeaks, &piles, body);
        let glyphs = Glyphs::new(mode);
        let buf = h.terminal.backend().buffer();

        for row in 0..3u8 {
            let rect = geometry.pile(peak(row, 0)).unwrap().slot;
            let edge = buf[(rect.x, rect.y + 1)].symbol();
            assert_eq!(edge, blocked_edge, "{mode:?} row {row}");
            let back_row = text_at(&lines, rect.x, rect.y + 1, rect.width);
            assert!(
                back_row.contains(glyphs.face_down),
                "{mode:?} row {row}: {back_row:?}"
            );
        }
        let playable_rect = geometry.pile(peak(3, 1)).unwrap().slot;
        assert_eq!(
            buf[(playable_rect.x, playable_rect.y + 1)].symbol(),
            playable_edge
        );
        assert!(
            !buf[(playable_rect.x + 1, playable_rect.y + 1)]
                .modifier
                .contains(Modifier::DIM)
        );
    }
}

#[test]
fn footer_and_guide_labels_match_every_focus_state() {
    let mut h = board_at((130, 40), TRUE_COLOR);
    let guide_text: String = h
        .lines()
        .iter()
        .map(|line| line.chars().skip(101).take(28).collect::<String>())
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        guide_text.contains("Clear all 28 cards from the three peaks"),
        "guide text: {guide_text:?}"
    );
    assert!(!guide_text.contains("run size"));
    assert!(!guide_text.contains("waste quick"));

    let playable = loop {
        if let Some(pile) = legal(&h).into_iter().find_map(|mv| match mv {
            Move::Transfer { from, .. } => Some(from),
            _ => None,
        }) {
            break pile;
        }
        h.type_char('s');
    };
    let unplayable = (0..10u8)
        .map(|i| peak(3, i))
        .find(|&p| {
            !legal(&h)
                .iter()
                .any(|mv| matches!(mv, Move::Transfer { from, .. } if *from == p))
        })
        .expect("has an unplayable bottom card");

    put_cursor(&mut h, playable);
    let text = h.text();
    assert!(text.contains("Now: Row 4, card "), "{text}");
    assert!(text.contains("Enter: pick up"), "{text}");
    assert!(text.contains("a: quick move"), "{text}");
    assert!(text.contains("turn stock"), "{text}");
    assert!(!text.contains("waste quick"), "{text}");
    assert!(!text.contains("change run size"), "{text}");

    h.press(KeyCode::Enter);
    let carrying = h.text();
    assert!(carrying.contains("Now: Carrying: "), "{carrying}");
    assert!(carrying.contains("1 target"), "{carrying}");
    assert!(carrying.contains("Enter: drop"), "{carrying}");
    assert!(carrying.contains("Esc: put back"), "{carrying}");
    h.press(KeyCode::Esc);

    put_cursor(&mut h, unplayable);
    let blocked_do = h.lines().last().cloned().unwrap_or_default();
    assert!(!blocked_do.contains("Enter: pick up"), "{blocked_do}");
    assert!(!blocked_do.contains("a: quick move"), "{blocked_do}");
    assert!(blocked_do.contains("s: turn stock"), "{blocked_do}");

    put_cursor(&mut h, peak(0, 0));
    let covered_now = h.lines()[38].clone();
    assert!(
        covered_now.contains("Row 1, card 1: face-down card"),
        "{covered_now}"
    );
    h.press(KeyCode::Enter);
    assert_eq!(status(&h), Some("board.error.face_down"));

    put_cursor(&mut h, PileRef::Waste);
    let waste_lines = h.lines();
    assert!(
        waste_lines[38].contains("Now: Waste: "),
        "{}",
        waste_lines[38]
    );
    assert!(
        !waste_lines[39].contains("Enter: pick up"),
        "{}",
        waste_lines[39]
    );
    assert!(
        !waste_lines[39].contains("waste quick"),
        "{}",
        waste_lines[39]
    );

    put_cursor(&mut h, PileRef::Stock);
    let stock_lines = h.lines();
    assert!(
        stock_lines[38].contains("Now: Stock: face-down card"),
        "{}",
        stock_lines[38]
    );
    assert!(
        stock_lines[39].starts_with("Do: Enter: turn stock"),
        "{}",
        stock_lines[39]
    );
}
