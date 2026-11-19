//! Unit tests of the window's state, text and key handling. Drawing, the entry points and the
//! mouse are tested through the application in `render_tests.rs`.

use oxysol_core::{DrawCount, PyramidRules, SuitCount, Variant};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::content::{self, Kind, TEXT_WIDTH, VIEW_ROWS};
use super::pages::page;
use super::*;
use crate::i18n::{self, Locale};
use crate::input::{action_for_key, primary_key};

fn press(state: &mut HowToPlay, code: KeyCode) -> Option<Effect> {
    let key = KeyEvent::new(code, KeyModifiers::NONE);
    handle(state, &key, action_for_key(&key))
}

#[test]
fn every_variant_maps_to_the_page_of_its_game() {
    let cases = [
        (Variant::Klondike(DrawCount::Three), Rules::Klondike),
        (Variant::Spider(SuitCount::Two), Rules::Spider),
        (Variant::FreeCell, Rules::FreeCell),
        (Variant::Pyramid(PyramidRules::Classic), Rules::Pyramid),
    ];
    for (variant, rules) in cases {
        assert_eq!(HowToPlay::for_variant(variant), HowToPlay::new(rules));
    }
    for variant in Variant::ALL {
        assert!(Rules::ALL.contains(&Rules::of(variant)), "{variant:?}");
    }
}

#[test]
fn switching_wraps_around_and_returns_to_the_top() {
    let mut state = HowToPlay::new(Rules::Klondike);
    state.scroll_by(5);
    state.switch(true);
    assert_eq!(state, HowToPlay::new(Rules::Spider));
    state.switch(false);
    state.switch(false);
    assert_eq!(state, HowToPlay::new(Rules::Pyramid));
    state.switch(true);
    assert_eq!(state.rules, Rules::Klondike);
}

#[test]
fn scrolling_stays_inside_the_text() {
    for rules in Rules::ALL {
        let mut state = HowToPlay::new(rules);
        assert!(state.max_scroll() > 0, "{rules:?} fits without scrolling");
        state.scroll_by(-3);
        assert_eq!(state.scroll, 0);
        state.scroll_by(10_000);
        assert_eq!(state.scroll, state.max_scroll());
        let rows = content::rows(rules).len();
        assert_eq!(usize::from(state.scroll) + VIEW_ROWS, rows);
        state.scroll_to(false);
        assert_eq!(state.scroll, 0);
        state.scroll_to(true);
        assert_eq!(state.scroll, state.max_scroll());
    }
}

#[test]
fn keys_scroll_switch_and_close() {
    let mut state = HowToPlay::new(Rules::Klondike).focused(Focus::Text);
    assert_eq!(press(&mut state, KeyCode::Down), None);
    assert_eq!(state.scroll, 1);
    assert_eq!(press(&mut state, KeyCode::Char('j')), None);
    assert_eq!(state.scroll, 2);
    assert_eq!(press(&mut state, KeyCode::Up), None);
    assert_eq!(press(&mut state, KeyCode::Char('k')), None);
    assert_eq!(state.scroll, 0);
    assert_eq!(press(&mut state, KeyCode::PageDown), None);
    assert_eq!(state.scroll, 19);
    assert_eq!(press(&mut state, KeyCode::PageUp), None);
    assert_eq!(state.scroll, 0);
    assert_eq!(press(&mut state, KeyCode::End), None);
    assert_eq!(state.scroll, state.max_scroll());
    assert_eq!(press(&mut state, KeyCode::Home), None);
    assert_eq!(state.scroll, 0);

    state.focus = Focus::Games;
    assert_eq!(press(&mut state, KeyCode::Right), None);
    assert_eq!(state.rules, Rules::Spider);
    assert_eq!(press(&mut state, KeyCode::Left), None);
    assert_eq!(press(&mut state, KeyCode::Left), None);
    assert_eq!(state.rules, Rules::Pyramid);
    assert_eq!(press(&mut state, KeyCode::Right), None);
    assert_eq!(state.rules, Rules::Klondike);

    for code in [KeyCode::Esc, KeyCode::Enter, KeyCode::Char(' ')] {
        assert_eq!(press(&mut state, code), Some(Effect::None), "{code:?}");
    }
    for code in [KeyCode::Char('h'), KeyCode::Char('?'), KeyCode::Char('i')] {
        assert_eq!(press(&mut state, code), None, "{code:?}");
    }
}

