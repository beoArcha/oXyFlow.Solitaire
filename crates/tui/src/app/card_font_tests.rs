//! The oXySol font: the console font option, the fallback to Unicode and the graphics check.

use std::cell::RefCell;
use std::rc::Rc;

use ratatui::crossterm::event::KeyCode;

use super::*;
use crate::i18n::t;
use crate::screens::settings::options::Field;
use crate::settings::{GlyphMode, Settings, TerminalKind};
use crate::storage::memory::MemoryStorage;
use crate::test_support::Harness;

struct Fake {
    asked: Rc<RefCell<Vec<bool>>>,
    answer: CardFontState,
}

impl ConsoleFont for Fake {
    fn step(&mut self, _step: FontStep) -> bool {
        false
    }

    fn use_card_font(&mut self, on: bool) -> CardFontState {
        self.asked.borrow_mut().push(on);
        if on { self.answer } else { CardFontState::Off }
    }
}

type Asked = Rc<RefCell<Vec<bool>>>;

fn with_storage(
    kind: TerminalKind,
    answer: CardFontState,
    storage: MemoryStorage,
) -> (Harness, Asked) {
    let asked = Asked::default();
    let display = DisplayDeps {
        display: Box::new(NullConsoleDisplay),
        facts: DisplayFacts {
            owns_window: false,
            kind,
            flag: None,
        },
        font: Box::new(Fake {
            asked: Rc::clone(&asked),
            answer,
        }),
    };
    (Harness::with_display(storage, display), asked)
}

fn classic(answer: CardFontState) -> (Harness, Asked) {
    with_storage(TerminalKind::ClassicConsole, answer, MemoryStorage::new())
}

fn apply(h: &mut Harness, change: impl FnOnce(&mut Settings)) {
    let mut settings = h.app.settings().clone();
    change(&mut settings);
    h.app.apply(Effect::Apply(settings));
}

#[test]
fn turning_the_console_font_on_uses_it_and_switches_the_symbols_to_font() {
    let (mut h, asked) = classic(CardFontState::Active);
    apply(&mut h, |s| s.console_font = true);
    assert_eq!(*asked.borrow(), [true]);
    assert_eq!(h.app.settings().glyph_mode, GlyphMode::Font);
    assert!(h.app.glyphs.font);
    assert!(h.app.modal().is_none());
}

#[test]
fn turning_it_off_restores_the_console_and_falls_back_to_unicode() {
    let (mut h, asked) = classic(CardFontState::Active);
    apply(&mut h, |s| s.console_font = true);
    apply(&mut h, |s| s.console_font = false);
    assert_eq!(*asked.borrow(), [true, false]);
    assert_eq!(h.app.settings().glyph_mode, GlyphMode::Font);
    assert!(!h.app.glyphs.font);
    assert_eq!(h.app.glyphs.mode(), GlyphMode::Unicode);
}

#[test]
fn a_font_that_is_not_registered_leaves_the_mode_alone_and_the_row_says_so() {
    let (mut h, asked) = classic(CardFontState::NotRegistered);
    apply(&mut h, |s| s.console_font = true);
    assert_eq!(*asked.borrow(), [true]);
    assert_eq!(h.app.settings().glyph_mode, GlyphMode::Unicode);
    assert!(!h.app.glyphs.font);
    assert_eq!(h.app.font_facts().card_font, CardFontState::NotRegistered);
    let facts = h.app.font_facts();
    let text = Field::ConsoleFont.describe(facts);
    assert_eq!(text, t("settings.desc.console_font_missing"));
}

#[test]
fn a_console_that_refuses_the_font_is_reported_and_unicode_stays() {
    let (mut h, _asked) = classic(CardFontState::Refused);
    apply(&mut h, |s| s.console_font = true);
    assert_eq!(h.app.font_facts().card_font, CardFontState::Refused);
    assert_eq!(h.app.glyphs.mode(), GlyphMode::Unicode);
    apply(&mut h, |s| s.console_font = false);
    assert_eq!(h.app.font_facts().card_font, CardFontState::Off);
}

#[test]
fn the_saved_option_is_applied_at_start_and_a_missing_font_falls_back_to_unicode() {
    for (answer, font) in [
        (CardFontState::Active, true),
        (CardFontState::NotRegistered, false),
        (CardFontState::Refused, false),
    ] {
        let mut storage = MemoryStorage::new();
        storage.settings = Some(Settings {
            glyph_mode: GlyphMode::Font,
            console_font: true,
            ..Settings::default()
        });
        let (h, asked) = with_storage(TerminalKind::ClassicConsole, answer, storage);
        assert_eq!(*asked.borrow(), [true], "{answer:?}");
        assert_eq!(h.app.glyphs.font, font, "{answer:?}");
        if !font {
            assert_eq!(h.app.glyphs.mode(), GlyphMode::Unicode, "{answer:?}");
        }
    }
}

