//! Board rendering and contrast tests in low-vision mode at 100 x 30 and 130 x 40.

use oxysol_core::{DrawCount, PyramidRules, SuitCount, Variant, YukonRules};
use ratatui::crossterm::event::KeyCode;
use ratatui::style::Modifier;

use super::*;
use crate::i18n::{Locale, set_locale};
use crate::settings::Settings;
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, TRUE_COLOR};

fn start_with_low_vision(variant: Variant, width: u16, height: u16, low_vision: bool) -> Harness {
    let settings = if low_vision {
        Settings::default().with_low_vision(true, false)
    } else {
        Settings::default()
    };
    let mut h = Harness::with(
        MemoryStorage {
            settings: Some(settings),
            ..MemoryStorage::default()
        },
        TRUE_COLOR,
        width,
        height,
    );
    h.app.apply(Effect::Start {
        variant,
        seed: Some(7),
        undo_allowed: true,
    });
    h
}

#[test]
fn board_renders_cleanly_at_100x30_and_130x40_with_low_vision_on() {
    let variants = [
        Variant::Klondike(DrawCount::Three),
        Variant::Spider(SuitCount::One),
        Variant::FreeCell,
        Variant::Pyramid(PyramidRules::Relaxed),
        Variant::Yukon(YukonRules::Classic),
        Variant::Golf,
    ];
    for variant in variants {
        for (width, height) in [(100, 30), (130, 40)] {
            let mut h = start_with_low_vision(variant, width, height, true);
            let lines = h.lines();
            assert_eq!(
                lines.len(),
                usize::from(height),
                "{variant:?} {width}x{height}"
            );
            assert!(
                lines
                    .iter()
                    .all(|l| l.chars().count() == usize::from(width)),
                "{variant:?} {width}x{height}"
            );
            let buf = h.terminal.backend().buffer();
            for y in 0..height {
                for x in 0..width {
                    assert!(
                        !buf[(x, y)].modifier.contains(Modifier::DIM),
                        "{variant:?} {width}x{height} ({x},{y}) has DIM"
                    );
                }
            }
        }
    }
}

#[test]
fn dump_player_views_for_verification() {
    set_locale(Locale::EnUs);
    let mut menu = Harness::with(MemoryStorage::default(), TRUE_COLOR, 100, 30);
    menu.press(KeyCode::Right);
    let menu_text = menu.text();
    let mut off = start_with_low_vision(Variant::Klondike(DrawCount::Three), 130, 40, false);
    off.type_char('s');
    let off_text = off.text();
    let mut on_100 = start_with_low_vision(Variant::Klondike(DrawCount::Three), 100, 30, true);
    on_100.type_char('s');
    on_100.press(KeyCode::Right);
    let on_100_text = on_100.text();
    let mut on_130 = start_with_low_vision(Variant::Klondike(DrawCount::Three), 130, 40, true);
    on_130.type_char('s');
    on_130.press(KeyCode::Right);
    let on_130_text = on_130.text();
    println!("=== MAIN MENU BANNER (100x30) ===\n{menu_text}");
    println!("=== BOARD LOW VISION OFF (130x40) ===\n{off_text}");
    println!("=== BOARD LOW VISION ON (100x30) ===\n{on_100_text}");
    println!("=== BOARD LOW VISION ON (130x40) ===\n{on_130_text}");
    assert!(!menu_text.is_empty());
    assert!(!off_text.is_empty());
    assert!(!on_100_text.is_empty());
    assert!(!on_130_text.is_empty());
}
