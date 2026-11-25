//! Quick move from the exposed waste pile (`e`, O-110): moves the exposed waste card from
//! anywhere on the board without moving the cursor, uses the same destination order as `a` on the
//! waste, and refuses cleanly when no waste card can move.

use oxysol_core::{DrawCount, Move, PileRef, PyramidRules, SuitCount, Variant, YukonRules};

use super::*;
use crate::audio::SoundCue;
use crate::screens::board::choices::{self, quick};
use crate::screens::board::status;

pub(super) const KLONDIKE_ONE: Variant = Variant::Klondike(DrawCount::One);
const KLONDIKE_THREE: Variant = Variant::Klondike(DrawCount::Three);
const AUSTRALIAN: Variant = Variant::Yukon(YukonRules::Australian);

pub(super) fn find_playable_waste(variant: Variant) -> (Harness, u64) {
    for seed in 1..40_u64 {
        let mut h = start(variant, seed);
        for _ in 0..24 {
            if legal(&h)
                .iter()
                .any(|mv| mv.source() == Some(PileRef::Waste))
            {
                return (h, seed);
            }
            h.type_char('s');
        }
    }
    unreachable!("{variant:?} has a deal with a playable waste card")
}

fn find_waste_both_targets(variant: Variant) -> Harness {
    for seed in 1..100_u64 {
        let mut h = start(variant, seed);
        for _ in 0..24 {
            let moves = legal(&h);
            let to_found = moves.iter().any(|mv| {
                matches!(
                    mv,
                    Move::Transfer {
                        from: PileRef::Waste,
                        to: PileRef::Foundation(_),
                        count: 1
                    }
                )
            });
            let to_tab = moves.iter().any(|mv| {
                matches!(
                    mv,
                    Move::Transfer {
                        from: PileRef::Waste,
                        to: PileRef::Tableau(_),
                        count: 1
                    }
                )
            });
            if to_found && to_tab {
                return h;
            }
            h.type_char('s');
        }
    }
    unreachable!("{variant:?} has a state where waste can move to foundation and tableau")
}

fn find_unplayable_waste(variant: Variant) -> Harness {
    for seed in 1..40_u64 {
        let mut h = start(variant, seed);
        for _ in 0..24 {
            let has_card = pile_len(&h, PileRef::Waste) > 0;
            let movable = legal(&h)
                .iter()
                .any(|mv| mv.source() == Some(PileRef::Waste));
            if has_card && !movable {
                return h;
            }
            h.type_char('s');
        }
    }
    unreachable!("{variant:?} has a deal with an unplayable waste card")
}

#[test]
fn e_moves_the_exposed_waste_card_from_a_tableau_cursor_without_moving_it() {
    for variant in [KLONDIKE_ONE, KLONDIKE_THREE, AUSTRALIAN] {
        let (mut h, _) = find_playable_waste(variant);
        let waste_before = pile_len(&h, PileRef::Waste);
        let tab = PileRef::Tableau(3);
        put_cursor(&mut h, tab);
        let before_moves = moves_made(&h);

        h.type_char('e');

        assert_eq!(moves_made(&h), before_moves + 1, "{variant:?}");
        assert_eq!(
            pile_len(&h, PileRef::Waste),
            waste_before - 1,
            "{variant:?}"
        );
        assert_eq!(cursor(&h).0, tab, "{variant:?} cursor must stay put");
        assert_eq!(board(&h).assist().quick, 1, "{variant:?}");
    }
}

