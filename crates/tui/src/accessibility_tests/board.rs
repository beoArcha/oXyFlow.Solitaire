//! Board checks: the rendered board tells focus, selection, targets, face-down cards and the four
//! suits apart without color, and every pile is reached with the arrow keys.

use std::collections::{HashSet, VecDeque};

use oxysol_core::{DrawCount, Move, PileRef, Session, Suit, Variant};
use ratatui::crossterm::event::KeyCode;

use ratatui::layout::{Position, Rect};

use crate::announce::pile_name;
use crate::glyphs::{Glyphs, is_wgl4};
use crate::layout;
use crate::screens::Effect;
use crate::settings::GlyphMode;
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, PLAIN, TRUE_COLOR};
use crate::widgets::aura;
use crate::widgets::pile::CardPos;

const ARROWS: [KeyCode; 4] = [KeyCode::Up, KeyCode::Down, KeyCode::Left, KeyCode::Right];

const SUITS: [Suit; 4] = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades];

fn is_wgl4_or_ascii(symbol: char) -> bool {
    symbol.is_ascii() || is_wgl4(symbol)
}

fn count(lines: &[String], symbol: &str) -> usize {
    lines.iter().map(|line| line.matches(symbol).count()).sum()
}

fn klondike_seed() -> Option<u64> {
    (1..5_000_u64).find(|seed| {
        let session = Session::new(Variant::Klondike(DrawCount::One), *seed);
        let faces: HashSet<Suit> = session
            .game()
            .piles()
            .iter()
            .flat_map(|snapshot| snapshot.cards.iter())
            .filter(|table| table.face_up)
            .map(|table| table.card.suit)
            .collect();
        let movable = session.game().legal_moves().iter().any(|mv| {
            matches!(
                mv,
                Move::Transfer {
                    from: PileRef::Tableau(0),
                    count: 1,
                    ..
                }
            )
        });
        movable && SUITS.iter().all(|suit| faces.contains(suit))
    })
}

fn klondike(caps: crate::caps::TerminalCaps, seed: u64) -> Harness {
    let mut h = Harness::with(MemoryStorage::new(), caps, 120, 40);
    h.app.apply(Effect::Start {
        variant: Variant::Klondike(DrawCount::One),
        seed: Some(seed),
    });
    h
}

#[test]
fn focus_selection_targets_face_down_cards_and_suits_read_without_color() {
    let Some(seed) = klondike_seed() else {
        unreachable!("a seed with a movable first column and all four suits exists")
    };
    for caps in [TRUE_COLOR, PLAIN] {
        let glyphs = Glyphs::new(if caps.unicode {
            GlyphMode::Unicode
        } else {
            GlyphMode::Ascii
        });
        let mut h = klondike(caps, seed);
        let resting = h.lines();
        h.press(KeyCode::Enter);
        let held = h.lines();

        assert!(count(&resting, glyphs.focus_left) >= 1, "focus");
        assert_eq!(count(&resting, glyphs.selected), 0, "nothing selected");
        assert_eq!(count(&resting, glyphs.target), 0, "no targets at rest");
        assert!(count(&held, glyphs.selected) >= 1, "selection");
        assert!(count(&held, glyphs.target) >= 1, "legal targets");
        assert!(count(&resting, glyphs.face_down) >= 1, "face-down cards");
        assert_ne!(resting, held, "picking up changes the drawing");
        if caps.unicode {
            for suit in SUITS {
                assert!(count(&resting, glyphs.suit(suit)) >= 1, "{suit:?} symbol");
            }
        }
        for ch in resting.iter().chain(&held).flat_map(|line| line.chars()) {
            assert!(is_wgl4_or_ascii(ch), "{ch:?} is outside WGL4");
        }
    }
}

fn pile_is_focusable(snapshot: &oxysol_core::PileSnapshot) -> bool {
    !(snapshot.cards.is_empty() && matches!(snapshot.pile, PileRef::Pyramid { .. }))
}

fn pile_names(variant: Variant) -> Vec<String> {
    Session::new(variant, 1)
        .game()
        .piles()
        .iter()
        .filter(|snapshot| pile_is_focusable(snapshot))
        .map(|snapshot| pile_name(snapshot.pile))
        .collect()
}

/// The focus as the status line announces it, for example `Tableau 3: seven of hearts`. The
/// announcement names the card under the cursor, so two depths of one column are two positions.
fn focus_at(h: &mut Harness, names: &[String]) -> Option<String> {
    h.lines().iter().find_map(|line| {
        names.iter().find_map(|name| {
            let start = line.find(&format!("{name}: "))?;
            let rest = &line[start..];
            Some(rest.split(". ").next()?.trim_end().to_owned())
        })
    })
}

