//! The quick options dialog (`o`): the aids of the running game from the board, applied at once,
//! kept apart from the defaults of the settings until "Save as default".

use oxysol_core::{DrawCount, Variant};
use ratatui::crossterm::event::KeyCode;

use super::aura::painted;
use super::finish::{KLONDIKE_SEED, at_forced_win, is_won};
use super::*;
use crate::app::Modal;
use crate::i18n::{Locale, set_locale, t};
use crate::settings::{Aid, Aids, AuraMode, Settings};
use crate::test_support::PLAIN;
use crate::theme::Token;

const KLONDIKE: Variant = Variant::Klondike(DrawCount::One);
const WIDE: (u16, u16) = (140, 40);
const SAVE: usize = Aid::ALL.len();
const CLOSE: usize = SAVE + 1;

fn game_with(settings: Settings, caps: crate::caps::TerminalCaps, size: (u16, u16)) -> Harness {
    let storage = MemoryStorage {
        settings: Some(settings),
        ..MemoryStorage::default()
    };
    let mut h = Harness::with(storage, caps, size.0, size.1);
    h.app.apply(Effect::Start {
        variant: KLONDIKE,
        seed: Some(5),
        undo_allowed: true,
    });
    h.lines();
    h
}

fn game() -> Harness {
    game_with(Settings::default(), TRUE_COLOR, WIDE)
}

fn defaults() -> Aids {
    Settings::default().aids()
}

fn dialog_focus(h: &Harness) -> Option<usize> {
    match h.app.modal() {
        Some(Modal::Quick(state)) => Some(state.focus),
        _ => None,
    }
}

fn is_open(h: &Harness) -> bool {
    matches!(h.app.modal(), Some(Modal::Quick(_)))
}

fn open(h: &mut Harness) {
    h.type_char('o');
    assert!(is_open(h), "{}", h.text());
}

fn focus(h: &mut Harness, index: usize) {
    for _ in 0..index {
        h.press(KeyCode::Down);
    }
    assert_eq!(dialog_focus(h), Some(index));
}

fn click_text(h: &mut Harness, text: &str) {
    let Some((column, row)) = h.find(text) else {
        panic!("{text:?} is not drawn:\n{}", h.text());
    };
    h.click(column + 1, row);
}

fn click_row(h: &mut Harness, aid: Aid) {
    let label = t(aid.label_key()).into_owned();
    click_text(h, &format!("{label} ."));
}

#[test]
fn o_opens_the_dialog_with_the_running_choices_and_esc_closes_it_keeping_them() {
    let mut h = game();
    open(&mut h);
    let text = h.text();
    assert!(text.contains("Quick options"), "{text}");
    for aid in Aid::ALL {
        assert!(text.contains(&t(aid.label_key()).into_owned()), "{aid:?}");
    }
    assert!(text.contains("Save as default"));
    assert!(text.contains("Close [Esc]"));
    focus(&mut h, 0);
    h.press(KeyCode::Enter);
    h.press(KeyCode::Esc);
    assert!(!is_open(&h));
    assert_eq!(h.app.aids().aura, AuraMode::FocusOnly);
}

#[test]
fn o_again_closes_the_dialog_like_the_guide_and_moves_keys() {
    let mut h = game();
    open(&mut h);
    h.type_char('o');
    assert!(!is_open(&h));
}

#[test]
fn the_footer_button_opens_the_dialog() {
    let mut h = game_with(Settings::default(), TRUE_COLOR, (100, 30));
    assert!(h.text().contains("o: options"));
    click_text(&mut h, "o: options");
    assert!(is_open(&h));
}

#[test]
fn every_aid_switches_from_the_dialog_by_keyboard_for_the_running_game_only() {
    for (index, aid) in Aid::ALL.into_iter().enumerate() {
        let mut h = game();
        open(&mut h);
        focus(&mut h, index);
        h.press(KeyCode::Enter);
        assert!(is_open(&h), "{aid:?}");
        assert_eq!(h.app.aids(), aid.toggled(&defaults()), "{aid:?}");
        assert_eq!(h.app.settings().aids(), defaults(), "{aid:?}");
        h.press(KeyCode::Right);
        assert_eq!(h.app.aids(), defaults(), "{aid:?} switches back");
    }
}

#[test]
fn every_aid_switches_by_a_click_on_its_row() {
    for aid in Aid::ALL {
        let mut h = game();
        open(&mut h);
        click_row(&mut h, aid);
        assert_eq!(h.app.aids(), aid.toggled(&defaults()), "{aid:?}");
        assert_eq!(h.app.settings().aids(), defaults(), "{aid:?}");
        assert!(is_open(&h));
    }
}

#[test]
fn tab_walks_the_rows_then_both_buttons_and_wraps_both_ways() {
    let mut h = game();
    open(&mut h);
    for expected in 1..=CLOSE {
        h.press(KeyCode::Tab);
        assert_eq!(dialog_focus(&h), Some(expected));
    }
    h.press(KeyCode::Tab);
    assert_eq!(dialog_focus(&h), Some(0));
    h.press(KeyCode::BackTab);
    assert_eq!(dialog_focus(&h), Some(CLOSE));
    h.press(KeyCode::BackTab);
    assert_eq!(dialog_focus(&h), Some(SAVE));
}

