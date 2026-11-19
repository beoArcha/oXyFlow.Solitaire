use std::time::Duration;

use oxysol_core::{DrawCount, Session, Variant};

use super::resume::label;
use crate::i18n::{Locale, set_locale};
use crate::screens::SavedSummary;
use crate::storage::memory::MemoryStorage;
use crate::storage::{GameId, SavedGame};
use crate::test_support::{Harness, TRUE_COLOR};

fn summary(variant: Variant) -> SavedSummary {
    SavedSummary {
        variant,
        elapsed: Duration::from_secs(754),
        moves: 42,
    }
}

#[test]
fn resume_names_the_game_its_time_and_its_moves_at_100_columns() {
    set_locale(Locale::EnUs);
    let klondike = summary(Variant::Klondike(DrawCount::Three));
    assert_eq!(
        label(klondike, 100),
        "Resume: Klondike, Draw 3, 12:34, Moves 42"
    );
    set_locale(Locale::PlPl);
    assert_eq!(
        label(klondike, 100),
        "Wznów: Klondike, 3 karty, 12:34, Ruchy 42"
    );
    set_locale(Locale::EnUs);
}

#[test]
fn a_narrow_row_drops_the_moves_then_the_time() {
    set_locale(Locale::EnUs);
    let klondike = summary(Variant::Klondike(DrawCount::Three));
    assert_eq!(label(klondike, 40), "Resume: Klondike, Draw 3, 12:34");
    assert_eq!(label(klondike, 28), "Resume: Klondike, Draw 3");
    assert_eq!(label(klondike, 4), "Resume: Klondike, Draw 3");
}

#[test]
fn the_saved_game_is_named_in_the_menu() {
    set_locale(Locale::EnUs);
    let storage = MemoryStorage {
        game: Some(SavedGame {
            id: GameId(3),
            record: Session::new(Variant::FreeCell, 5).record(),
            elapsed_ms: 61_000,
            undo_allowed: true,
        }),
        ..MemoryStorage::new()
    };
    let mut h = Harness::with(storage, TRUE_COLOR, 100, 30);
    assert!(h.text().contains("Resume: FreeCell, 1:01, Moves 0"));
}
