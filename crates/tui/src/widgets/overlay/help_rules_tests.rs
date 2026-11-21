//! Integration tests for the help window tabs (Keys and Rules), switching,
//! scrolling, opening from board and menu, and clean rendering.

use oxysol_core::{DrawCount, PyramidRules, SuitCount, Variant};
use ratatui::crossterm::event::{KeyCode, KeyModifiers, MouseEvent, MouseEventKind};
use ratatui::style::Modifier;

use super::{Focus, HowToPlay, Rules};
use crate::app::Modal;
use crate::caps::TerminalCaps;
use crate::glyphs::is_wgl4;
use crate::i18n::{self, Locale};
use crate::input::Input;
use crate::screens::Effect;
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, PLAIN, TRUE_COLOR};

const WIDTH: u16 = 100;
const HEIGHT: u16 = 30;
const FORBIDDEN_UNICODE: [char; 6] = ['▶', '◀', '╭', '╮', '╰', '╯'];

fn harness(caps: TerminalCaps) -> Harness {
    Harness::with(MemoryStorage::new(), caps, WIDTH, HEIGHT)
}

fn playing(variant: Variant) -> Harness {
    let mut h = harness(TRUE_COLOR);
    h.app.apply(Effect::Start {
        variant,
        seed: Some(42),
        undo_allowed: true,
    });
    h
}

fn shown(h: &Harness) -> Option<(Rules, Focus)> {
    match h.app.modal() {
        Some(Modal::HowToPlay(state)) => Some((state.rules, state.focus)),
        _ => None,
    }
}

fn shown_rules(h: &Harness) -> Option<HowToPlay> {
    match h.app.modal() {
        Some(Modal::HowToPlay(state)) => Some(*state),
        _ => None,
    }
}

fn click_text(h: &mut Harness, needle: &str) {
    let (col, row) = h
        .find(needle)
        .unwrap_or_else(|| panic!("{needle} not found"));
    h.click(col, row);
}

fn titled(h: &mut Harness, title: &str) -> bool {
    h.text().contains(&format!(" {title} "))
}

#[test]
fn the_rules_opened_as_how_to_play_are_titled_how_to_play() {
    let mut h = harness(TRUE_COLOR);
    h.app.apply(Effect::Open(Modal::HowToPlay(HowToPlay::new(
        Rules::Klondike,
    ))));
    let title = i18n::t("help.rules.title").into_owned();
    assert!(titled(&mut h, &title), "{title}");
}

#[test]
fn the_rules_reached_from_help_are_titled_help_rules() {
    let mut h = harness(TRUE_COLOR);
    h.type_char('?');
    h.press(KeyCode::Tab);
    assert!(matches!(shown(&h), Some((Rules::Klondike, _))));
    let title = i18n::t("help.rules.title_from_help").into_owned();
    assert!(titled(&mut h, &title), "{title}");
}

#[test]
fn the_title_from_help_stays_when_the_game_changes() {
    let mut h = harness(TRUE_COLOR);
    h.type_char('?');
    h.press(KeyCode::Tab);
    h.press(KeyCode::Down);
    h.press(KeyCode::Right);
    assert!(matches!(shown(&h), Some((Rules::Spider, _))));
    let title = i18n::t("help.rules.title_from_help").into_owned();
    assert!(titled(&mut h, &title), "{title}");
}

#[test]
fn help_opens_on_keys_tab_with_uncolored_selection() {
    let mut h = harness(TRUE_COLOR);
    h.type_char('?');
    assert_eq!(h.app.modal(), Some(&Modal::help()));
    let text = h.text();
    assert!(text.contains("[Keys]"));
    assert!(text.contains("Rules"));
    assert!(text.contains("Navigation"));
    assert!(text.contains("Quit"));

    let (col, row) = h.find("[Keys]").expect("tab");
    let cell = &h.terminal.backend().buffer()[(col, row)];
    assert!(cell.modifier.contains(Modifier::REVERSED));
    assert!(cell.modifier.contains(Modifier::BOLD));
}

#[test]
fn switching_between_keys_and_rules_by_key_and_mouse() {
    let mut h = harness(TRUE_COLOR);
    h.type_char('?');
    assert_eq!(h.app.modal(), Some(&Modal::help()));

    h.press(KeyCode::Tab);
    assert_eq!(shown(&h), Some((Rules::Klondike, Focus::Tabs)));
    assert!(h.text().contains("Keys"));
    assert!(h.text().contains("[Rules]"));
    assert!(h.text().contains("[Klondike]"));

    for _ in 0..3 {
        h.press(KeyCode::Tab);
    }
    assert!(matches!(h.app.modal(), Some(Modal::CommandLine(_))));
    h.press(KeyCode::Tab);
    assert_eq!(h.app.modal(), Some(&Modal::help()));
    assert!(h.text().contains("[Keys]"));

    click_text(&mut h, "Rules");
    assert_eq!(shown(&h), Some((Rules::Klondike, Focus::Games)));
    assert!(h.text().contains("[Rules]"));

    click_text(&mut h, "Keys");
    assert_eq!(h.app.modal(), Some(&Modal::help()));
    assert!(h.text().contains("[Keys]"));

    h.press(KeyCode::Right);
    assert_eq!(shown(&h), Some((Rules::Klondike, Focus::Tabs)));
    assert!(h.text().contains("[Rules]"));
}