#[test]
fn left_and_right_move_between_the_two_buttons() {
    let mut h = game();
    open(&mut h);
    focus(&mut h, SAVE);
    h.press(KeyCode::Right);
    assert_eq!(dialog_focus(&h), Some(CLOSE));
    h.press(KeyCode::Left);
    assert_eq!(dialog_focus(&h), Some(SAVE));
    assert_eq!(h.app.aids(), defaults());
}

#[test]
fn the_close_button_and_the_back_key_close_the_dialog() {
    let mut h = game();
    open(&mut h);
    focus(&mut h, CLOSE);
    h.press(KeyCode::Enter);
    assert!(!is_open(&h));
    open(&mut h);
    click_text(&mut h, "Close [Esc]");
    assert!(!is_open(&h));
}

#[test]
fn the_focused_row_is_marked_without_color_and_explained() {
    let mut h = game();
    open(&mut h);
    let lines = h.lines();
    let aura = lines.iter().find(|line| line.contains("Aura ."));
    assert!(aura.is_some_and(|line| line.contains('►')), "{lines:?}");
    assert!(h.text().contains(&t(Aid::Aura.description_key())[..20]));
    h.press(KeyCode::Down);
    let lines = h.lines();
    let marked: Vec<&String> = lines.iter().filter(|line| line.contains("► ")).collect();
    assert!(
        marked.iter().any(|line| line.contains("Target marks")),
        "{marked:?}"
    );
}

#[test]
fn the_guide_and_the_moves_panel_follow_the_dialog_at_once() {
    let mut h = game();
    assert!(h.text().contains("Guide (g: hide)"));
    assert!(h.text().contains("Possible moves (v: hide)"));
    open(&mut h);
    focus(&mut h, 2);
    h.press(KeyCode::Enter);
    assert!(!h.text().contains("Guide (g: hide)"), "{}", h.text());
    assert!(h.text().contains("Possible moves (v: hide)"));
    h.press(KeyCode::Down);
    h.press(KeyCode::Enter);
    assert!(!h.text().contains("Possible moves (v: hide)"));
    h.press(KeyCode::Esc);
    assert!(!h.app.aids().guide_panel && !h.app.aids().moves_panel);
}

#[test]
fn target_marks_off_leave_a_carried_run_without_target_marks_at_once() {
    let mut h = game();
    let Some((from, _, _)) = first_transfer(&h) else {
        unreachable!("seed 5 has a transfer")
    };
    put_cursor(&mut h, from);
    h.press(KeyCode::Enter);
    assert!(!board(&h).marks().targets.is_empty());
    assert!(!painted(&mut h, "·", Token::Target).is_empty());
    open(&mut h);
    focus(&mut h, 1);
    h.press(KeyCode::Enter);
    h.lines();
    assert!(board(&h).marks().targets.is_empty());
    assert!(painted(&mut h, "·", Token::Target).is_empty());
    assert!(board(&h).marks().selected.is_some());
    h.press(KeyCode::Enter);
    h.lines();
    assert!(!board(&h).marks().targets.is_empty());
}

#[test]
fn the_focus_only_aura_drops_the_carry_halo_at_once() {
    let mut h = game();
    let Some((from, _, _)) = first_transfer(&h) else {
        unreachable!("seed 5 has a transfer")
    };
    put_cursor(&mut h, from);
    h.press(KeyCode::Enter);
    assert!(!painted(&mut h, "▓", Token::AuraCarry).is_empty());
    open(&mut h);
    h.press(KeyCode::Enter);
    assert!(painted(&mut h, "▓", Token::AuraCarry).is_empty());
    assert!(!painted(&mut h, "░", Token::AuraFocus).is_empty());
}

#[test]
fn auto_finish_off_in_the_dialog_leaves_the_forced_win_to_the_player() {
    let (mut h, base, _) = at_forced_win(KLONDIKE, KLONDIKE_SEED, true);
    open(&mut h);
    focus(&mut h, 4);
    h.press(KeyCode::Enter);
    assert!(!h.app.aids().auto_finish);
    assert!(h.app.settings().auto_finish);
    h.press(KeyCode::Esc);
    h.advance(10);
    assert_eq!(moves_made(&h), base);
    assert!(!is_won(&h));
}

#[test]
fn auto_finish_on_in_the_dialog_plays_the_win_out_although_the_default_is_off() {
    let (mut h, base, _) = at_forced_win(KLONDIKE, KLONDIKE_SEED, false);
    assert!(!h.app.aids().auto_finish);
    open(&mut h);
    focus(&mut h, 4);
    h.press(KeyCode::Enter);
    assert!(h.app.aids().auto_finish);
    assert!(!h.app.settings().auto_finish);
    h.press(KeyCode::Esc);
    for _ in 0..40 {
        h.advance(1);
    }
    assert!(moves_made(&h) > base || is_won(&h));
}