#[test]
fn font_mode_without_a_confirmation_or_a_console_font_draws_unicode() {
    let mut storage = MemoryStorage::new();
    storage.settings = Some(Settings {
        glyph_mode: GlyphMode::Font,
        ..Settings::default()
    });
    let (h, asked) = with_storage(TerminalKind::Linux, CardFontState::Active, storage);
    assert!(asked.borrow().is_empty());
    assert!(!h.app.glyphs.font);
    assert_eq!(h.app.glyphs.mode(), GlyphMode::Unicode);
}

#[test]
fn a_terminal_that_is_not_the_classic_console_never_asks_for_the_font() {
    let (mut h, asked) = with_storage(
        TerminalKind::Linux,
        CardFontState::Active,
        MemoryStorage::new(),
    );
    assert_eq!(h.app.font_facts().card_font, CardFontState::Unavailable);
    apply(&mut h, |s| s.glyph_mode = GlyphMode::Ascii);
    assert!(asked.borrow().is_empty());
}

#[test]
fn choosing_font_without_a_confirmation_opens_the_graphics_check_and_keeps_the_mode() {
    let mut h = Harness::new();
    apply(&mut h, |s| s.glyph_mode = GlyphMode::Font);
    assert!(matches!(h.app.modal(), Some(Modal::GraphicsCheck(_))));
    assert_eq!(h.app.settings().glyph_mode, GlyphMode::Unicode);
    assert!(!h.app.settings().font_confirmed);
}

#[test]
fn yes_in_the_graphics_check_switches_to_font_and_remembers_it() {
    let mut h = Harness::new();
    apply(&mut h, |s| s.glyph_mode = GlyphMode::Font);
    h.press(KeyCode::Left);
    h.press(KeyCode::Enter);
    assert!(h.app.modal().is_none());
    assert_eq!(h.app.settings().glyph_mode, GlyphMode::Font);
    assert!(h.app.settings().font_confirmed);
    assert!(h.app.glyphs.font);
    assert_eq!(
        h.storage
            .borrow()
            .settings
            .as_ref()
            .map(|s| s.font_confirmed),
        Some(true)
    );
}

#[test]
fn no_in_the_graphics_check_keeps_unicode_and_points_to_the_wiki() {
    let mut h = Harness::new();
    apply(&mut h, |s| s.glyph_mode = GlyphMode::Font);
    h.press(KeyCode::Enter);
    assert_eq!(h.app.settings().glyph_mode, GlyphMode::Unicode);
    assert!(matches!(
        h.app.modal(),
        Some(Modal::Alert {
            message: "settings.graphics.declined"
        })
    ));
    assert!(h.text().contains("wiki"));
}

#[test]
fn escape_in_the_graphics_check_is_a_no() {
    let mut h = Harness::new();
    apply(&mut h, |s| s.glyph_mode = GlyphMode::Font);
    h.press(KeyCode::Esc);
    assert_eq!(h.app.settings().glyph_mode, GlyphMode::Unicode);
    assert!(matches!(h.app.modal(), Some(Modal::Alert { .. })));
}

#[test]
fn a_confirmed_font_is_chosen_without_asking_again() {
    let mut h = Harness::new();
    apply(&mut h, |s| s.font_confirmed = true);
    apply(&mut h, |s| s.glyph_mode = GlyphMode::Font);
    assert!(h.app.modal().is_none());
    assert!(h.app.glyphs.font);
}

fn check_text(kind: TerminalKind) -> String {
    let (mut h, _asked) = with_storage(kind, CardFontState::Active, MemoryStorage::new());
    apply(&mut h, |s| s.glyph_mode = GlyphMode::Font);
    h.text()
}

#[test]
fn the_windows_terminal_hint_appears_only_in_windows_terminal() {
    let hint = "oXySol Mono";
    assert!(check_text(TerminalKind::WindowsTerminal).contains(hint));
    assert!(check_text(TerminalKind::WindowsTerminal).contains("1.20.11381"));
    for kind in [
        TerminalKind::ClassicConsole,
        TerminalKind::Linux,
        TerminalKind::MacOs,
    ] {
        assert!(!check_text(kind).contains("1.20.11381"), "{kind:?}");
    }
}

#[test]
fn the_graphics_check_shows_samples_in_the_card_font_and_the_question() {
    let mut h = Harness::new();
    apply(&mut h, |s| s.glyph_mode = GlyphMode::Font);
    let text = h.text();
    assert!(
        text.contains(t("settings.graphics.title").as_ref()),
        "{text}"
    );
    assert!(
        text.contains(t("settings.graphics.question").as_ref()),
        "{text}"
    );
    assert!(
        text.chars().any(|c| ('\u{E6BC}'..='\u{E6FF}').contains(&c)),
        "{text}"
    );
}
