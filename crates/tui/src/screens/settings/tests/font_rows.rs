//! The font face and font size rows (P3-29): what they ask of the classic console, how they share
//! the face with the oXySol font, and what they show in every other terminal.

use std::cell::RefCell;
use std::rc::Rc;

use ratatui::crossterm::event::KeyCode;

use super::super::options::{Field, Group};
use super::{index_of, line_of, move_down, settings_menu_index};
use crate::app::{
    CardFontState, ConsoleFont, DisplayDeps, DisplayFacts, FontStep, Modal, NullConsoleDisplay,
    Screen,
};
use crate::i18n::{Locale, set_locale, t};
use crate::settings::{ConsoleFace, Settings, TerminalKind};
use crate::storage::memory::MemoryStorage;
use crate::test_support::Harness;

#[derive(Default)]
struct Calls {
    faces: Vec<Option<String>>,
    sizes: Vec<Option<u16>>,
}

struct Fake {
    calls: Rc<RefCell<Calls>>,
    face_answer: CardFontState,
    card_answer: CardFontState,
}

impl ConsoleFont for Fake {
    fn step(&mut self, _step: FontStep) -> bool {
        false
    }

    fn use_card_font(&mut self, on: bool) -> CardFontState {
        if on {
            self.card_answer
        } else {
            CardFontState::Off
        }
    }

    fn use_face(&mut self, face: Option<&str>) -> CardFontState {
        self.calls.borrow_mut().faces.push(face.map(str::to_owned));
        if face.is_some() {
            self.face_answer
        } else {
            CardFontState::Off
        }
    }

    fn use_size(&mut self, height: Option<u16>) -> bool {
        self.calls.borrow_mut().sizes.push(height);
        true
    }
}

type Shared = Rc<RefCell<Calls>>;

fn harness(
    kind: TerminalKind,
    (face_answer, card_answer): (CardFontState, CardFontState),
    stored: Option<Settings>,
) -> (Harness, Shared) {
    let calls = Shared::default();
    let display = DisplayDeps {
        display: Box::new(NullConsoleDisplay),
        facts: DisplayFacts {
            owns_window: false,
            kind,
            flag: None,
        },
        font: Box::new(Fake {
            calls: Rc::clone(&calls),
            face_answer,
            card_answer,
        }),
    };
    let storage = MemoryStorage {
        settings: stored,
        ..MemoryStorage::new()
    };
    (Harness::with_display(storage, display), calls)
}

fn classic() -> (Harness, Shared) {
    harness(
        TerminalKind::ClassicConsole,
        (CardFontState::Active, CardFontState::Active),
        None,
    )
}

fn open(mut h: Harness) -> Harness {
    move_down(&mut h, settings_menu_index());
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::Settings);
    h
}

fn focus(h: &mut Harness, field: Field) {
    move_down(h, index_of(field));
}

fn row_text(h: &mut Harness, field: Field) -> String {
    line_of(h, &format!("{} .", t(field.label_key())))
}

#[test]
fn the_rows_follow_the_console_font_row_in_the_interface_group() {
    assert_eq!(Field::FontFace.group(), Group::Interface);
    assert_eq!(Field::FontSize.group(), Group::Interface);
    assert_eq!(index_of(Field::FontFace), index_of(Field::ConsoleFont) + 1);
    assert_eq!(index_of(Field::FontSize), index_of(Field::FontFace) + 1);
}

#[test]
fn a_face_is_asked_of_the_console_at_once_and_shown() {
    let (h, calls) = classic();
    let mut h = open(h);
    focus(&mut h, Field::FontFace);
    assert!(row_text(&mut h, Field::FontFace).contains("Console default"));
    h.press(KeyCode::Right);
    assert_eq!(h.app.settings().console_face, ConsoleFace::Consolas);
    assert_eq!(
        calls.borrow().faces.last(),
        Some(&Some("Consolas".to_owned()))
    );
    assert!(row_text(&mut h, Field::FontFace).contains("Consolas"));
    assert_eq!(h.storage.borrow().settings, None);
}

#[test]
fn a_size_is_asked_of_the_console_at_once_and_shown() {
    let (h, calls) = classic();
    let mut h = open(h);
    focus(&mut h, Field::FontSize);
    h.press(KeyCode::Right);
    assert_eq!(h.app.settings().console_font_size, 12);
    assert_eq!(calls.borrow().sizes.last(), Some(&Some(12)));
    assert!(row_text(&mut h, Field::FontSize).contains("12 px"));
    assert_eq!(h.storage.borrow().settings, None);
}

#[test]
fn keeping_saves_the_face_and_the_size() {
    let (h, _) = classic();
    let mut h = open(h);
    focus(&mut h, Field::FontFace);
    h.press(KeyCode::Right);
    h.press(KeyCode::Down);
    h.press(KeyCode::Right);
    h.press(KeyCode::Esc);
    assert!(matches!(h.app.modal(), Some(Modal::Confirm { .. })));
    h.press(KeyCode::Tab);
    h.press(KeyCode::Enter);
    let saved = h.storage.borrow().settings.clone();
    assert!(
        saved.is_some_and(|s| s.console_face == ConsoleFace::Consolas && s.console_font_size == 12)
    );
}