#[test]
fn e_also_removes_a_playable_waste_card_in_pyramid_without_moving_an_unpaired_cursor() {
    for rules in [PyramidRules::Relaxed, PyramidRules::Classic] {
        let variant = Variant::Pyramid(rules);
        let (mut h, _) = find_playable_waste(variant);
        let waste_before = pile_len(&h, PileRef::Waste);
        let away = (0..7)
            .map(|index| PileRef::Pyramid { row: 6, index })
            .find(|pile| {
                !legal(&h).iter().any(|mv| match mv {
                    Move::Remove {
                        first: PileRef::Waste,
                        second: Some(other),
                    }
                    | Move::Remove {
                        first: other,
                        second: Some(PileRef::Waste),
                    } => other == pile,
                    _ => false,
                })
            });
        let Some(away) = away else {
            unreachable!("pyramid has an unpaired bottom card")
        };
        put_cursor(&mut h, away);
        let before_moves = moves_made(&h);

        h.type_char('e');

        assert_eq!(moves_made(&h), before_moves + 1, "{variant:?}");
        assert_eq!(
            pile_len(&h, PileRef::Waste),
            waste_before - 1,
            "{variant:?}"
        );
        assert_eq!(cursor(&h).0, away, "{variant:?}");
    }
}

#[test]
fn e_picks_the_same_destination_as_a_on_the_waste() {
    for variant in [KLONDIKE_ONE, KLONDIKE_THREE, AUSTRALIAN] {
        let mut h_e = find_waste_both_targets(variant);
        let mut h_a = find_waste_both_targets(variant);

        put_cursor(&mut h_e, PileRef::Tableau(0));
        h_e.type_char('e');

        put_cursor(&mut h_a, PileRef::Waste);
        h_a.type_char('a');

        assert_eq!(
            session(&h_e).record().moves,
            session(&h_a).record().moves,
            "{variant:?}"
        );
        let last = session(&h_e).record().moves.last().copied();
        assert!(
            matches!(
                last,
                Some(Move::Transfer {
                    from: PileRef::Waste,
                    to: PileRef::Foundation(_),
                    count: 1
                })
            ),
            "{variant:?}: foundation preferred over tableau, got {last:?}"
        );
    }
}

#[test]
fn e_matches_choices_quick_for_waste() {
    let (h, _) = find_playable_waste(KLONDIKE_ONE);
    let legal_moves = legal(&h);
    let expected = choices::waste_quick(&legal_moves);
    let from_choices = quick(&choices::choices(&legal_moves, PileRef::Waste, 1));
    assert_eq!(expected, from_choices);
}

#[test]
fn e_does_nothing_with_no_waste_card_or_no_legal_destination() {
    for variant in [KLONDIKE_ONE, KLONDIKE_THREE, AUSTRALIAN] {
        let mut h = start(variant, 5);
        assert_eq!(pile_len(&h, PileRef::Waste), 0);
        put_cursor(&mut h, PileRef::Tableau(0));
        h.type_char('e');
        assert_eq!(moves_made(&h), 0, "{variant:?} empty waste");
        assert_eq!(status(&h), Some(status::NO_TARGETS), "{variant:?}");
        assert_eq!(cues(&h).last(), Some(&SoundCue::Illegal), "{variant:?}");
        assert_eq!(cursor(&h).0, PileRef::Tableau(0), "{variant:?}");

        let mut blocked = find_unplayable_waste(variant);
        let before = (moves_made(&blocked), pile_len(&blocked, PileRef::Waste));
        put_cursor(&mut blocked, PileRef::Tableau(0));
        blocked.type_char('e');
        assert_eq!(
            (moves_made(&blocked), pile_len(&blocked, PileRef::Waste)),
            before,
            "{variant:?} unplayable waste"
        );
        assert_eq!(status(&blocked), Some(status::NO_TARGETS), "{variant:?}");
        assert_eq!(
            cues(&blocked).last(),
            Some(&SoundCue::Illegal),
            "{variant:?}"
        );
    }

    for no_waste in [
        Variant::FreeCell,
        Variant::Spider(SuitCount::One),
        Variant::Yukon(YukonRules::Classic),
    ] {
        let mut h = start(no_waste, 5);
        h.type_char('e');
        assert_eq!(moves_made(&h), 0, "{no_waste:?}");
        assert_eq!(status(&h), Some(status::NO_TARGETS), "{no_waste:?}");
        assert_eq!(cues(&h).last(), Some(&SoundCue::Illegal), "{no_waste:?}");
    }
}
