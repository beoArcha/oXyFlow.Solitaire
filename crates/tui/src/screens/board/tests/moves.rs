use oxysol_core::{DrawCount, Move, PileRef, SuitCount, Variant};
use ratatui::crossterm::event::KeyCode;

use super::*;
use crate::screens::board::status;

const KLONDIKE: Variant = Variant::Klondike(DrawCount::One);
const SEED: u64 = 5;

fn single_card_transfer(h: &Harness) -> (PileRef, PileRef) {
    let found = legal(h).into_iter().find_map(|mv| match mv {
        Move::Transfer { from, to, count: 1 } if matches!(from, PileRef::Tableau(_)) => {
            Some((from, to))
        }
        _ => None,
    });
    found.unwrap_or((PileRef::Stock, PileRef::Stock))
}

#[test]
fn a_legal_move_is_picked_then_dropped() {
    let mut h = start(KLONDIKE, SEED);
    let (from, to) = single_card_transfer(&h);
    assert_ne!(from, PileRef::Stock);
    put_cursor(&mut h, from);
    h.press(KeyCode::Enter);
    let marks = board(&h).marks();
    assert!(marks.targets.contains(&to));
    let len = pile_len(&h, from);
    assert_eq!(
        marks.selected.map(|s| (s.pile, s.index)),
        Some((from, len - 1))
    );
    assert_eq!(status(&h), Some(status::PICKED));
    assert_eq!(cues(&h), vec![SoundCue::CardPick]);
    assert!(h.text().contains("Cards picked up"));

    put_cursor(&mut h, to);
    h.press(KeyCode::Enter);
    assert_eq!(moves_made(&h), 1);
    assert_eq!(
        session(&h).record().moves,
        vec![Move::Transfer { from, to, count: 1 }]
    );
    assert!(board(&h).marks().targets.is_empty());
    assert_eq!(status(&h), None);
    assert!(cues(&h).contains(&SoundCue::CardPlace));
    assert_eq!(cursor(&h).0, to);
}

#[test]
fn space_picks_and_drops_like_enter() {
    let mut h = start(KLONDIKE, SEED);
    let (from, to) = single_card_transfer(&h);
    put_cursor(&mut h, from);
    h.type_char(' ');
    put_cursor(&mut h, to);
    h.type_char(' ');
    assert_eq!(moves_made(&h), 1);
}

#[test]
fn picking_the_same_pile_again_or_esc_cancels() {
    let mut h = start(KLONDIKE, SEED);
    let (from, _) = single_card_transfer(&h);
    put_cursor(&mut h, from);
    h.press(KeyCode::Enter);
    h.press(KeyCode::Enter);
    assert!(board(&h).marks().selected.is_none());
    assert_eq!(status(&h), None);

    h.press(KeyCode::Enter);
    assert!(board(&h).marks().selected.is_some());
    h.press(KeyCode::Esc);
    assert!(board(&h).marks().selected.is_none());
    assert_eq!(h.app.screen(), crate::app::Screen::Board);
    assert_eq!(moves_made(&h), 0);
}

#[test]
fn a_drop_on_a_pile_that_refuses_shows_the_engine_message() {
    let mut h = start(KLONDIKE, SEED);
    let (from, to) = single_card_transfer(&h);
    put_cursor(&mut h, from);
    h.press(KeyCode::Enter);
    let refusing = (0..7)
        .map(PileRef::Tableau)
        .find(|pile| *pile != from && *pile != to && !board(&h).marks().targets.contains(pile));
    let Some(refusing) = refusing else {
        unreachable!("seed 5 has a column that refuses")
    };
    put_cursor(&mut h, refusing);
    h.press(KeyCode::Enter);
    assert_eq!(status(&h), Some("board.error.target_rejects"));
    assert!(h.text().contains("The cards cannot go there."));
    assert_eq!(moves_made(&h), 0);
    assert!(board(&h).marks().selected.is_some());
    assert_eq!(cues(&h), vec![SoundCue::CardPick, SoundCue::Illegal]);
    put_cursor(&mut h, to);
    h.press(KeyCode::Enter);
    assert_eq!(moves_made(&h), 1);
}

#[test]
fn picking_cards_with_nowhere_to_go_is_refused_without_a_log() {
    let mut h = start(KLONDIKE, SEED);
    let stuck = (0..7).map(PileRef::Tableau).find(|pile| {
        !legal(&h)
            .iter()
            .any(|mv| matches!(mv, Move::Transfer { from, .. } if from == pile))
    });
    let Some(stuck) = stuck else {
        unreachable!("seed 5 has a column without moves")
    };
    put_cursor(&mut h, stuck);
    h.press(KeyCode::Enter);
    assert_eq!(status(&h), Some(status::NO_TARGETS));
    assert_eq!(cues(&h), vec![SoundCue::Illegal]);
    assert!(board(&h).marks().selected.is_none());
}

#[test]
fn an_empty_pile_and_face_down_cards_have_their_own_messages() {
    let mut h = start(KLONDIKE, SEED);
    put_cursor(&mut h, PileRef::Waste);
    h.press(KeyCode::Enter);
    assert_eq!(status(&h), Some("board.error.empty_source"));
    put_cursor(&mut h, PileRef::Stock);
    h.type_char('a');
    assert_eq!(status(&h), Some("board.error.face_down"));
}

