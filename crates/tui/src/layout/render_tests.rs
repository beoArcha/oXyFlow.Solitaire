use std::time::Instant;

use oxysol_core::{DrawCount, PileRef, PileSnapshot, Variant, deal};
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;

use super::*;
use crate::audio::AudioStatus;
use crate::caps::TerminalCaps;
use crate::glyphs::Glyphs;
use crate::screens::ViewCtx;
use crate::settings::{GlyphMode, Settings};
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, PLAIN, TRUE_COLOR};
use crate::theme::{Theme, ThemeId};
use crate::widgets::card::{self, Face, Mark, label};
use crate::widgets::pile::{self, CardPos, Marks};

fn started(variant: Variant, caps: TerminalCaps, width: u16, height: u16) -> Harness {
    let mut h = Harness::with(MemoryStorage::new(), caps, width, height);
    h.press(KeyCode::Enter);
    let index = Variant::ALL.iter().position(|v| *v == variant).unwrap_or(0);
    for _ in 0..index {
        h.press(KeyCode::Down);
    }
    h.press(KeyCode::Enter);
    h
}

fn symbols_at(lines: &[String], x: u16, y: u16, count: usize) -> String {
    lines[usize::from(y)]
        .chars()
        .skip(usize::from(x))
        .take(count)
        .collect()
}

fn visible_labels(
    piles: &[PileSnapshot],
    geometry: &BoardGeometry,
) -> Vec<(PileRef, Rect, String)> {
    let glyphs = Glyphs::new(GlyphMode::Unicode);
    let mut found = Vec::new();
    for (snapshot, placed) in piles.iter().zip(&geometry.piles) {
        for (index, table_card) in snapshot.cards.iter().enumerate() {
            let rect = placed.cards[index];
            let stacked = placed.cards.get(index + 1) == Some(&rect);
            if table_card.face_up && !stacked {
                found.push((snapshot.pile, rect, label(table_card.card, &glyphs)));
            }
        }
    }
    found
}

#[test]
fn every_variant_draws_every_visible_card_label_at_both_sizes() {
    for variant in Variant::ALL {
        for (width, height) in [(100, 30), (160, 50)] {
            let mut h = started(variant, TRUE_COLOR, width, height);
            let lines = h.lines();
            let piles = deal(variant, 42).piles();
            let body = chrome_with(Rect::new(0, 0, width, height), true).body;
            let labels = visible_labels(&piles, &compute(variant, &piles, body));
            assert!(labels.len() >= 7, "{variant:?} {width}x{height}");
            for (pile, rect, text) in labels {
                let found = symbols_at(&lines, rect.x + 1, rect.y, text.chars().count());
                assert_eq!(found, text, "{variant:?} {width}x{height} {pile:?}");
            }
        }
    }
}

#[test]
fn ascii_mode_draws_only_ascii_and_keeps_suits_apart() {
    for variant in Variant::ALL {
        let mut h = started(variant, PLAIN, 100, 30);
        let text = h.text();
        assert!(text.is_ascii(), "{variant:?}");
        assert!(text.contains(" seed 42 "), "{variant:?}");
    }
    let glyphs = Glyphs::new(GlyphMode::Ascii);
    let suits: Vec<&str> = oxysol_core::Suit::ALL
        .iter()
        .map(|s| glyphs.suit(*s))
        .collect();
    assert_eq!(suits, ["C", "D", "H", "S"]);
}

#[test]
fn the_header_shows_the_seed_and_the_time() {
    let mut h = started(Variant::Klondike(DrawCount::Three), TRUE_COLOR, 100, 30);
    assert!(h.lines()[0].contains("Klondike, Draw 3 | seed 42 | 0:00"));
    h.advance(75);
    assert!(h.lines()[0].contains("seed 42 | 1:15"));
}

#[test]
fn the_footer_lists_the_board_keys() {
    let mut h = started(Variant::FreeCell, TRUE_COLOR, 100, 30);
    let last = h.lines()[29].clone();
    assert!(
        last.contains("Enter Select") && last.contains("Esc Back"),
        "{last}"
    );
}

#[test]
fn collapsed_face_down_cards_show_a_counter_and_the_stock_its_size() {
    let mut h = started(
        Variant::Spider(oxysol_core::SuitCount::Four),
        TRUE_COLOR,
        100,
        30,
    );
    assert!(h.text().contains("50"));
    let variant = Variant::Klondike(DrawCount::One);
    let mut h = started(variant, TRUE_COLOR, 100, 30);
    assert!(h.text().contains("24"));
}

struct Fixture {
    theme: Theme,
    glyphs: Glyphs,
    settings: Settings,
    caps: TerminalCaps,
}

