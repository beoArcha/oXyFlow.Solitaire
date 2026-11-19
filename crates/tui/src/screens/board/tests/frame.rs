//! The field frame, drawn only in the themes whose felt has no edge of its own (High Contrast): a
//! one-cell line around the felt, in Unicode and in ASCII, that no card covers and no click reaches.
//! The other themes have no frame and keep the full board area.

use oxysol_core::{DrawCount, PyramidRules, SuitCount, Variant};
use ratatui::layout::{Position, Rect};

use super::*;
use crate::caps::TerminalCaps;
use crate::layout::{self, CardSize};
use crate::settings::Settings;
use crate::test_support::PLAIN;
use crate::theme::ThemeId;

const SEED: u64 = 5;
const SIZES: [(u16, u16); 5] = [(100, 30), (115, 35), (130, 30), (180, 36), (200, 60)];
const KLONDIKE: Variant = Variant::Klondike(DrawCount::Three);
const GAMES: [Variant; 4] = [
    KLONDIKE,
    Variant::Spider(SuitCount::Two),
    Variant::FreeCell,
    Variant::Pyramid(PyramidRules::Relaxed),
];

fn started(variant: Variant, theme: ThemeId, caps: TerminalCaps, size: (u16, u16)) -> Harness {
    let storage = MemoryStorage {
        settings: Some(Settings {
            theme,
            ..Settings::for_caps(&caps)
        }),
        ..MemoryStorage::default()
    };
    let mut h = Harness::with(storage, caps, size.0, size.1);
    h.app.apply(Effect::Start {
        variant,
        seed: Some(SEED),
        undo_allowed: true,
    });
    h
}

fn chrome(size: (u16, u16), theme: ThemeId) -> layout::Chrome {
    layout::chrome_panels(
        Rect::new(0, 0, size.0, size.1),
        true,
        true,
        theme.palette().field_edge,
    )
}

fn cell(lines: &[String], x: u16, y: u16) -> char {
    lines[usize::from(y)]
        .chars()
        .nth(usize::from(x))
        .unwrap_or(' ')
}

fn corners(lines: &[String], field: Rect) -> [char; 4] {
    let (right, bottom) = (field.right() - 1, field.bottom() - 1);
    [
        cell(lines, field.x, field.y),
        cell(lines, right, field.y),
        cell(lines, field.x, bottom),
        cell(lines, right, bottom),
    ]
}

fn ring(field: Rect) -> Vec<(u16, u16)> {
    let columns = field.left()..field.right();
    let rows = field.top()..field.bottom();
    let edge = |x: u16, y: u16| {
        x == field.left() || x == field.right() - 1 || y == field.top() || y == field.bottom() - 1
    };
    rows.flat_map(|y| columns.clone().map(move |x| (x, y)))
        .filter(|&(x, y)| edge(x, y))
        .collect()
}

#[test]
fn high_contrast_draws_the_frame_around_the_field_in_every_size_and_game() {
    for size in SIZES {
        for variant in GAMES {
            for (caps, expected) in [
                (TRUE_COLOR, ['┌', '┐', '└', '┘']),
                (PLAIN, ['+', '+', '+', '+']),
            ] {
                let mut h = started(variant, ThemeId::HighContrast, caps, size);
                let lines = h.lines();
                let field = chrome(size, ThemeId::HighContrast).field;
                assert_eq!(corners(&lines, field), expected, "{size:?} {variant:?}");
            }
        }
    }
}

#[test]
fn the_frame_uses_ascii_edges_in_ascii_mode() {
    let mut h = started(KLONDIKE, ThemeId::HighContrast, PLAIN, (100, 30));
    let lines = h.lines();
    let field = chrome((100, 30), ThemeId::HighContrast).field;
    let mid = field.y + field.height / 2;
    assert_eq!(cell(&lines, field.x, mid), '|');
    assert_eq!(cell(&lines, field.right() - 1, mid), '|');
    assert_eq!(cell(&lines, field.x + field.width / 2, field.y), '-');
    assert_eq!(
        cell(&lines, field.x + field.width / 2, field.bottom() - 1),
        '-'
    );
}

#[test]
fn in_high_contrast_no_card_pile_or_click_touches_the_frame() {
    for size in SIZES {
        let parts = chrome(size, ThemeId::HighContrast);
        for variant in GAMES {
            let mut h = started(variant, ThemeId::HighContrast, TRUE_COLOR, size);
            h.lines();
            let piles = board(&h).piles();
            let Some(geometry) = board(&h).geometry(&piles) else {
                unreachable!("a game is running")
            };
            for pile in &geometry.piles {
                for rect in pile.cards.iter().chain([&pile.slot, &pile.area]) {
                    assert_eq!(rect.intersection(parts.body), *rect, "{size:?} {variant:?}");
                }
            }
            for (x, y) in ring(parts.field) {
                assert!(!parts.body.contains(Position::new(x, y)));
                assert_eq!(geometry.hit(x, y), None, "{size:?} {variant:?} ({x},{y})");
                h.click(x, y);
                assert!(board(&h).pick.is_none(), "{size:?} {variant:?} ({x},{y})");
            }
        }
    }
}

#[test]
fn the_other_themes_draw_no_frame_and_keep_the_whole_field_for_the_board() {
    for theme in [ThemeId::Dark, ThemeId::Light] {
        for size in SIZES {
            let parts = chrome(size, theme);
            assert_eq!(parts.body, parts.field, "{theme:?} {size:?}");
            for variant in GAMES {
                for caps in [TRUE_COLOR, PLAIN] {
                    let mut h = started(variant, theme, caps, size);
                    h.lines();
                    assert_eq!(board(&h).chrome(), parts, "{theme:?} {size:?} {variant:?}");
                }
            }
        }
    }
}

#[test]
fn a_standard_screen_keeps_13_by_11_klondike_cards_without_the_frame() {
    let thirteen = CardSize {
        width: 13,
        height: 11,
    };
    let eleven = CardSize {
        width: 11,
        height: 9,
    };
    for (theme, expected) in [
        (ThemeId::Dark, thirteen),
        (ThemeId::Light, thirteen),
        (ThemeId::HighContrast, eleven),
    ] {
        let mut h = started(KLONDIKE, theme, TRUE_COLOR, (130, 40));
        h.lines();
        let piles = board(&h).piles();
        let Some(geometry) = board(&h).geometry(&piles) else {
            unreachable!("a game is running")
        };
        assert_eq!(geometry.card, expected, "{theme:?}");
    }
}