#[test]
fn undoing_gives_the_console_its_face_and_size_back() {
    let (h, calls) = classic();
    let mut h = open(h);
    focus(&mut h, Field::FontFace);
    h.press(KeyCode::Right);
    h.press(KeyCode::Down);
    h.press(KeyCode::Right);
    h.press(KeyCode::Esc);
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::MainMenu);
    assert_eq!(h.app.settings().console_face, ConsoleFace::Default);
    assert_eq!(h.app.settings().console_font_size, 0);
    assert_eq!(calls.borrow().faces.last(), Some(&None));
    assert_eq!(calls.borrow().sizes.last(), Some(&None));
    let saved = h.storage.borrow().settings.clone();
    assert!(
        saved.is_none_or(|s| s.console_face == ConsoleFace::Default && s.console_font_size == 0)
    );
}

#[test]
fn a_face_the_machine_does_not_have_says_not_installed() {
    let (h, _) = harness(
        TerminalKind::ClassicConsole,
        (CardFontState::NotRegistered, CardFontState::Active),
        None,
    );
    let mut h = open(h);
    focus(&mut h, Field::FontFace);
    h.press(KeyCode::Right);
    let text = row_text(&mut h, Field::FontFace);
    assert!(text.contains("(not installed)"), "{text}");
    assert!(h.text().contains(&*t("settings.desc.font_face_missing")));
}

#[test]
fn the_oxysol_font_holds_the_face_and_the_row_says_so() {
    let (h, calls) = classic();
    let mut h = open(h);
    focus(&mut h, Field::ConsoleFont);
    h.press(KeyCode::Right);
    assert!(h.app.settings().console_font);
    h.press(KeyCode::Down);
    let text = row_text(&mut h, Field::FontFace);
    assert!(text.contains("oXySol font"), "{text}");
    assert!(h.text().contains(&*t("settings.desc.font_face_card")));
    let asked = calls.borrow().faces.len();
    h.press(KeyCode::Right);
    assert_eq!(h.app.settings().console_face, ConsoleFace::Default);
    assert_eq!(calls.borrow().faces.len(), asked);
}

#[test]
fn turning_the_oxysol_font_off_brings_the_chosen_face_back() {
    let (h, calls) = classic();
    let mut h = open(h);
    focus(&mut h, Field::ConsoleFont);
    h.press(KeyCode::Down);
    h.press(KeyCode::Right);
    assert_eq!(h.app.settings().console_face, ConsoleFace::Consolas);
    h.press(KeyCode::Up);
    h.press(KeyCode::Right);
    h.press(KeyCode::Right);
    assert!(!h.app.settings().console_font);
    assert_eq!(
        calls.borrow().faces.last(),
        Some(&Some("Consolas".to_owned()))
    );
}

#[test]
fn a_saved_face_and_size_reach_the_console_at_start() {
    let stored = Settings {
        console_face: ConsoleFace::LucidaConsole,
        console_font_size: 20,
        ..Settings::default()
    };
    let (_h, calls) = harness(
        TerminalKind::ClassicConsole,
        (CardFontState::Active, CardFontState::Active),
        Some(stored),
    );
    assert_eq!(
        calls.borrow().faces.last(),
        Some(&Some("Lucida Console".to_owned()))
    );
    assert_eq!(calls.borrow().sizes.last(), Some(&Some(20)));
}

#[test]
fn other_terminals_show_their_own_shortcut_say_why_and_change_nothing() {
    let cases = [
        (TerminalKind::WindowsTerminal, "Ctrl+= / Ctrl+-"),
        (TerminalKind::Linux, "Ctrl+Shift+= / Ctrl+-"),
        (TerminalKind::MacOs, "Cmd+= / Cmd+-"),
    ];
    for (kind, keys) in cases {
        for field in [Field::FontFace, Field::FontSize] {
            let (h, calls) = harness(
                kind,
                (CardFontState::Unavailable, CardFontState::Unavailable),
                None,
            );
            let mut h = open(h);
            focus(&mut h, field);
            let text = row_text(&mut h, field);
            assert!(text.contains(keys), "{kind:?} {field:?}: {text}");
            let screen = h.text();
            assert!(
                screen.contains("cannot change it"),
                "{kind:?} {field:?}:\n{screen}"
            );
            let before = h.app.settings().clone();
            h.press(KeyCode::Right);
            h.press(KeyCode::Enter);
            assert_eq!(h.app.settings(), &before, "{kind:?} {field:?}");
            assert!(calls.borrow().faces.is_empty() && calls.borrow().sizes.is_empty());
        }
    }
}

#[test]
fn windows_terminal_names_the_profile_and_other_terminals_do_not() {
    let (h, _) = harness(
        TerminalKind::WindowsTerminal,
        (CardFontState::Unavailable, CardFontState::Unavailable),
        None,
    );
    let mut h = open(h);
    focus(&mut h, Field::FontFace);
    assert!(h.text().contains("profile"));
    let (h, _) = harness(
        TerminalKind::Linux,
        (CardFontState::Unavailable, CardFontState::Unavailable),
        None,
    );
    let mut h = open(h);
    focus(&mut h, Field::FontFace);
    assert!(!h.text().contains("profile"));
}

#[test]
fn the_rows_and_their_texts_exist_in_every_offered_language() {
    for locale in Locale::offered() {
        set_locale(locale);
        for field in [Field::FontFace, Field::FontSize] {
            for key in [field.label_key(), field.description_key()] {
                assert_ne!(t(key), key, "{locale:?} {key}");
            }
        }
        for key in [
            "settings.value.console_default",
            "settings.value.pixels",
            "settings.value.oxysol_font_in_use",
            "settings.desc.font_face_card",
            "settings.desc.font_face_missing",
            "settings.desc.font_face_refused",
            "settings.desc.font_terminal_wt",
            "settings.desc.font_terminal_other",
        ] {
            assert_ne!(t(key), key, "{locale:?} {key}");
        }
    }
    set_locale(Locale::EnUs);
}
