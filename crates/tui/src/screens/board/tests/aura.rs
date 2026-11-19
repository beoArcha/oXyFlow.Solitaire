use std::collections::HashSet;

use oxysol_core::{DrawCount, Variant};
use ratatui::layout::{Position, Rect};

use super::*;
use crate::layout::{self, BoardGeometry};
use crate::settings::{AuraMode, Settings};
use crate::storage::memory::MemoryStorage;
use crate::test_support::TRUE_COLOR;
use crate::theme::{Theme, ThemeId, Token};
use crate::widgets::aura;
use crate::widgets::pile::CardPos;

type Cells = HashSet<(u16, u16)>;

const SIZES: [(u16, u16); 2] = [(120, 40), (140, 40)];

fn start_sized(variant: Variant, width: u16, height: u16) -> Harness {
    let mut h = Harness::with(MemoryStorage::new(), TRUE_COLOR, width, height);
    h.app.apply(Effect::Start {
        variant,
        seed: Some(5),
        undo_allowed: true,
    });
    h.lines();
    h
}

fn geometry(h: &Harness, width: u16, height: u16) -> (BoardGeometry, Rect) {
    let body = layout::chrome_with(Rect::new(0, 0, width, height), true).body;
    let session = session(h);
    let geometry = layout::compute(session.variant(), &session.game().piles(), body);
    (geometry, body)
}

fn painted(h: &mut Harness, glyph: &str, token: Token) -> Cells {
    h.lines();
    let theme = Theme::new(ThemeId::Dark, TRUE_COLOR.color);
    let wanted = theme.color(token);
    let buffer = h.terminal.backend().buffer();
    let area = buffer.area;
    let mut cells = Cells::new();
    for y in 0..area.height {
        for x in 0..area.width {
            let cell = &buffer[(x, y)];
            if cell.symbol() == glyph && cell.fg == wanted {
                cells.insert((x, y));
            }
        }
    }
    cells
}

fn ring_of(geometry: &BoardGeometry, body: Rect, pos: CardPos) -> Cells {
    aura::run_rect(geometry, pos)
        .map(|rect| aura::ring(rect, body).collect())
        .unwrap_or_default()
}

fn on_a_card(geometry: &BoardGeometry, (x, y): (u16, u16)) -> bool {
    let at = Position::new(x, y);
    geometry
        .piles
        .iter()
        .any(|placed| placed.cards.iter().any(|rect| rect.contains(at)))
}

#[test]
fn the_focused_card_has_a_halo_in_every_variant_and_nowhere_else() {
    for variant in Variant::ALL {
        for (width, height) in SIZES {
            let mut h = start_sized(variant, width, height);
            let (geometry, body) = geometry(&h, width, height);
            let Some(focus) = h.app.board().marks().focus else {
                unreachable!("a started board has a focus")
            };
            let halo = painted(&mut h, "░", Token::AuraFocus);
            let ring = ring_of(&geometry, body, focus);
            let label = format!("{variant:?} {width}x{height}");
            assert!(!halo.is_empty(), "{label}: no halo");
            assert!(halo.is_subset(&ring), "{label}: halo outside the ring");
            assert!(
                halo.iter().all(|cell| !on_a_card(&geometry, *cell)),
                "{label}: halo over a card"
            );
        }
    }
}

#[test]
fn the_halo_follows_the_cursor() {
    let mut h = start_sized(Variant::Klondike(DrawCount::One), 120, 40);
    let before = painted(&mut h, "░", Token::AuraFocus);
    h.press(ratatui::crossterm::event::KeyCode::Right);
    let after = painted(&mut h, "░", Token::AuraFocus);
    assert!(!after.is_empty());
    assert_ne!(before, after);
}

#[test]
fn a_carried_run_and_the_drop_targets_get_their_own_halos() {
    let mut h = start_sized(Variant::Klondike(DrawCount::One), 140, 40);
    let Some((from, to, _)) = first_transfer(&h) else {
        unreachable!("seed 5 has a transfer")
    };
    put_cursor(&mut h, from);
    h.press(ratatui::crossterm::event::KeyCode::Enter);
    put_cursor(&mut h, to);
    let (geometry, body) = geometry(&h, 140, 40);
    let marks = h.app.board().marks();
    let (Some(selected), Some(focus)) = (marks.selected, marks.focus) else {
        unreachable!("a carried run and a focus exist")
    };

    let carry = painted(&mut h, "▓", Token::AuraCarry);
    assert!(!carry.is_empty(), "no carry halo");
    assert!(carry.is_subset(&ring_of(&geometry, body, selected)));

    let halo = painted(&mut h, "░", Token::AuraFocus);
    assert!(!halo.is_empty(), "no focus halo on the drop target");
    assert!(halo.is_subset(&ring_of(&geometry, body, focus)));

    let target_rings: Cells = marks
        .targets
        .iter()
        .flat_map(|pile| {
            let index = pile_len(&h, *pile).saturating_sub(1);
            ring_of(&geometry, body, CardPos { pile: *pile, index })
        })
        .collect();
    let weak = painted(&mut h, "·", Token::Target);
    assert!(
        weak.is_subset(&target_rings),
        "weak halo outside the targets"
    );
    assert!(
        weak.is_disjoint(&carry) && weak.is_disjoint(&halo),
        "the weak halo must not cover a stronger one"
    );
}

#[test]
fn no_halo_without_a_focus_or_while_paused() {
    let mut h = start_sized(Variant::Klondike(DrawCount::One), 120, 40);
    h.type_char('p');
    assert!(painted(&mut h, "░", Token::AuraFocus).is_empty());
    assert!(painted(&mut h, "▓", Token::AuraCarry).is_empty());
}

#[test]
fn focus_only_keeps_the_focus_halo_and_drops_the_carry_and_target_halos() {
    let storage = MemoryStorage {
        settings: Some(Settings {
            aura: AuraMode::FocusOnly,
            ..Settings::default()
        }),
        ..MemoryStorage::default()
    };
    let mut h = Harness::with(storage, TRUE_COLOR, 140, 40);
    h.app.apply(Effect::Start {
        variant: Variant::Klondike(DrawCount::One),
        seed: Some(5),
        undo_allowed: true,
    });
    let Some((from, to, _)) = first_transfer(&h) else {
        unreachable!("seed 5 has a transfer")
    };
    put_cursor(&mut h, from);
    h.press(ratatui::crossterm::event::KeyCode::Enter);
    put_cursor(&mut h, to);
    assert!(board(&h).marks().selected.is_some(), "no carried run");
    assert!(!board(&h).marks().targets.is_empty(), "no drop target");

    assert!(painted(&mut h, "▓", Token::AuraCarry).is_empty());
    assert!(painted(&mut h, "·", Token::Target).is_empty());
    assert!(!painted(&mut h, "░", Token::AuraFocus).is_empty());
}