fn focus_after(variant: Variant, path: &[KeyCode], names: &[String]) -> Option<String> {
    let mut h = Harness::new();
    h.app.apply(Effect::Start {
        variant,
        seed: Some(1),
    });
    for key in path {
        h.press(*key);
    }
    focus_at(&mut h, names)
}

fn reached_focus(variant: Variant, names: &[String]) -> HashSet<String> {
    let mut reached = HashSet::new();
    let mut queue: VecDeque<Vec<KeyCode>> = VecDeque::from([Vec::new()]);
    while let Some(path) = queue.pop_front() {
        let Some(focus) = focus_after(variant, &path, names) else {
            continue;
        };
        if !reached.insert(focus) {
            continue;
        }
        for key in ARROWS {
            let mut next = path.clone();
            next.push(key);
            queue.push_back(next);
        }
    }
    reached
}

#[test]
fn every_pile_of_every_variant_is_reached_with_the_arrow_keys() {
    for variant in Variant::ALL {
        let names = pile_names(variant);
        let reached = reached_focus(variant, &names);
        for name in &names {
            let prefix = format!("{name}: ");
            assert!(
                reached.iter().any(|focus| focus.starts_with(&prefix)),
                "{variant:?}: {name} is not reached; reached {reached:?}"
            );
        }
    }
}

fn halo_symbols(h: &mut Harness, pos: CardPos) -> Vec<String> {
    let lines = h.lines();
    let area = Rect::new(0, 0, 120, 40);
    let body = layout::chrome(area).body;
    let Some(session) = h.app.board().session() else {
        unreachable!("a game is running")
    };
    let geometry = layout::compute(session.variant(), &session.game().piles(), body);
    let Some(rect) = aura::run_rect(&geometry, pos) else {
        unreachable!("the focused pile has a rectangle")
    };
    let taken = |x: u16, y: u16| {
        let at = Position::new(x, y);
        geometry
            .piles
            .iter()
            .any(|placed| placed.cards.iter().any(|card| card.contains(at)))
    };
    aura::ring(rect, body)
        .filter(|(x, y)| !taken(*x, *y))
        .map(|(x, y)| {
            lines[usize::from(y)]
                .chars()
                .nth(usize::from(x))
                .map(String::from)
                .unwrap_or_default()
        })
        .collect()
}

#[test]
fn the_halo_of_the_focus_and_of_a_carried_run_read_without_color() {
    let Some(seed) = klondike_seed() else {
        unreachable!("a seed with a movable first column and all four suits exists")
    };
    for caps in [TRUE_COLOR, PLAIN] {
        let glyphs = Glyphs::new(if caps.unicode {
            GlyphMode::Unicode
        } else {
            GlyphMode::Ascii
        });
        let auras = [glyphs.aura_focus, glyphs.aura_carry, glyphs.aura_target];
        assert_eq!(auras.iter().collect::<HashSet<_>>().len(), 3, "told apart");
        assert!(!auras.contains(&glyphs.face_down) && !auras.contains(&glyphs.selected));

        let mut h = klondike(caps, seed);
        let Some(focus) = h.app.board().marks().focus else {
            unreachable!("a started board has a focus")
        };
        let around = halo_symbols(&mut h, focus);
        assert!(!around.is_empty(), "free cells around the focus");
        assert!(around.iter().all(|symbol| symbol == glyphs.aura_focus));

        h.press(KeyCode::Enter);
        let Some(selected) = h.app.board().marks().selected else {
            unreachable!("the first column can be picked up")
        };
        let around = halo_symbols(&mut h, selected);
        assert!(!around.is_empty(), "free cells around the carried run");
        assert!(around.iter().all(|symbol| symbol == glyphs.aura_carry));
    }
}

#[test]
fn the_guide_panel_adds_only_wgl4_or_ascii_symbols() {
    let Some(seed) = klondike_seed() else {
        unreachable!("a seed with a movable first column and all four suits exists")
    };
    for caps in [TRUE_COLOR, PLAIN] {
        let mut h = Harness::with(MemoryStorage::new(), caps, 140, 40);
        h.app.apply(Effect::Start {
            variant: Variant::Klondike(DrawCount::One),
            seed: Some(seed),
        });
        h.press(KeyCode::Enter);
        let text = h.text();
        assert!(text.contains("Guide"), "the panel is drawn");
        for ch in text.chars().filter(|ch| *ch != '\n') {
            assert!(is_wgl4_or_ascii(ch), "{ch:?} is outside WGL4");
            if !caps.unicode {
                assert!(ch.is_ascii(), "{ch:?} in ASCII mode");
            }
        }
    }
}
