use oxysol_core::{DrawCount, Move, MoveError, PileRef, PyramidRules, Variant};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::*;
use crate::app::{Modal, Screen};
use crate::i18n::{Locale, set_locale, t};
use crate::input::Input;
use crate::screens::board::status::{self, error_key};

const KLONDIKE: Variant = Variant::Klondike(DrawCount::One);
const SEED: u64 = 5;

fn play_one(h: &mut Harness) -> Move {
    let Some((from, to, count)) = first_transfer(h) else {
        unreachable!("seed 5 has a transfer")
    };
    assert_eq!(count, 1);
    drag(h, from, to);
    Move::Transfer { from, to, count }
}

#[test]
fn undo_and_redo_walk_the_history() {
    let mut h = start(KLONDIKE, SEED);
    let dealt = session(&h).game().piles();
    let mv = play_one(&mut h);
    let played = session(&h).game().piles();
    assert_ne!(dealt, played);

    h.type_char('u');
    assert_eq!(session(&h).game().piles(), dealt);
    assert_eq!(moves_made(&h), 0);
    assert_eq!(cues(&h).last(), Some(&SoundCue::Undo));

    h.type_char('r');
    assert_eq!(session(&h).game().piles(), played);
    assert_eq!(session(&h).record().moves, vec![mv]);
}

#[test]
fn control_z_and_control_y_undo_and_redo() {
    let mut h = start(KLONDIKE, SEED);
    play_one(&mut h);
    h.send(Input::Key(KeyEvent::new(
        KeyCode::Char('z'),
        KeyModifiers::CONTROL,
    )));
    assert_eq!(moves_made(&h), 0);
    h.send(Input::Key(KeyEvent::new(
        KeyCode::Char('y'),
        KeyModifiers::CONTROL,
    )));
    assert_eq!(moves_made(&h), 1);
}

#[test]
fn nothing_to_undo_or_redo_is_a_message() {
    let mut h = start(KLONDIKE, SEED);
    h.type_char('u');
    assert_eq!(status(&h), Some(status::NOTHING_TO_UNDO));
    assert!(h.text().contains("Nothing to undo."));
    h.type_char('r');
    assert_eq!(status(&h), Some(status::NOTHING_TO_REDO));
    assert_eq!(cues(&h), vec![SoundCue::Illegal, SoundCue::Illegal]);
    assert_eq!(moves_made(&h), 0);
}

#[test]
fn a_new_move_after_undo_drops_the_redo_tail() {
    let mut h = start(KLONDIKE, SEED);
    play_one(&mut h);
    h.type_char('u');
    h.type_char('s');
    h.type_char('r');
    assert_eq!(status(&h), Some(status::NOTHING_TO_REDO));
}

#[test]
fn the_hint_highlights_its_cards_and_stays_while_repeated() {
    let mut h = start(KLONDIKE, SEED);
    let Some(Move::Transfer { from, to, .. }) = session(&h).game().hint() else {
        unreachable!("seed 5 hints a transfer")
    };
    h.type_char('h');
    let marks = board(&h).marks();
    assert!(marks.hints.iter().any(|card| card.pile == from));
    assert!(marks.hints.iter().any(|card| card.pile == to));
    assert_eq!(status(&h), None);
    assert!(h.text().contains("Hint: "));
    h.type_char('h');
    h.type_char('h');
    assert_eq!(cues(&h), vec![SoundCue::Hint]);
    assert_eq!(board(&h).marks(), marks);

    drag(&mut h, from, to);
    assert!(board(&h).marks().hints.is_empty());
}

#[test]
fn a_stock_hint_names_the_stock() {
    let mut session = oxysol_core::Session::new(KLONDIKE, SEED);
    let mut prefix = Vec::new();
    while let Some(mv) = session.game().hint().filter(|mv| *mv != Move::Stock) {
        assert_eq!(session.apply(mv), Ok(()));
        prefix.push(mv);
    }
    assert_eq!(session.game().hint(), Some(Move::Stock));
    let mut h = resumed(KLONDIKE, SEED, &prefix);
    h.type_char('h');
    assert_eq!(status(&h), None);
    assert!(h.text().contains("Hint: draw from the stock"));
    let marks = board(&h).marks();
    assert!(marks.hints.iter().all(|card| card.pile == PileRef::Stock));
    assert!(!marks.hints.is_empty());
}

#[test]
fn no_legal_moves_gives_a_message_and_no_hint() {
    let moves = hint_moves(Variant::Pyramid(PyramidRules::Relaxed), 0);
    let mut h = resumed(Variant::Pyramid(PyramidRules::Relaxed), 0, &moves);
    assert!(legal(&h).is_empty());
    assert_eq!(status(&h), None);
    assert!(h.text().contains("No legal moves"));
    h.type_char('h');
    assert_eq!(status(&h), Some(status::NO_HINT));
    assert_eq!(cues(&h), vec![SoundCue::Illegal]);
    h.type_char('u');
    assert_eq!(status(&h), Some("board.status.undone"));
}

#[test]
fn pause_stops_the_clock_and_hides_the_board() {
    let mut h = start(KLONDIKE, SEED);
    h.advance(10);
    assert!(h.text().contains("seed 5 | Time 0:10"));
    assert!(
        h.text().contains('♥')
            || h.text().contains('♠')
            || h.text().contains('♦')
            || h.text().contains('♣')
    );
    h.type_char('p');
    assert_eq!(h.app.modal(), Some(&Modal::Pause));
    assert!(!board(&h).clock_running());
    h.advance(60);
    let paused = h.text();
    assert!(paused.contains("seed 5 | Time 0:10"));
    for suit in ['♥', '♠', '♦', '♣'] {
        assert!(!paused.contains(suit), "{suit}");
    }
    h.press(KeyCode::Esc);
    assert!(h.app.modal().is_none());
    h.advance(5);
    assert!(h.text().contains("seed 5 | Time 0:15"));
}

