use std::time::Instant;

use oxysol_core::{
    Card, CardId, DrawCount, PileRef, PileSnapshot, Rank, Suit, SuitCount, TableCard, Variant,
    YukonRules, deal,
};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::*;
use crate::audio::AudioStatus;
use crate::glyphs::Glyphs;
use crate::screens::ViewCtx;
use crate::settings::{GlyphMode, Settings};
use crate::test_support::TRUE_COLOR;
use crate::theme::{Theme, ThemeId};
use crate::widgets::card::label;
use crate::widgets::pile::{self, Marks};

fn card(id: u16, face_up: bool) -> TableCard {
    TableCard {
        card: Card {
            id: CardId(id),
            suit: Suit::ALL[usize::from(id % 4)],
            rank: Rank::ALL[usize::from(id % 13)],
        },
        face_up,
    }
}

fn column(down: u16, up: u16) -> Vec<TableCard> {
    (0..down + up).map(|id| card(id, id >= down)).collect()
}

fn with_cards(
    mut piles: Vec<PileSnapshot>,
    pile: PileRef,
    cards: Vec<TableCard>,
) -> Vec<PileSnapshot> {
    for snapshot in piles.iter_mut().filter(|snapshot| snapshot.pile == pile) {
        snapshot.cards = cards.clone();
    }
    piles
}

struct Fixture {
    theme: Theme,
    glyphs: Glyphs,
    settings: Settings,
}

impl Fixture {
    fn new() -> Self {
        Self {
            theme: Theme::new(ThemeId::Dark, TRUE_COLOR.color),
            glyphs: Glyphs::new(GlyphMode::Unicode),
            settings: Settings::default(),
        }
    }

    fn ctx(&self) -> ViewCtx<'_> {
        ViewCtx {
            theme: &self.theme,
            glyphs: &self.glyphs,
            settings: &self.settings,
            caps: &TRUE_COLOR,
            audio_status: AudioStatus::NoDevice,
            sound_styles: &[],
            saved: None,
            pointer: None,
            now: Instant::now(),
            font: crate::app::FontFacts::default(),
        }
    }
}

fn render_board(
    piles: &[PileSnapshot],
    geometry: &BoardGeometry,
    screen: Rect,
    ctx: &ViewCtx<'_>,
) -> Vec<String> {
    let mut buf = Buffer::empty(screen);
    let marks = Marks::default();
    for (snapshot, placed) in piles.iter().zip(&geometry.piles) {
        pile::draw(&mut buf, snapshot, placed, &marks, false, ctx);
    }
    (screen.top()..screen.bottom())
        .map(|y| {
            (screen.left()..screen.right())
                .map(|x| buf[(x, y)].symbol())
                .collect()
        })
        .collect()
}

fn row_slice(lines: &[String], x: u16, y: u16, width: u16) -> String {
    lines[usize::from(y)]
        .chars()
        .skip(usize::from(x))
        .take(usize::from(width))
        .collect()
}

fn assert_last_card_fully_drawn(
    lines: &[String],
    placed: &PileGeometry,
    snapshot: &PileSnapshot,
    card_size: CardSize,
    status_top: u16,
    glyphs: &Glyphs,
    what: &str,
) {
    let Some(last_rect) = placed.cards.last().copied() else {
        unreachable!("{what}: empty column");
    };
    let Some(last_card) = snapshot.cards.last() else {
        unreachable!("{what}: empty snapshot");
    };
    assert_eq!(
        (last_rect.width, last_rect.height),
        (card_size.width, card_size.height),
        "{what}: last card clipped"
    );
    assert!(
        last_rect.bottom() <= status_top,
        "{what}: bottom {} exceeds status bar {status_top}",
        last_rect.bottom()
    );
    let expected = label(last_card.card, glyphs);
    let top_row = row_slice(lines, last_rect.x, last_rect.y, last_rect.width);
    let bottom_row = row_slice(lines, last_rect.x, last_rect.bottom() - 1, last_rect.width);
    assert!(
        top_row.starts_with(&format!("┌{expected}")),
        "{what}: {top_row}"
    );
    assert!(top_row.ends_with('┐'), "{what}: {top_row}");
    assert!(bottom_row.starts_with('└'), "{what}: {bottom_row}");
    assert!(
        bottom_row.ends_with(&format!("{expected}┘")),
        "{what}: {bottom_row}"
    );
}

