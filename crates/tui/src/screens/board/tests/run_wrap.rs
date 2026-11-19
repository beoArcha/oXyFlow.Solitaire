use oxysol_core::{DrawCount, Move, PileRef, Session, SuitCount, Variant};
use ratatui::crossterm::event::KeyCode;

use super::*;
use crate::action::Action;
use crate::screens::board::cursor::max_depth;

const KLONDIKE: Variant = Variant::Klondike(DrawCount::One);
const SPIDER: Variant = Variant::Spider(SuitCount::One);
const FREECELL: Variant = Variant::FreeCell;
const SEED: u64 = 5;
const VARIANTS: [Variant; 3] = [KLONDIKE, SPIDER, FREECELL];

fn row(h: &mut Harness, prefix: &str) -> String {
    h.lines()
        .into_iter()
        .map(|line| line.trim().to_owned())
        .find(|line| line.starts_with(prefix))
        .unwrap_or_default()
}

/// The fixed deal of `variant` after its first move onto a column (or a stock turn), and a pile
/// of that position that offers a run of more than one card.
fn deal_with_a_run(variant: Variant) -> (Harness, PileRef) {
    let mut session = Session::new(variant, SEED);
    let legal = session.game().legal_moves();
    let first = legal
        .iter()
        .copied()
        .find(|mv| {
            matches!(
                mv,
                Move::Transfer {
                    to: PileRef::Tableau(_),
                    ..
                }
            )
        })
        .or_else(|| legal.contains(&Move::Stock).then_some(Move::Stock));
    let Some(mv) = first else {
        unreachable!("the fixed deal of {variant:?} has a column move or a stock turn")
    };
    assert_eq!(session.apply(mv), Ok(()));
    let pile = session
        .game()
        .piles()
        .iter()
        .find(|snapshot| max_depth(snapshot) > 1)
        .map(|snapshot| snapshot.pile);
    let Some(pile) = pile else {
        unreachable!("the position after {mv:?} in {variant:?} has a run")
    };
    (resumed(variant, SEED, &[mv]), pile)
}

#[test]
fn down_on_the_top_card_selects_the_whole_run() {
    for variant in VARIANTS {
        let (mut h, pile) = deal_with_a_run(variant);
        put_cursor(&mut h, pile);
        h.press(KeyCode::Down);
        let whole = run_len(&h, pile);
        assert!(whole > 1, "{variant:?}");
        assert_eq!(cursor(&h), (pile, whole), "{variant:?}");
    }
}

#[test]
fn minus_on_the_top_card_selects_the_whole_run() {
    for variant in VARIANTS {
        let (mut h, pile) = deal_with_a_run(variant);
        put_cursor(&mut h, pile);
        h.type_char('-');
        assert_eq!(cursor(&h), (pile, run_len(&h, pile)), "{variant:?}");
    }
}

#[test]
fn plus_at_the_deepest_card_returns_to_the_top_card() {
    for variant in VARIANTS {
        let (mut h, pile) = deal_with_a_run(variant);
        put_cursor(&mut h, pile);
        h.press(KeyCode::Down);
        h.type_char('+');
        assert_eq!(cursor(&h), (pile, 1), "{variant:?}");
    }
}

#[test]
fn up_at_the_deepest_card_still_leaves_the_pile() {
    for variant in VARIANTS {
        let (mut h, pile) = deal_with_a_run(variant);
        put_cursor(&mut h, pile);
        h.press(KeyCode::Down);
        h.press(KeyCode::Up);
        let (left, depth) = cursor(&h);
        assert_ne!(left, pile, "{variant:?}");
        assert_eq!(depth, 1, "{variant:?}");
    }
}

#[test]
fn down_steps_up_the_run_and_then_wraps_to_the_whole_run() {
    for variant in VARIANTS {
        let (mut h, pile) = deal_with_a_run(variant);
        put_cursor(&mut h, pile);
        h.press(KeyCode::Up);
        assert_eq!(cursor(&h), (pile, 2), "{variant:?}");
        h.press(KeyCode::Down);
        assert_eq!(cursor(&h), (pile, 1), "{variant:?}");
        h.press(KeyCode::Down);
        assert_eq!(cursor(&h), (pile, run_len(&h, pile)), "{variant:?}");
    }
}

#[test]
fn down_on_a_one_card_pile_still_moves_to_the_pile_below() {
    let mut h = start(KLONDIKE, SEED);
    put_cursor(&mut h, PileRef::Waste);
    assert_eq!(run_len(&h, PileRef::Waste), 1);
    h.press(KeyCode::Down);
    assert_eq!(cursor(&h), (PileRef::Tableau(1), 1));
}

#[test]
fn the_run_keys_are_usable_and_named_when_the_pile_has_a_run() {
    for variant in VARIANTS {
        let (mut h, pile) = deal_with_a_run(variant);
        put_cursor(&mut h, pile);
        let usable = board(&h).guidance().map(|g| g.usable).unwrap_or_default();
        assert!(usable.contains(&Action::RunMore), "{variant:?}");
        assert!(usable.contains(&Action::RunLess), "{variant:?}");
        assert!(
            row(&mut h, "Do:").contains("+/-: change run size"),
            "{variant:?}"
        );
        h.press(KeyCode::Down);
        let whole = run_len(&h, pile);
        let now = row(&mut h, "Now:");
        assert!(now.contains(&format!("{whole} cards from")), "{now}");
        let usable = board(&h).guidance().map(|g| g.usable).unwrap_or_default();
        assert!(usable.contains(&Action::RunLess), "{variant:?}");
    }
}

#[test]
fn the_run_keys_are_not_usable_on_a_one_card_pile() {
    let mut h = start(KLONDIKE, SEED);
    put_cursor(&mut h, PileRef::Waste);
    let usable = board(&h).guidance().map(|g| g.usable).unwrap_or_default();
    assert!(!usable.contains(&Action::RunMore));
    assert!(!usable.contains(&Action::RunLess));
    assert!(!row(&mut h, "Do:").contains("change run size"));
}
