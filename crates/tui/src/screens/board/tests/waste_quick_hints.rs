//! Hints, guide, help and quick-move setting coverage for `e` (`Action::WasteQuickMove`, O-110).

use oxysol_core::{PileRef, Variant};

use super::waste_quick::{KLONDIKE_ONE, find_playable_waste};
use super::*;
use crate::action::Action;
use crate::app::Modal;
use crate::audio::SoundCue;
use crate::i18n::{Locale, set_locale, t, tf};
use crate::input::primary_key;
use crate::screens::board::status;
use crate::settings::{Aids, Settings};

fn game_in(locale: Locale, variant: Variant, seed: u64, size: (u16, u16)) -> Harness {
    let storage = MemoryStorage {
        settings: Some(Settings {
            locale,
            moves_panel: false,
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
    set_locale(locale);
    h.lines();
    h
}

fn set_quick_move(h: &mut Harness, quick_move: bool) {
    h.app.apply(Effect::Quick(Aids {
        quick_move,
        ..Settings::default().aids()
    }));
}

fn waste_do_texts() -> [String; 2] {
    let key = primary_key(Action::WasteQuickMove).unwrap_or_default();
    [
        tf("board.action.waste_quick", &[("key", &key)]),
        tf("board.action.waste_quick_foundation", &[("key", &key)]),
    ]
}

#[test]
fn golf_has_no_waste_quick_move_because_cards_never_leave_the_waste() {
    let mut h = game_in(Locale::EnUs, Variant::Golf, 5, (140, 40));
    let before = (moves_made(&h), pile_len(&h, PileRef::Waste));
    assert!(pile_len(&h, PileRef::Waste) > 0);
    h.type_char('e');
    assert_eq!((moves_made(&h), pile_len(&h, PileRef::Waste)), before);
    assert_eq!(status(&h), Some(status::NO_TARGETS));
    assert_eq!(cues(&h).last(), Some(&SoundCue::Illegal));

    let text = h.text();
    let guide_label = t("board.guide.quick_waste").into_owned();
    assert!(!text.contains(&guide_label), "{text}");
    for do_text in waste_do_texts() {
        assert!(!text.contains(&do_text), "{text}");
    }
}

#[test]
fn e_is_refused_and_its_hints_are_hidden_when_quick_move_is_off() {
    let (mut h, seed) = find_playable_waste(KLONDIKE_ONE);
    h.resize(200, 60);
    set_quick_move(&mut h, true);
    put_cursor(&mut h, PileRef::Tableau(0));
    let text_on = h.text();
    let do_texts = waste_do_texts();
    let guide_label = t("board.guide.quick_waste").into_owned();
    assert!(do_texts.iter().any(|s| text_on.contains(s)), "{text_on}");
    assert!(text_on.contains(&guide_label), "{text_on}");

    set_quick_move(&mut h, false);
    let text_off = h.text();
    assert!(!do_texts.iter().any(|s| text_off.contains(s)), "{text_off}");
    assert!(!text_off.contains(&guide_label), "{text_off}");

    let before = moves_made(&h);
    h.type_char('e');
    assert_eq!(moves_made(&h), before, "seed {seed}");
    assert_eq!(status(&h), Some(status::QUICK_OFF));
    assert_eq!(cues(&h).last(), Some(&SoundCue::Illegal));
    assert!(h.text().contains(&t("board.status.quick_off").into_owned()));

    set_quick_move(&mut h, true);
    h.type_char('e');
    assert_eq!(moves_made(&h), before + 1);
}

#[test]
fn do_row_guide_and_help_show_waste_quick_move_in_every_offered_locale() {
    let (_, seed) = find_playable_waste(KLONDIKE_ONE);
    for locale in Locale::offered() {
        let mut h = game_in(locale, KLONDIKE_ONE, seed, (200, 60));
        while !legal(&h)
            .iter()
            .any(|mv| mv.source() == Some(PileRef::Waste))
        {
            h.type_char('s');
        }
        put_cursor(&mut h, PileRef::Tableau(0));
        set_locale(locale);
        let key = primary_key(Action::WasteQuickMove).unwrap_or_default();
        let quick_key = primary_key(Action::QuickMove).unwrap_or_default();
        let do_variants = waste_do_texts();
        let guide_label = t("board.guide.quick_waste").into_owned();
        let help_label = t("help.action.waste_quick_move").into_owned();

        let lines = h.lines();
        let do_row = lines.last().cloned().unwrap_or_default();
        assert!(
            do_variants.iter().any(|part| do_row.contains(part)),
            "{locale:?}: missing waste quick move in Do row: {do_row}"
        );
        let panel_text = lines.join("\n");
        let guide_keys = format!("• {quick_key}/{key}");
        assert!(
            panel_text.contains(&guide_keys) && panel_text.contains(&guide_label),
            "{locale:?}: missing guide entry `{guide_keys} {guide_label}`"
        );

        h.type_char('?');
        assert!(matches!(h.app.modal(), Some(Modal::Help(_))), "{locale:?}");
        let help_text = h.text();
        assert!(
            help_text.contains(&help_label) && help_text.contains(&format!("{key}  ")),
            "{locale:?}: missing help row `{key}  {help_label}` in:\n{help_text}"
        );

        let mut compact = game_in(locale, KLONDIKE_ONE, seed, (100, 30));
        while !legal(&compact)
            .iter()
            .any(|mv| mv.source() == Some(PileRef::Waste))
        {
            compact.type_char('s');
        }
        for line in compact.lines() {
            assert!(line.chars().count() <= 100, "{locale:?}: {line}");
        }
    }
    set_locale(Locale::EnUs);
}

#[test]
fn do_row_hides_waste_quick_move_when_waste_cannot_move_in_every_offered_locale() {
    for locale in Locale::offered() {
        let mut h = game_in(locale, KLONDIKE_ONE, 5, (200, 60));
        put_cursor(&mut h, PileRef::Tableau(0));
        set_locale(locale);
        let do_variants = waste_do_texts();
        let lines = h.lines();
        let do_row = lines.last().cloned().unwrap_or_default();
        assert!(
            !do_variants.iter().any(|part| do_row.contains(part)),
            "{locale:?}: Do row must not show waste quick move on empty waste: {do_row}"
        );
    }
    set_locale(Locale::EnUs);
}