fn assert_visible_cards_and_hits(
    lines: &[String],
    geometry: &BoardGeometry,
    snapshot: &PileSnapshot,
    glyphs: &Glyphs,
    require_all_face_up: bool,
    what: &str,
) {
    let placed = match geometry.pile(snapshot.pile) {
        Some(placed) => placed,
        None => unreachable!("{what}: missing pile"),
    };
    for (index, table_card) in snapshot.cards.iter().enumerate() {
        let rect = placed.cards[index];
        let stacked = placed.cards.get(index + 1) == Some(&rect);
        if require_all_face_up && table_card.face_up {
            assert!(!stacked, "{what}: face-up card {index} hidden");
        }
        if !table_card.face_up || stacked {
            continue;
        }
        let expected = label(table_card.card, glyphs);
        let drawn = row_slice(
            lines,
            rect.x + 1,
            rect.y,
            u16::try_from(expected.chars().count()).unwrap_or(0),
        );
        assert_eq!(drawn, expected, "{what}: card {index} at y={}", rect.y);
        let hit = geometry.hit(rect.x + 1, rect.y);
        assert_eq!(
            hit,
            Some(Hit {
                pile: snapshot.pile,
                card: Some(index),
            }),
            "{what}: hit at ({}, {})",
            rect.x + 1,
            rect.y
        );
    }
    if placed.collapsed > 0 {
        let first = placed.cards[0];
        let collapsed_row = row_slice(lines, first.x, first.y, first.width);
        assert!(
            collapsed_row.contains(&format!("x{}", placed.collapsed)),
            "{what}: collapsed row {collapsed_row}"
        );
        let hit = geometry.hit(first.x + 1, first.y);
        assert_eq!(
            hit,
            Some(Hit {
                pile: snapshot.pile,
                card: Some(placed.collapsed - 1),
            }),
            "{what}: collapsed hit"
        );
    }
}

#[test]
fn yukon_klondike_and_spider_columns_fit_above_status_bar_for_every_card_size() {
    let fixture = Fixture::new();
    let ctx = fixture.ctx();
    let cases = [
        (
            Variant::Yukon(YukonRules::Classic),
            PileRef::Tableau(6),
            6,
            16,
            true,
        ),
        (
            Variant::Yukon(YukonRules::Classic),
            PileRef::Tableau(6),
            6,
            18,
            false,
        ),
        (
            Variant::Yukon(YukonRules::Classic),
            PileRef::Tableau(6),
            6,
            35,
            false,
        ),
        (
            Variant::Klondike(DrawCount::Three),
            PileRef::Tableau(6),
            6,
            13,
            true,
        ),
        (
            Variant::Klondike(DrawCount::Three),
            PileRef::Tableau(6),
            6,
            25,
            false,
        ),
        (
            Variant::Spider(SuitCount::Four),
            PileRef::Tableau(0),
            5,
            16,
            true,
        ),
        (
            Variant::Spider(SuitCount::Four),
            PileRef::Tableau(0),
            5,
            35,
            false,
        ),
    ];
    for (width, height) in [(100_u16, 30_u16), (100_u16, 60_u16)] {
        let screen = Rect::new(0, 0, width, height);
        let parts = chrome(screen);
        for (variant, pile, down, up, all_ranks_at_30) in cases {
            let piles = with_cards(deal(variant, 9).piles(), pile, column(down, up));
            let snapshot = match piles.iter().find(|s| s.pile == pile) {
                Some(snapshot) => snapshot,
                None => unreachable!("pile exists"),
            };
            for limit in SIZES.into_iter().map(Some).chain([None]) {
                let what = format!("{variant:?} ({down}+{up}) {width}x{height} limit={limit:?}");
                let geometry = compute_within(variant, &piles, parts.body, limit);
                let placed = match geometry.pile(pile) {
                    Some(placed) => placed,
                    None => unreachable!("{what}: missing pile"),
                };
                let lines = render_board(&piles, &geometry, screen, &ctx);
                assert_last_card_fully_drawn(
                    &lines,
                    placed,
                    snapshot,
                    geometry.card,
                    parts.now.y,
                    ctx.glyphs,
                    &what,
                );
                let require_all = height == 60 || all_ranks_at_30;
                assert_visible_cards_and_hits(
                    &lines,
                    &geometry,
                    snapshot,
                    ctx.glyphs,
                    require_all,
                    &what,
                );
            }
        }
    }
}