#[test]
fn the_stock_key_and_enter_on_the_stock_draw() {
    let mut h = start(KLONDIKE, SEED);
    let stock = pile_len(&h, PileRef::Stock);
    h.type_char('s');
    assert_eq!(pile_len(&h, PileRef::Stock), stock - 1);
    assert_eq!(pile_len(&h, PileRef::Waste), 1);
    put_cursor(&mut h, PileRef::Stock);
    h.press(KeyCode::Enter);
    assert_eq!(pile_len(&h, PileRef::Waste), 2);
    assert_eq!(cues(&h), vec![SoundCue::StockDeal, SoundCue::StockDeal]);
}

#[test]
fn an_empty_stock_is_a_message_not_a_log() {
    let mut h = start(Variant::Spider(SuitCount::One), 14);
    for _ in 0..5 {
        h.type_char('s');
        assert_eq!(status(&h), None);
    }
    assert_eq!(pile_len(&h, PileRef::Stock), 0);
    h.type_char('s');
    assert_eq!(status(&h), Some("board.error.stock_empty"));
    assert!(h.text().contains("The stock is empty."));
    assert_eq!(cues(&h).last(), Some(&SoundCue::Illegal));
    assert_eq!(moves_made(&h), 5);
}

#[test]
fn a_move_that_uncovers_a_card_flips_it() {
    let mut h = start(KLONDIKE, SEED);
    let (from, to) = (PileRef::Tableau(3), PileRef::Tableau(0));
    assert!(legal(&h).contains(&Move::Transfer { from, to, count: 1 }));
    drag(&mut h, from, to);
    assert_eq!(
        cues(&h),
        vec![SoundCue::CardPick, SoundCue::CardPlace, SoundCue::CardFlip]
    );
}

#[test]
fn quick_move_prefers_the_foundation_over_a_column() {
    let mut h = start(KLONDIKE, 3);
    let from = PileRef::Tableau(0);
    let moves = legal(&h);
    assert!(moves.iter().any(|mv| matches!(mv, Move::Transfer { from: f, to: PileRef::Foundation(_), count: 1 } if *f == from)));
    assert!(moves.iter().any(|mv| matches!(mv, Move::Transfer { from: f, to: PileRef::Tableau(_), count: 1 } if *f == from)));
    put_cursor(&mut h, from);
    h.type_char('a');
    let made = session(&h).record().moves;
    assert!(matches!(
        made.as_slice(),
        [Move::Transfer { from: f, to: PileRef::Foundation(_), count: 1 }] if *f == from
    ));
    assert_eq!(cues(&h)[0], SoundCue::CardPlace);
}

#[test]
fn quick_move_without_a_target_says_so() {
    let mut h = start(KLONDIKE, SEED);
    let stuck = (0..7).map(PileRef::Tableau).find(|pile| {
        !legal(&h)
            .iter()
            .any(|mv| matches!(mv, Move::Transfer { from, .. } if from == pile))
    });
    put_cursor(&mut h, stuck.unwrap_or(PileRef::Tableau(0)));
    h.type_char('a');
    assert_eq!(status(&h), Some(status::NO_TARGETS));
    assert_eq!(moves_made(&h), 0);
}

#[test]
fn a_run_is_picked_with_the_depth_keys() {
    let mut h = start(Variant::FreeCell, 27);
    let (from, to) = (PileRef::Tableau(6), PileRef::Tableau(2));
    assert!(legal(&h).contains(&Move::Transfer { from, to, count: 2 }));
    put_cursor(&mut h, from);
    h.type_char('+');
    h.press(KeyCode::Enter);
    let marks = board(&h).marks();
    let len = pile_len(&h, from);
    assert_eq!(marks.selected.map(|s| s.index), Some(len - 2));
    assert!(marks.targets.contains(&to));
    put_cursor(&mut h, to);
    h.press(KeyCode::Enter);
    assert_eq!(
        session(&h).record().moves,
        vec![Move::Transfer { from, to, count: 2 }]
    );
}

#[test]
fn a_row_is_dealt_in_spider() {
    let mut h = start(Variant::Spider(SuitCount::One), 14);
    let stock = pile_len(&h, PileRef::Stock);
    let columns: Vec<usize> = (0..10).map(|i| pile_len(&h, PileRef::Tableau(i))).collect();
    h.type_char('s');
    assert_eq!(pile_len(&h, PileRef::Stock), stock - 10);
    for (i, before) in columns.iter().enumerate() {
        assert_eq!(
            pile_len(&h, PileRef::Tableau(u8::try_from(i).unwrap_or(0))),
            before + 1
        );
    }
    assert_eq!(cues(&h), vec![SoundCue::StockDeal]);
    assert_eq!(moves_made(&h), 1);
}

#[test]
fn a_row_is_dealt_in_spider_with_enter_on_the_stock() {
    let mut h = start(Variant::Spider(SuitCount::One), 14);
    put_cursor(&mut h, PileRef::Stock);
    h.press(KeyCode::Enter);
    assert_eq!(session(&h).record().moves, vec![Move::Stock]);
}