#[test]
fn save_as_default_keeps_the_choices_in_the_settings_and_in_storage() {
    let mut h = game();
    open(&mut h);
    h.press(KeyCode::Enter);
    h.press(KeyCode::Down);
    h.press(KeyCode::Down);
    h.press(KeyCode::Enter);
    let chosen = h.app.aids();
    assert_ne!(chosen, defaults());
    assert_eq!(h.app.settings().aids(), defaults());
    focus_save(&mut h);
    h.press(KeyCode::Enter);
    assert!(!is_open(&h));
    assert_eq!(h.app.settings().aids(), chosen);
    assert_eq!(h.app.aids(), chosen);
    let saved = h.storage.borrow().settings.clone();
    assert!(saved.is_some_and(|settings| settings.aids() == chosen));
    assert_eq!(status(&h), Some("board.status.defaults_saved"));
    assert!(h.text().contains("Saved as default for new games."));
}

fn focus_save(h: &mut Harness) {
    while dialog_focus(h) != Some(SAVE) {
        h.press(KeyCode::Tab);
    }
}

#[test]
fn save_as_default_works_by_a_click() {
    let mut h = game();
    open(&mut h);
    click_row(&mut h, Aid::TargetMarks);
    click_text(&mut h, "Save as default");
    assert!(!is_open(&h));
    assert!(!h.app.settings().target_marks);
    let saved = h.storage.borrow().settings.clone();
    assert!(saved.is_some_and(|settings| !settings.target_marks));
}

#[test]
fn the_defaults_apply_to_a_new_game() {
    let mut h = game();
    h.app.apply(Effect::Apply(Settings {
        guide_panel: false,
        aura: AuraMode::FocusOnly,
        ..Settings::default()
    }));
    h.app.apply(Effect::Start {
        variant: KLONDIKE,
        seed: Some(6),
        undo_allowed: true,
    });
    assert!(!h.app.aids().guide_panel);
    assert_eq!(h.app.aids().aura, AuraMode::FocusOnly);
    assert!(!h.text().contains("Guide (g: hide)"));
}

#[test]
fn changing_a_default_does_not_change_the_running_game() {
    let mut h = game();
    open(&mut h);
    h.press(KeyCode::Enter);
    h.press(KeyCode::Esc);
    let running = h.app.aids();
    h.app.apply(Effect::Apply(Settings {
        guide_panel: false,
        auto_finish: false,
        target_marks: false,
        ..Settings::default()
    }));
    assert_eq!(h.app.aids(), running);
    assert!(h.text().contains("Guide (g: hide)"));
}

#[test]
fn a_restart_keeps_the_quick_choices_and_a_new_game_takes_the_defaults_again() {
    let mut h = game();
    h.type_char('g');
    assert!(!h.app.aids().guide_panel);
    h.app.apply(Effect::Restart);
    assert!(!h.app.aids().guide_panel);
    h.app.apply(Effect::Start {
        variant: KLONDIKE,
        seed: Some(7),
        undo_allowed: true,
    });
    assert!(h.app.aids().guide_panel);
}

#[test]
fn the_board_toggles_g_and_v_are_quick_choices_and_save_nothing() {
    let mut h = game();
    h.type_char('g');
    h.type_char('v');
    assert!(!h.app.aids().guide_panel && !h.app.aids().moves_panel);
    assert_eq!(h.app.settings().aids(), defaults());
    assert_eq!(
        h.storage.borrow().settings.as_ref().map(Settings::aids),
        Some(defaults())
    );
    open(&mut h);
    focus_save(&mut h);
    h.press(KeyCode::Enter);
    let saved = h.storage.borrow().settings.clone();
    assert!(saved.is_some_and(|s| !s.guide_panel && !s.moves_panel));
}

#[test]
fn the_dialog_renders_in_both_locales_and_glyph_sets_at_100_by_30() {
    for (locale, title, save) in [
        (Locale::EnUs, "Quick options", "Save as default"),
        (Locale::PlPl, "Szybkie opcje", "Zapisz jako domyślne"),
    ] {
        for caps in [TRUE_COLOR, PLAIN] {
            let settings = Settings {
                locale,
                ..Settings::for_caps(&caps)
            };
            let mut h = game_with(settings, caps, (100, 30));
            open(&mut h);
            let text = h.text();
            assert!(text.contains(title), "{locale:?}: {text}");
            assert!(text.contains(save), "{locale:?}");
            for aid in Aid::ALL {
                assert!(text.contains(&t(aid.label_key()).into_owned()), "{aid:?}");
            }
            if !caps.unicode && locale == Locale::EnUs {
                assert!(text.is_ascii(), "{locale:?} ASCII");
            }
        }
    }
    set_locale(Locale::EnUs);
}

#[test]
fn a_panel_row_names_the_columns_it_needs_on_a_narrow_screen() {
    let mut h = game_with(Settings::default(), TRUE_COLOR, (100, 30));
    open(&mut h);
    let text = h.text();
    assert!(text.contains("On (needs 130 columns, now 100)"), "{text}");
}
