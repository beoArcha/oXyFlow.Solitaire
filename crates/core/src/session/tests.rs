use super::*;
use crate::card::{Card, CardId, Rank, Suit};
use crate::game::DrawCount;
use crate::pile::{PileRef, PileSnapshot, TableCard};

const LIMIT: u16 = 6;
const VARIANT: Variant = Variant::Klondike(DrawCount::One);

struct Counter {
    count: u16,
}

impl Game for Counter {
    fn apply(&mut self, mv: Move) -> Result<(), MoveError> {
        if mv != Move::Stock || self.count >= LIMIT {
            return Err(MoveError::Unsupported);
        }
        self.count += 1;
        Ok(())
    }

    fn legal_moves(&self) -> Vec<Move> {
        if self.count < LIMIT {
            vec![Move::Stock]
        } else {
            Vec::new()
        }
    }

    fn hint(&self) -> Option<Move> {
        self.legal_moves().into_iter().next()
    }

    fn is_won(&self) -> bool {
        self.count == LIMIT
    }

    fn piles(&self) -> Vec<PileSnapshot> {
        let cards = (0..self.count)
            .map(|id| TableCard {
                card: Card {
                    id: CardId(id),
                    suit: Suit::Spades,
                    rank: Rank::Ace,
                },
                face_up: true,
            })
            .collect();
        vec![PileSnapshot {
            pile: PileRef::Waste,
            cards,
        }]
    }
}

fn counter_deal(_: Variant, seed: u64) -> Box<dyn Game> {
    Box::new(Counter {
        count: u16::try_from(seed % 3).unwrap_or(0),
    })
}

fn session(seed: u64) -> Session {
    Session::with_dealer(VARIANT, seed, counter_deal)
}

fn count(session: &Session) -> usize {
    session.game().piles()[0].cards.len()
}

fn record(moves: usize, cursor: usize) -> SessionRecord {
    SessionRecord {
        variant: VARIANT,
        seed: 0,
        moves: vec![Move::Stock; moves],
        cursor,
    }
}

fn play(session: &mut Session, moves: usize) {
    for _ in 0..moves {
        assert_eq!(session.apply(Move::Stock), Ok(()));
    }
}

#[test]
fn new_session_is_the_deal_with_empty_history() {
    let session = session(4);
    assert_eq!(count(&session), 1);
    assert_eq!(session.variant(), VARIANT);
    assert_eq!(session.seed(), 4);
    let record = session.record();
    assert!(record.moves.is_empty());
    assert_eq!(record.cursor, 0);
}

#[test]
fn undo_and_redo_walk_the_history() {
    let mut session = session(0);
    play(&mut session, 3);
    assert_eq!(count(&session), 3);
    assert!(session.undo());
    assert!(session.undo());
    assert_eq!(count(&session), 1);
    assert!(session.redo());
    assert_eq!(count(&session), 2);
    assert_eq!(session.record().cursor, 2);
    assert_eq!(session.record().moves.len(), 3);
}

#[test]
fn undo_at_start_and_redo_at_end_do_nothing() {
    let mut session = session(0);
    assert!(!session.undo());
    assert!(!session.redo());
    play(&mut session, 1);
    assert!(!session.redo());
    assert!(session.undo());
    assert!(!session.undo());
    assert_eq!(count(&session), 0);
}

#[test]
fn a_new_move_truncates_the_redo_tail() {
    let mut session = session(0);
    play(&mut session, 3);
    assert!(session.undo());
    assert!(session.undo());
    play(&mut session, 1);
    let record = session.record();
    assert_eq!(record.moves.len(), 2);
    assert_eq!(record.cursor, 2);
    assert!(!session.redo());
    assert_eq!(count(&session), 2);
}

#[test]
fn a_rejected_move_keeps_game_and_history() {
    let mut session = session(0);
    play(&mut session, 2);
    assert!(session.undo());
    let before = session.record();
    let other = Move::Remove {
        first: PileRef::Waste,
        second: None,
    };
    assert_eq!(session.apply(other), Err(MoveError::Unsupported));
    assert_eq!(session.record(), before);
    assert!(session.redo());
    assert_eq!(count(&session), 2);
}

#[test]
fn restart_returns_to_the_same_deal() {
    let mut session = session(5);
    let dealt = session.game().piles();
    play(&mut session, 2);
    session.restart();
    assert_eq!(session.game().piles(), dealt);
    assert_eq!(session.seed(), 5);
    assert!(session.record().moves.is_empty());
    assert!(!session.undo());
    assert!(!session.redo());
}

#[test]
fn restore_reproduces_game_and_history() {
    let mut original = session(1);
    play(&mut original, 4);
    assert!(original.undo());
    let restored = Session::restore_with(original.record(), counter_deal);
    let Ok(mut restored) = restored else {
        panic!("a valid record must restore");
    };
    assert_eq!(restored.record(), original.record());
    assert_eq!(restored.game().piles(), original.game().piles());
    assert!(restored.redo());
    assert_eq!(count(&restored), 5);
}

#[test]
fn restore_rejects_a_cursor_beyond_the_log() {
    let error = Session::restore_with(record(2, 3), counter_deal).err();
    assert_eq!(
        error,
        Some(RestoreError::CursorBeyondMoves { cursor: 3, len: 2 })
    );
}

#[test]
fn restore_rejects_a_bad_move_and_keeps_its_cause() {
    let error = Session::restore_with(record(7, 7), counter_deal).err();
    let expected = RestoreError::MoveRejected {
        index: 6,
        source: MoveError::Unsupported,
    };
    assert_eq!(error, Some(expected));
    let source = std::error::Error::source(&RestoreError::MoveRejected {
        index: 0,
        source: MoveError::Unsupported,
    });
    assert!(source.is_some());
}

#[test]
fn restore_validates_the_redo_tail_too() {
    let error = Session::restore_with(record(7, 2), counter_deal).err();
    assert!(matches!(
        error,
        Some(RestoreError::MoveRejected { index: 6, .. })
    ));
}

#[test]
fn record_json_is_stable() {
    let record = SessionRecord {
        variant: Variant::Klondike(DrawCount::Three),
        seed: 7,
        moves: vec![Move::Stock],
        cursor: 1,
    };
    let json = r#"{"variant":{"klondike":"three"},"seed":7,"moves":["stock"],"cursor":1}"#;
    assert_eq!(serde_json::to_string(&record).unwrap(), json);
    assert_eq!(serde_json::from_str::<SessionRecord>(json).unwrap(), record);
}

#[test]
fn the_public_constructors_use_the_real_games() {
    let mut session = Session::new(VARIANT, 9);
    assert_eq!(session.apply(Move::Stock), Ok(()));
    let restored = Session::restore(session.record());
    assert!(restored.is_ok());
}