#[test]
fn every_row_fits_the_text_width_in_both_locales() {
    for locale in Locale::ALL {
        i18n::set_locale(locale);
        for rules in Rules::ALL {
            for row in content::rows(rules) {
                let width = row.lead.chars().count() + row.text.chars().count();
                assert!(width <= TEXT_WIDTH, "{rules:?} {}: {width}", row.text);
            }
        }
    }
    i18n::set_locale(Locale::EnUs);
}

#[test]
fn every_page_is_written_in_both_locales_without_loose_placeholders() {
    for locale in Locale::ALL {
        i18n::set_locale(locale);
        for rules in Rules::ALL {
            let rows = content::rows(rules);
            let headings = rows.iter().filter(|row| row.kind == Kind::Heading).count();
            assert_eq!(headings, 5, "{rules:?}");
            for row in &rows {
                let text = format!("{}{}", row.lead, row.text);
                assert!(!text.contains("rules."), "{locale:?} missing text: {text}");
                assert!(!text.contains('{'), "{locale:?} placeholder: {text}");
            }
            assert_ne!(i18n::t(rules.name_key()), rules.name_key());
        }
    }
    i18n::set_locale(Locale::EnUs);
}

#[test]
fn controls_list_the_keys_of_the_key_table() {
    for rules in Rules::ALL {
        let rows = content::rows(rules);
        let leads: Vec<&str> = rows
            .iter()
            .filter(|row| row.kind == Kind::Key)
            .map(|row| row.lead.trim_end())
            .collect();
        assert_eq!(leads.len(), page(rules).controls.len(), "{rules:?}");
        for control in page(rules).controls {
            let keys = content::keys_of(control.actions);
            assert!(!keys.is_empty());
            assert!(leads.contains(&keys.as_str()), "{rules:?}: {keys}");
        }
    }
}

#[test]
fn control_keys_are_the_documented_defaults() {
    let rows = content::rows(Rules::Klondike);
    let leads: Vec<&str> = rows
        .iter()
        .filter(|row| row.kind == Kind::Key)
        .map(|row| row.lead.trim_end())
        .collect();
    for expected in [
        "Left/Right/Up/Down",
        "Enter / Space",
        "Esc",
        "s",
        "+/-",
        "a",
        "u / Ctrl+Z",
        "h",
    ] {
        assert!(leads.contains(&expected), "{expected} in {leads:?}");
    }
}

#[test]
fn only_games_with_a_stock_list_the_stock_key() {
    let has_stock_row = |rules: Rules| {
        content::rows(rules)
            .iter()
            .any(|row| row.kind == Kind::Key && row.lead.trim_end() == "s")
    };
    assert!(has_stock_row(Rules::Klondike));
    assert!(has_stock_row(Rules::Spider));
    assert!(has_stock_row(Rules::Pyramid));
    assert!(!has_stock_row(Rules::FreeCell));
}

#[test]
fn tips_name_the_current_keys_instead_of_hard_coded_ones() {
    let text: String = content::rows(Rules::Klondike)
        .iter()
        .map(|row| row.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let hint = primary_key(crate::action::Action::Hint).unwrap_or_default();
    assert!(text.contains(&format!("Press {hint} for a hint")), "{text}");
}

#[test]
fn wrapping_packs_words_and_keeps_long_words_whole() {
    assert_eq!(content::wrap("one two three", 7), vec!["one two", "three"]);
    assert_eq!(content::wrap("", 7), Vec::<String>::new());
    assert_eq!(
        content::wrap("a verylongword b", 4),
        vec!["a", "verylongword", "b"]
    );
}