#[test]
fn card_size_falls_back_only_when_compression_is_not_enough() {
    let body = chrome(Rect::new(0, 0, 100, 60)).body;
    let yukon = Variant::Yukon(YukonRules::Classic);
    let base = deal(yukon, 9).piles();
    let fits_huge = with_cards(base.clone(), PileRef::Tableau(6), column(6, 33));
    assert_eq!(compute(yukon, &fits_huge, body).card, SIZES[0]);
    let needs_large = with_cards(base.clone(), PileRef::Tableau(6), column(6, 34));
    assert_eq!(compute(yukon, &needs_large, body).card, SIZES[1]);
    let needs_medium = with_cards(base.clone(), PileRef::Tableau(6), column(6, 38));
    assert_eq!(compute(yukon, &needs_medium, body).card, SIZES[2]);
    let needs_small = with_cards(base, PileRef::Tableau(6), column(6, 43));
    assert_eq!(compute(yukon, &needs_small, body).card, SIZES[3]);

    let spider = Variant::Spider(SuitCount::Four);
    let spider_base = deal(spider, 9).piles();
    let fits_medium = with_cards(spider_base.clone(), PileRef::Tableau(0), column(5, 42));
    assert_eq!(compute(spider, &fits_medium, body).card, SIZES[2]);
    let needs_small_spider = with_cards(spider_base, PileRef::Tableau(0), column(5, 43));
    assert_eq!(compute(spider, &needs_small_spider, body).card, SIZES[3]);
}

#[test]
fn single_leading_face_down_card_stays_visible_when_squeezed() {
    let variant = Variant::Yukon(YukonRules::Classic);
    let screen = Rect::new(0, 0, 100, 30);
    let parts = chrome(screen);
    let piles = with_cards(deal(variant, 9).piles(), PileRef::Tableau(6), column(1, 22));
    let geometry = compute(variant, &piles, parts.body);
    let placed = match geometry.pile(PileRef::Tableau(6)) {
        Some(placed) => placed,
        None => unreachable!("pile exists"),
    };
    assert_eq!(placed.collapsed, 0);
    assert!(placed.cards[1].y > placed.cards[0].y);
    assert_eq!(
        geometry.hit(placed.cards[0].x + 1, placed.cards[0].y),
        Some(Hit {
            pile: PileRef::Tableau(6),
            card: Some(0),
        })
    );
    let fixture = Fixture::new();
    let lines = render_board(&piles, &geometry, screen, &fixture.ctx());
    let down_row = row_slice(
        &lines,
        placed.cards[0].x,
        placed.cards[0].y,
        placed.cards[0].width,
    );
    assert!(
        down_row.starts_with('┌') && down_row.ends_with('┐'),
        "{down_row}"
    );
}

#[test]
fn compressed_yukon_column_shows_collapsed_bar_ranks_and_full_last_card() {
    let variant = Variant::Yukon(YukonRules::Classic);
    let screen = Rect::new(0, 0, 100, 30);
    let parts = chrome(screen);
    let piles = with_cards(deal(variant, 9).piles(), PileRef::Tableau(6), column(6, 18));
    let geometry = compute(variant, &piles, parts.body);
    let placed = match geometry.pile(PileRef::Tableau(6)) {
        Some(placed) => placed,
        None => unreachable!("pile exists"),
    };
    let fixture = Fixture::new();
    let lines = render_board(&piles, &geometry, screen, &fixture.ctx());
    let rows: Vec<String> = (placed.area.top()..placed.area.bottom())
        .map(|y| row_slice(&lines, placed.slot.x, y, placed.slot.width))
        .collect();
    assert_eq!(
        rows,
        vec![
            "▒ x6 ▒▒",
            "┌9♣───┐",
            "┌10♦──┐",
            "┌J♥───┐",
            "┌Q♠───┐",
            "┌K♣───┐",
            "┌A♦───┐",
            "┌2♥───┐",
            "┌3♠───┐",
            "┌4♣───┐",
            "┌5♦───┐",
            "┌6♥───┐",
            "┌7♠───┐",
            "┌8♣───┐",
            "┌9♦───┐",
            "┌10♥──┐",
            "┌J♠───┐",
            "│     │",
            "│  ♠  │",
            "│     │",
            "└───J♠┘",
        ]
    );
}