impl Fixture {
    fn new(mode: GlyphMode) -> Self {
        Self {
            theme: Theme::new(ThemeId::Dark, TRUE_COLOR.color),
            glyphs: Glyphs::new(mode),
            settings: Settings::default(),
            caps: TRUE_COLOR,
        }
    }

    fn ctx(&self) -> ViewCtx<'_> {
        ViewCtx {
            theme: &self.theme,
            glyphs: &self.glyphs,
            settings: &self.settings,
            caps: &self.caps,
            audio_status: AudioStatus::NoDevice,
            sound_styles: &[],
            has_saved_game: false,
            now: Instant::now(),
        }
    }
}

fn card_rows(buf: &Buffer, area: Rect) -> Vec<String> {
    (area.top()..area.bottom())
        .map(|y| {
            (area.left()..area.right())
                .map(|x| buf[(x, y)].symbol())
                .collect()
        })
        .collect()
}

fn drawn_card(mode: GlyphMode, mark: Mark) -> Vec<String> {
    let fixture = Fixture::new(mode);
    let area = Rect::new(0, 0, 7, 5);
    let mut buf = Buffer::empty(area);
    let face = Face::Up(deal(Variant::FreeCell, 1).piles()[8].cards[0].card);
    card::draw(&mut buf, area, face, mark, None, &fixture.ctx());
    card_rows(&buf, area)
}

#[test]
fn focus_selection_target_and_hint_change_the_card_without_color() {
    for mode in [GlyphMode::Unicode, GlyphMode::Ascii] {
        let marks = [
            Mark::default(),
            Mark {
                focus: true,
                ..Mark::default()
            },
            Mark {
                selected: true,
                ..Mark::default()
            },
            Mark {
                target: true,
                ..Mark::default()
            },
            Mark {
                hint: true,
                ..Mark::default()
            },
        ];
        let looks: Vec<Vec<String>> = marks
            .into_iter()
            .map(|mark| drawn_card(mode, mark))
            .collect();
        for (i, first) in looks.iter().enumerate() {
            for second in &looks[i + 1..] {
                assert_ne!(first, second, "{mode:?}");
            }
        }
    }
}

#[test]
fn a_selected_run_marks_every_card_from_the_picked_one() {
    let fixture = Fixture::new(GlyphMode::Unicode);
    let ctx = fixture.ctx();
    let variant = Variant::FreeCell;
    let piles = deal(variant, 3).piles();
    let body = chrome(Rect::new(0, 0, 100, 30)).body;
    let geometry = compute(variant, &piles, body);
    let column = PileRef::Tableau(0);
    let marks = Marks {
        selected: Some(CardPos {
            pile: column,
            index: 3,
        }),
        focus: Some(CardPos {
            pile: column,
            index: 5,
        }),
        ..Marks::default()
    };
    let mut buf = Buffer::empty(Rect::new(0, 0, 100, 30));
    for (snapshot, placed) in piles.iter().zip(&geometry.piles) {
        pile::draw(&mut buf, snapshot, placed, &marks, &ctx);
    }
    let placed = geometry
        .pile(column)
        .map(|p| p.cards.clone())
        .unwrap_or_default();
    let lines: Vec<String> = card_rows(&buf, Rect::new(0, 0, 100, 30));
    let corner = |index: usize| symbols_at(&lines, placed[index].x, placed[index].y, 1);
    assert_eq!(corner(2), "┌");
    assert_eq!(corner(3), "╒");
    assert_eq!(corner(4), "╒");
    assert_eq!(corner(5), "╒");
}

#[test]
fn an_empty_slot_can_be_focused_and_a_target_pile_is_marked() {
    let fixture = Fixture::new(GlyphMode::Ascii);
    let ctx = fixture.ctx();
    let variant = Variant::FreeCell;
    let piles = deal(variant, 3).piles();
    let geometry = compute(variant, &piles, chrome(Rect::new(0, 0, 100, 30)).body);
    let cell = PileRef::Cell(0);
    let marks = Marks {
        focus: Some(CardPos {
            pile: cell,
            index: 0,
        }),
        targets: vec![PileRef::Tableau(2)],
        ..Marks::default()
    };
    let mut buf = Buffer::empty(Rect::new(0, 0, 100, 30));
    for (snapshot, placed) in piles.iter().zip(&geometry.piles) {
        pile::draw(&mut buf, snapshot, placed, &marks, &ctx);
    }
    let lines = card_rows(&buf, Rect::new(0, 0, 100, 30));
    let slot = geometry.pile(cell).map(|p| p.slot).unwrap_or_default();
    assert_eq!(symbols_at(&lines, slot.x, slot.y, 1), "@");
    let top = geometry
        .pile(PileRef::Tableau(2))
        .and_then(|p| p.cards.last().copied())
        .unwrap_or_default();
    assert!(symbols_at(&lines, top.x, top.y, top.width.into()).contains('~'));
}