#[test]
fn rules_opens_on_current_game_from_board() {
    let cases = [
        (Variant::Klondike(DrawCount::Three), Rules::Klondike),
        (Variant::Spider(SuitCount::Two), Rules::Spider),
        (Variant::FreeCell, Rules::FreeCell),
        (Variant::Pyramid(PyramidRules::Classic), Rules::Pyramid),
    ];
    for (variant, rules) in cases {
        let mut h = playing(variant);
        h.type_char('?');
        assert_eq!(h.app.modal(), Some(&Modal::help()));
        h.press(KeyCode::Tab);
        assert_eq!(shown(&h).map(|state| state.0), Some(rules), "{variant:?}");
        let name = i18n::t(rules.name_key());
        assert!(h.text().contains(&format!("[{name}]")));

        h.press(KeyCode::Esc);
        assert!(h.app.modal().is_none());

        h.type_char('i');
        assert_eq!(
            shown_rules(&h),
            Some(HowToPlay::new(rules)),
            "via i {variant:?}"
        );
    }
}

#[test]
fn scrolling_works_in_rules_tab() {
    let mut h = harness(TRUE_COLOR);
    h.type_char('?');
    h.press(KeyCode::Tab);
    h.press(KeyCode::Tab);
    h.press(KeyCode::Tab);
    assert_eq!(shown(&h).map(|state| state.1), Some(Focus::Text));
    assert!(h.text().contains("1-20 / "));

    h.press(KeyCode::Down);
    assert!(h.text().contains("2-21 / "));

    h.press(KeyCode::PageDown);
    let state = shown_rules(&h).expect("open rules");
    assert!(state.scroll >= 19);

    h.press(KeyCode::Home);
    assert!(h.text().contains("1-20 / "));

    h.send(Input::Mouse(MouseEvent {
        kind: MouseEventKind::ScrollDown,
        column: 50,
        row: 15,
        modifiers: KeyModifiers::NONE,
    }));
    assert!(h.text().contains("4-23 / "));
}

#[test]
fn i_and_menu_item_show_rules_and_can_switch_to_keys() {
    let mut h = harness(TRUE_COLOR);
    h.type_char('i');
    assert_eq!(shown_rules(&h), Some(HowToPlay::new(Rules::Klondike)));
    assert!(h.text().contains("[Rules]"));
    h.press(KeyCode::BackTab);
    h.press(KeyCode::BackTab);
    assert_eq!(h.app.modal(), Some(&Modal::help()));
    assert!(h.text().contains("[Keys]"));
    h.press(KeyCode::Esc);
    assert!(h.app.modal().is_none());

    click_text(&mut h, "How to play");
    assert_eq!(shown(&h), Some((Rules::Klondike, Focus::Games)));
    assert!(h.text().contains("[Rules]"));
    h.press(KeyCode::BackTab);
    h.press(KeyCode::BackTab);
    assert_eq!(h.app.modal(), Some(&Modal::help()));
    assert!(h.text().contains("[Keys]"));
}

#[test]
fn renders_cleanly_at_100x30_in_both_locales_and_glyph_modes() {
    for locale in Locale::offered() {
        i18n::set_locale(locale);
        for caps in [TRUE_COLOR, PLAIN] {
            let mut h = harness(caps);
            h.type_char('?');
            let text = h.text();
            let ctx = format!("{locale:?} unicode={} keys", caps.unicode);
            for ch in text.chars() {
                assert!(ch.is_ascii() || is_wgl4(ch), "{ctx}: {ch:?}");
            }
            if caps.unicode {
                for glyph in FORBIDDEN_UNICODE {
                    assert!(!text.contains(glyph), "{ctx}: {glyph}");
                }
            }

            h.press(KeyCode::Tab);
            let rules_text = h.text();
            let ctx_rules = format!("{locale:?} unicode={} rules", caps.unicode);
            for ch in rules_text.chars() {
                assert!(ch.is_ascii() || is_wgl4(ch), "{ctx_rules}: {ch:?}");
            }
            if caps.unicode {
                for glyph in FORBIDDEN_UNICODE {
                    assert!(!rules_text.contains(glyph), "{ctx_rules}: {glyph}");
                }
            }
        }
    }
    i18n::set_locale(Locale::EnUs);
}
