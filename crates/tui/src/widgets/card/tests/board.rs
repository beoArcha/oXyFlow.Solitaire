use oxysol_core::{DrawCount, PileRef, SuitCount, Variant};
use ratatui::layout::Rect;

use crate::caps::TerminalCaps;
use crate::i18n::{Locale, set_locale};
use crate::layout::{BoardGeometry, chrome_with, compute};
use crate::screens::Effect;
use crate::settings::Settings;
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, PLAIN, TRUE_COLOR};

pub(super) const KLONDIKE_THREE: Variant = Variant::Klondike(DrawCount::Three);
pub(super) const KLONDIKE_ONE: Variant = Variant::Klondike(DrawCount::One);
pub(super) const SPIDER: Variant = Variant::Spider(SuitCount::Four);

pub(super) fn board(
    (variant, seed): (Variant, u64),
    caps: TerminalCaps,
    locale: Locale,
    size: (u16, u16),
) -> Harness {
    let storage = MemoryStorage {
        settings: Some(Settings {
            locale,
            ..Settings::for_caps(&caps)
        }),
        ..MemoryStorage::new()
    };
    let mut h = Harness::with(storage, caps, size.0, size.1);
    h.app.apply(Effect::Start {
        variant,
        seed: Some(seed),
        undo_allowed: true,
    });
    h
}

fn geometry(h: &Harness, size: (u16, u16)) -> BoardGeometry {
    let Some(session) = h.app.board().session() else {
        unreachable!("a game is running")
    };
    let body = chrome_with(Rect::new(0, 0, size.0, size.1), true).body;
    compute(session.variant(), &session.game().piles(), body)
}

fn text_at(lines: &[String], rect: Rect, row: u16, width: u16) -> String {
    lines[usize::from(rect.y + row)]
        .chars()
        .skip(usize::from(rect.x))
        .take(usize::from(width))
        .collect()
}

fn slot_middle(h: &mut Harness, size: (u16, u16), pile: PileRef) -> String {
    let lines = h.lines();
    let Some(slot) = geometry(h, size).pile(pile).map(|placed| placed.slot) else {
        unreachable!("the pile has a geometry")
    };
    text_at(&lines, slot, slot.height / 2, slot.width)
        .trim_matches([' ', '│', '|'])
        .to_owned()
}

#[test]
fn spider_face_down_columns_differ_from_empty_slots_at_both_screen_sizes() {
    for size in [(100, 30), (130, 40)] {
        for caps in [TRUE_COLOR, PLAIN] {
            let mut h = board((SPIDER, 42), caps, Locale::EnUs, size);
            let lines = h.lines();
            let placed = geometry(&h, size);
            let Some(slot) = placed.pile(PileRef::Foundation(0)).map(|p| p.slot) else {
                unreachable!("Spider has foundations")
            };
            let empty_top = text_at(&lines, slot, 0, slot.width);
            let mut columns = 0;
            for pile in &placed.piles {
                let PileRef::Tableau(_) = pile.pile else {
                    continue;
                };
                let first = pile.cards[0];
                let top = text_at(&lines, first, 0, first.width);
                assert_ne!(top, empty_top, "{size:?} {:?}", pile.pile);
                let back = if caps.unicode { '▒' } else { '#' };
                assert!(top.contains(back), "{size:?} {:?}: {top}", pile.pile);
                columns += 1;
            }
            assert_eq!(columns, 10);
        }
    }
}

#[test]
fn a_run_of_face_down_cards_at_a_down_step_of_one_is_one_counted_row() {
    let mut h = board((SPIDER, 42), TRUE_COLOR, Locale::EnUs, (100, 30));
    let lines = h.lines();
    let text = lines.join("\n");
    assert!(text.contains("x5") && text.contains("x4"), "{text}");
}

#[test]
fn the_waste_fan_shows_two_covered_cards_by_their_label_and_no_bottom_fragments() {
    let size = (100, 30);
    let mut h = board((KLONDIKE_THREE, 42), TRUE_COLOR, Locale::EnUs, size);
    h.type_char('s');
    let lines = h.lines();
    let Some(waste) = geometry(&h, size).pile(PileRef::Waste).cloned() else {
        unreachable!("Klondike has a waste")
    };
    let shown: Vec<Rect> = waste
        .cards
        .iter()
        .copied()
        .skip(waste.cards.len() - 3)
        .collect();
    assert_eq!(shown.len(), 3);
    for pair in shown.windows(2) {
        let visible = pair[1].x - pair[0].x;
        assert!(
            visible == 3 || visible == 4,
            "label width plus one, got {visible}"
        );
        let bottom = text_at(&lines, pair[0], pair[0].height - 1, visible);
        assert!(bottom.trim().is_empty(), "no fragment: {bottom:?}");
        let top = text_at(&lines, pair[0], 0, visible);
        assert!(
            top[top.char_indices().nth(1).map_or(0, |(i, _)| i)..]
                .chars()
                .next()
                .is_some()
        );
    }
    let last = shown[2];
    let bottom = text_at(&lines, last, last.height - 1, last.width);
    assert!(
        bottom.contains('┘'),
        "the playable card keeps its bottom index: {bottom}"
    );
}

#[test]
fn an_empty_foundation_shows_the_ace_placeholder_in_both_locales() {
    for locale in Locale::ALL {
        let mut h = board((KLONDIKE_ONE, 42), TRUE_COLOR, locale, (100, 30));
        assert_eq!(slot_middle(&mut h, (100, 30), PileRef::Foundation(0)), "A");
        assert_eq!(slot_middle(&mut h, (100, 30), PileRef::Foundation(3)), "A");
    }
    set_locale(Locale::EnUs);
}

#[test]
fn the_stock_shows_its_count_with_a_prefix_and_an_empty_one_its_placeholder() {
    let size = (100, 30);
    let mut h = board((KLONDIKE_ONE, 42), TRUE_COLOR, Locale::EnUs, size);
    assert!(h.text().contains("x24"));
    for _ in 0..24 {
        h.type_char('s');
    }
    assert_eq!(slot_middle(&mut h, size, PileRef::Stock), "↑");
    let mut ascii = board((KLONDIKE_ONE, 42), PLAIN, Locale::EnUs, size);
    for _ in 0..24 {
        ascii.type_char('s');
    }
    assert_eq!(slot_middle(&mut ascii, size, PileRef::Stock), "^");
}

#[test]
fn a_spent_stock_shows_the_x_placeholder() {
    let size = (100, 30);
    let mut h = board((SPIDER, 42), TRUE_COLOR, Locale::EnUs, size);
    assert!(h.text().contains("x50"));
    for _ in 0..5 {
        h.type_char('s');
    }
    assert_eq!(slot_middle(&mut h, size, PileRef::Stock), "x");
}

#[test]
fn the_hint_marks_the_source_and_the_target_on_the_board() {
    for (caps, source, target) in [(TRUE_COLOR, "?→", "→?"), (PLAIN, "?>", ">?")] {
        let mut h = board((KLONDIKE_ONE, 5), caps, Locale::EnUs, (100, 30));
        h.type_char('h');
        let text = h.text();
        assert!(text.contains(source), "{source}");
        assert!(text.contains(target), "{target}");
    }
}
