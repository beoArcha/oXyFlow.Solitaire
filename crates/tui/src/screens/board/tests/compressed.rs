use oxysol_core::{DrawCount, Move, PileRef, SuitCount, Variant, YukonRules};
use ratatui::crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use super::*;
use crate::input::Input;
use crate::settings::{CardSizeSetting, Settings};
use crate::test_support::{Harness, TRUE_COLOR};

fn board_at(variant: Variant, seed: u64, size: (u16, u16), card_size: CardSizeSetting) -> Harness {
    let storage = MemoryStorage {
        settings: Some(Settings {
            card_size,
            ..Settings::default()
        }),
        ..MemoryStorage::default()
    };
    let mut h = Harness::with(storage, TRUE_COLOR, size.0, size.1);
    h.app.apply(Effect::Start {
        variant,
        seed: Some(seed),
        undo_allowed: true,
    });
    h.lines();
    h
}

fn click(h: &mut Harness, column: u16, row: u16) {
    let event = |kind| {
        Input::Mouse(MouseEvent {
            kind,
            column,
            row,
            modifiers: KeyModifiers::NONE,
        })
    };
    h.send(event(MouseEventKind::Down(MouseButton::Left)));
    h.send(event(MouseEventKind::Up(MouseButton::Left)));
}

fn grow_column(h: &mut Harness, target: PileRef, min_cards: usize) {
    for _ in 0..40 {
        if pile_len(h, target) >= min_cards {
            return;
        }
        let next = legal(h).into_iter().find(|mv| match mv {
            Move::Transfer { from, to, count } => *to == target && *from != target && *count >= 1,
            _ => false,
        });
        let Some(mv) = next else {
            break;
        };
        assert_eq!(h.app.board_mut().play(mv), Effect::None);
    }
}

#[test]
fn compressed_yukon_column_renders_above_status_bar_and_picks_clicked_card() {
    let yukon = Variant::Yukon(YukonRules::Classic);
    for (width, height) in [(100_u16, 30_u16), (100_u16, 60_u16)] {
        let mut h = board_at(yukon, 9, (width, height), CardSizeSetting::Large);
        grow_column(&mut h, PileRef::Tableau(6), 14);
        let lines = h.lines();
        let piles = session(&h).game().piles();
        let geometry = match board(&h).geometry(&piles) {
            Some(geometry) => geometry,
            None => unreachable!("board has geometry"),
        };
        let placed = match geometry.pile(PileRef::Tableau(6)) {
            Some(placed) => placed,
            None => unreachable!("tableau 6 exists"),
        };
        let last = match placed.cards.last().copied() {
            Some(last) => last,
            None => unreachable!("tableau 6 has cards"),
        };
        let now_row = usize::from(height - 2);
        assert!(lines[now_row].starts_with("Now: "), "{}", lines[now_row]);
        assert!(last.bottom() <= height - 2);
        let Some(snapshot) = piles.iter().find(|s| s.pile == PileRef::Tableau(6)) else {
            unreachable!("tableau 6 exists");
        };
        assert!(snapshot.cards.len() >= 14, "{}", snapshot.cards.len());
        let first_up = snapshot
            .cards
            .iter()
            .position(|card| card.face_up)
            .unwrap_or(0);
        let step = placed.cards[first_up + 1].y - placed.cards[first_up].y;
        assert!(step < geometry.card.up_step(), "step={step}");
        let rect = placed.cards[first_up];
        click(&mut h, rect.x + 1, rect.y);
        assert_eq!(
            cursor(&h),
            (PileRef::Tableau(6), snapshot.cards.len() - first_up)
        );
        let mid_up = (first_up + snapshot.cards.len() - 1) / 2;
        if mid_up > first_up {
            let mid_rect = placed.cards[mid_up];
            click(&mut h, mid_rect.x + 1, mid_rect.y);
            assert_eq!(
                cursor(&h),
                (PileRef::Tableau(6), snapshot.cards.len() - mid_up)
            );
        }
        if placed.collapsed > 0 {
            let down_rect = placed.cards[0];
            click(&mut h, down_rect.x + 1, down_rect.y);
            assert_eq!(status(&h), Some("board.error.face_down"));
        }
    }
}

#[test]
fn klondike_and_spider_boards_keep_last_card_above_status_bar() {
    for variant in [
        Variant::Klondike(DrawCount::Three),
        Variant::Spider(SuitCount::Four),
    ] {
        for size in [(100_u16, 30_u16), (100_u16, 60_u16)] {
            for card_size in CardSizeSetting::ALL {
                let mut h = board_at(variant, 9, size, card_size);
                let lines = h.lines();
                let piles = session(&h).game().piles();
                let geometry = match board(&h).geometry(&piles) {
                    Some(geometry) => geometry,
                    None => unreachable!("board has geometry"),
                };
                let now_row = size.1 - 2;
                assert!(lines[usize::from(now_row)].starts_with("Now: "));
                for placed in &geometry.piles {
                    if let Some(last) = placed.cards.last() {
                        assert!(
                            last.bottom() <= now_row,
                            "{variant:?} {size:?} {card_size:?}"
                        );
                    }
                }
            }
        }
    }
}
