//! Settings screen tests for low-vision mode: canonical Audio & Visual placement, Popular
//! alias, classic console font sizing & restore, non-classic terminal font hints across all 9
//! locales, and 100 x 30 / 130 x 40 rendering with the mode on.

use std::cell::RefCell;
use std::rc::Rc;

use ratatui::crossterm::event::KeyCode;

use super::super::options::Field;
use super::super::tabs::Tab;
use super::{canonical_tab, focus_in_tab, open_settings, switch_to_tab};
use crate::app::{ConsoleFont, DisplayDeps, DisplayFacts, FontFacts, FontStep, NullConsoleDisplay};
use crate::i18n::{Locale, set_locale, t};
use crate::screens::settings::tests::settings_menu_index;
use crate::settings::{AuraSize, CardSizeSetting, LOW_VISION_MIN_FONT_SIZE, Settings, TerminalKind};
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, TRUE_COLOR};
use crate::theme::ThemeId;

struct RecordingFont {
    sizes: Rc<RefCell<Vec<Option<u16>>>>,
}

impl ConsoleFont for RecordingFont {
    fn step(&mut self, _step: FontStep) -> bool {
        false
    }

    fn use_size(&mut self, height: Option<u16>) -> bool {
        self.sizes.borrow_mut().push(height);
        true
    }
}

fn owned_classic_console_harness(initial: Settings) -> (Harness, Rc<RefCell<Vec<Option<u16>>>>) {
    let sizes = Rc::new(RefCell::new(Vec::new()));
    let display = DisplayDeps {
        display: Box::new(NullConsoleDisplay),
        facts: DisplayFacts {
            owns_window: true,
            kind: TerminalKind::ClassicConsole,
            flag: None,
        },
        font: Box::new(RecordingFont {
            sizes: Rc::clone(&sizes),
        }),
        ..DisplayDeps::none()
    };
    let storage = MemoryStorage {
        settings: Some(initial),
        ..MemoryStorage::default()
    };
    let h = Harness::with_display(storage, display);
    (h, sizes)
}

#[test]
fn low_vision_is_canonical_in_audio_visual_and_aliased_in_popular() {
    assert!(Tab::AudioVisual.fields().contains(&Field::LowVision));
    assert!(Tab::Popular.fields().contains(&Field::LowVision));
    assert!(!Tab::Gameplay.fields().contains(&Field::LowVision));
    assert!(Field::LowVision.is_interface());
}

#[test]
fn turning_low_vision_on_in_owned_classic_console_raises_and_restores_font_size() {
    let initial = Settings {
        theme: ThemeId::Dark,
        card_size: CardSizeSetting::Small,
        aura_size: AuraSize::Thin,
        console_font_size: 16,
        ..Settings::default()
    };
    let (mut h, sizes) = owned_classic_console_harness(initial.clone());
    for _ in 0..settings_menu_index() {
        h.press(KeyCode::Down);
    }
    h.press(KeyCode::Enter);

    focus_in_tab(&mut h, Tab::Popular, Field::LowVision);
    h.press(KeyCode::Right);
    assert!(h.app.settings().low_vision);
    assert_eq!(
        h.app.settings().console_font_size,
        LOW_VISION_MIN_FONT_SIZE
    );
    assert_eq!(sizes.borrow().last().copied(), Some(Some(24)));

    h.press(KeyCode::Right);
    assert!(!h.app.settings().low_vision);
    assert_eq!(h.app.settings().console_font_size, 16);
    assert_eq!(h.app.settings().theme, initial.theme);
    assert_eq!(h.app.settings().card_size, initial.card_size);
    assert_eq!(h.app.settings().aura_size, initial.aura_size);
    assert_eq!(sizes.borrow().last().copied(), Some(Some(16)));
}

#[test]
fn non_classic_terminals_keep_console_font_size_and_show_font_hint_in_all_nine_locales() {
    for locale in Locale::ALL {
        set_locale(locale);
        for terminal in [TerminalKind::WindowsTerminal, TerminalKind::Linux] {
            let facts = FontFacts {
                terminal,
                owns_window: false,
                ..FontFacts::default()
            };
            let desc = Field::LowVision.describe(facts);
            assert!(!desc.is_empty(), "{locale:?} {terminal:?}");
            assert!(!desc.contains("24+"), "{locale:?} {terminal:?}: {desc}");
            if terminal == TerminalKind::WindowsTerminal {
                assert!(desc.contains("Ctrl+="), "{locale:?}: {desc}");
            }
        }
        let classic_desc = Field::LowVision.describe(FontFacts {
            terminal: TerminalKind::ClassicConsole,
            owns_window: true,
            ..FontFacts::default()
        });
        assert!(classic_desc.contains("24+"), "{locale:?}: {classic_desc}");
    }
    set_locale(Locale::EnUs);
}

#[test]
fn low_vision_renders_settings_at_100x30_and_130x40_across_all_nine_locales() {
    for locale in Locale::ALL {
        set_locale(locale);
        let settings = Settings {
            locale,
            ..Settings::default()
        }
        .with_low_vision(true, false);
        for (width, height) in [(100, 30), (130, 40)] {
            let mut h = Harness::with(
                MemoryStorage {
                    settings: Some(settings.clone()),
                    ..MemoryStorage::default()
                },
                TRUE_COLOR,
                width,
                height,
            );
            for _ in 0..settings_menu_index() {
                h.press(KeyCode::Down);
            }
            h.press(KeyCode::Enter);
            switch_to_tab(&mut h, canonical_tab(Field::LowVision));
            let lines = h.lines();
            assert!(lines.len() <= usize::from(height), "{locale:?} {width}x{height}");
            assert!(
                lines.iter().all(|l| l.chars().count() <= usize::from(width)),
                "{locale:?} {width}x{height} overflow"
            );
            let text = lines.join("\n");
            assert!(
                text.contains(&*t(Field::LowVision.label_key())),
                "{locale:?} {width}x{height}"
            );
        }
    }
    set_locale(Locale::EnUs);
}

#[test]
fn open_settings_in_low_vision_keeps_preview_confirm_when_toggling() {
    let mut h = open_settings();
    focus_in_tab(&mut h, Tab::Popular, Field::LowVision);
    let before = h.app.settings().clone();
    h.press(KeyCode::Right);
    assert!(h.app.settings().low_vision);
    h.press(KeyCode::Esc);
    h.press(KeyCode::Enter);
    assert_eq!(h.app.settings(), &before);
}