#[test]
fn p_closes_the_pause_again() {
    let mut h = start(KLONDIKE, SEED);
    h.type_char('p');
    h.type_char('p');
    assert!(h.app.modal().is_none());
    assert!(board(&h).clock_running());
}

#[test]
fn esc_leaves_for_the_menu_and_m_too() {
    let mut h = start(KLONDIKE, SEED);
    h.press(KeyCode::Esc);
    assert_eq!(h.app.screen(), Screen::MainMenu);
    let mut h = start(KLONDIKE, SEED);
    h.type_char('m');
    assert_eq!(h.app.screen(), Screen::MainMenu);
    assert!(!board(&h).clock_running());
}

#[test]
fn restart_asks_first_and_deals_the_same_seed_again() {
    let mut h = start(KLONDIKE, SEED);
    let dealt = session(&h).game().piles();
    play_one(&mut h);
    h.advance(30);
    h.type_char('R');
    assert!(matches!(h.app.modal(), Some(Modal::Confirm { .. })));
    assert!(
        h.text()
            .contains("Deal this game again from the beginning?")
    );
    h.press(KeyCode::Esc);
    assert_eq!(moves_made(&h), 1);

    h.type_char('R');
    h.press(KeyCode::Left);
    h.press(KeyCode::Enter);
    assert!(h.app.modal().is_none());
    assert_eq!(session(&h).game().piles(), dealt);
    assert_eq!(moves_made(&h), 0);
    assert_eq!(session(&h).seed(), SEED);
    assert!(h.text().contains("seed 5 | Time 0:00"));
    assert!(board(&h).clock_running());
    assert_eq!(cursor(&h), (PileRef::Tableau(0), 1));
}

#[test]
fn a_new_game_asks_first_and_takes_a_fresh_seed() {
    let mut h = start(KLONDIKE, SEED);
    h.type_char('n');
    assert!(matches!(h.app.modal(), Some(Modal::Confirm { .. })));
    h.press(KeyCode::Enter);
    assert_eq!(session(&h).seed(), SEED);
    h.type_char('n');
    h.press(KeyCode::Right);
    h.press(KeyCode::Enter);
    assert_eq!(session(&h).seed(), 42);
    assert_eq!(session(&h).variant(), KLONDIKE);
    assert_eq!(moves_made(&h), 0);
}

#[test]
fn q_asks_before_quitting() {
    let mut h = start(KLONDIKE, SEED);
    h.type_char('q');
    assert!(!h.app.should_quit());
    h.press(KeyCode::Right);
    h.press(KeyCode::Enter);
    assert!(h.app.should_quit());
}

#[test]
fn the_last_move_wins_the_game() {
    let variant = Variant::Pyramid(PyramidRules::Relaxed);
    let moves = hint_moves(variant, 2);
    let Some((last, earlier)) = moves.split_last() else {
        unreachable!("seed 2 is solved by hints")
    };
    let mut h = resumed(variant, 2, earlier);
    assert!(h.app.modal().is_none());
    h.advance(20);
    let Move::Remove { first, second } = *last else {
        unreachable!("pyramid moves are removals")
    };
    match second {
        Some(second) if first == PileRef::Stock => drag(&mut h, second, first),
        Some(second) => drag(&mut h, first, second),
        None => drag(&mut h, first, first),
    }
    assert!(session(&h).game().is_won());
    assert_eq!(h.app.modal(), Some(&Modal::win()));
    assert_eq!(cues(&h).last(), Some(&SoundCue::Win));
    assert!(!board(&h).clock_running());
    h.advance(100);
    h.press(KeyCode::Esc);
    assert!(h.app.modal().is_none());
    assert!(!board(&h).clock_running());
    h.type_char('s');
    assert_eq!(status(&h), Some(status::SOLVED));
}

#[test]
fn every_move_error_has_a_message_in_every_locale() {
    let pile = PileRef::Waste;
    let errors = [
        MoveError::Unsupported,
        MoveError::NoSuchPile(pile),
        MoveError::EmptySource(pile),
        MoveError::FaceDown,
        MoveError::InvalidSequence,
        MoveError::TargetRejects,
        MoveError::StockEmpty,
        MoveError::EmptyColumnBlocksDeal,
        MoveError::PassesExhausted,
        MoveError::CountOutOfRange { count: 3 },
        MoveError::NotEnoughSpace,
        MoveError::NotAPair,
        MoveError::Blocked,
    ];
    let mut seen = Vec::new();
    for locale in Locale::ALL {
        set_locale(locale);
        for error in &errors {
            let key = error_key(error);
            assert_ne!(t(key), key, "{key}");
            seen.push(key);
        }
    }
    set_locale(Locale::EnUs);
    seen.sort_unstable();
    seen.dedup();
    assert_eq!(seen.len(), errors.len());
}

#[test]
fn every_status_key_exists_in_every_locale() {
    let keys = [
        status::NO_TARGETS,
        status::NOTHING_TO_UNDO,
        status::NOTHING_TO_REDO,
        status::NO_HINT,
        status::SOLVED,
        "board.confirm.restart",
        "board.confirm.new_game",
        "board.confirm.quit",
    ];
    for locale in Locale::ALL {
        set_locale(locale);
        for key in keys {
            assert_ne!(t(key), key, "{key}");
        }
    }
    set_locale(Locale::EnUs);
}
