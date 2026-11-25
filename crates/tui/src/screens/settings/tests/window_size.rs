//! The `Window size` row: beside `Full screen` in Audio & Visual, cycles Small/Medium/Large,
//! unavailable on non-resizable or unowned consoles, explains deferred resize when full screen is
//! active, and fits at 100 x 30 in all nine locales.

use std::cell::RefCell;
use std::rc::Rc;

use ratatui::crossterm::event::KeyCode;

use super::super::options::{Field, Step};
use super::super::tabs::Tab;
use super::{bare, changed, focus_in_tab, line_of, settings_menu_index, switch_to_tab, value};
use crate::app::{ConsoleDisplay, DisplayDeps, DisplayFacts, FontFacts, NullConsoleFont, Screen};
use crate::i18n::{Locale, set_locale, t};
use crate::settings::{FullScreen, Settings, TerminalKind, WindowSize};
use crate::storage::memory::MemoryStorage;
use crate::test_support::Harness;

type Sizes = Rc<RefCell<Vec<(u16, u16)>>>;

struct RecordingDisplay(Sizes);

impl ConsoleDisplay for RecordingDisplay {
    fn set_full_screen(&mut self, _on: bool) -> bool {
        true
    }

    fn set_window_size(&mut self, columns: u16, rows: u16) -> bool {
        self.0.borrow_mut().push((columns, rows));
        true
    }
}

fn owned_classic(full_screen: FullScreen) -> (Harness, Sizes) {
    let sizes = Rc::new(RefCell::new(Vec::new()));
    let storage = MemoryStorage {
        settings: Some(Settings {
            full_screen,
            ..Settings::default()
        }),
        ..MemoryStorage::new()
    };
    let display = DisplayDeps {
        priority: crate::app::PriorityDeps::none(),
        display: Box::new(RecordingDisplay(Rc::clone(&sizes))),
        facts: DisplayFacts {
            owns_window: true,
            kind: TerminalKind::ClassicConsole,
            flag: None,
        },
        font: Box::new(NullConsoleFont),
    };
    let mut h = Harness::with_display(storage, display);
    for _ in 0..settings_menu_index() {
        h.press(KeyCode::Down);
    }
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::Settings);
    (h, sizes)
}

#[test]
fn window_size_sits_beside_full_screen_in_audio_and_visual() {
    let av = Tab::AudioVisual.fields();
    let full_screen = av.iter().position(|f| *f == Field::FullScreen);
    let window_size = av.iter().position(|f| *f == Field::WindowSize);
    assert!(full_screen.is_some());
    assert_eq!(window_size, full_screen.map(|index| index + 1));
    assert!(!Tab::Gameplay.fields().contains(&Field::WindowSize));
    assert!(!Field::WindowSize.is_interface());
}

#[test]
fn window_size_cycles_small_medium_large_both_ways() {
    let mut settings = Settings::default();
    assert_eq!(settings.window_size, WindowSize::Small);
    for expected in [WindowSize::Medium, WindowSize::Large, WindowSize::Small] {
        settings = changed(Field::WindowSize, &settings, Step::Next);
        assert_eq!(settings.window_size, expected);
    }
    assert_eq!(
        changed(Field::WindowSize, &Settings::default(), Step::Previous).window_size,
        WindowSize::Large
    );
}

#[test]
fn window_size_changes_by_keyboard_and_mouse_in_owned_classic_console() {
    let (mut h, sizes) = owned_classic(FullScreen::Off);
    focus_in_tab(&mut h, Tab::AudioVisual, Field::WindowSize);
    h.press(KeyCode::Right);
    assert_eq!(h.app.settings().window_size, WindowSize::Medium);
    assert_eq!(sizes.borrow().last().copied(), Some((195, 60)));

    let label = format!("{} .", t(Field::WindowSize.label_key()));
    let (col, row) = h.find(&label).expect("window size row drawn");
    h.click(col + 1, row);
    assert_eq!(h.app.settings().window_size, WindowSize::Large);
    assert_eq!(sizes.borrow().last().copied(), Some((260, 80)));
}

#[test]
fn window_size_descriptions_cover_all_states_in_every_locale() {
    for locale in Locale::offered() {
        set_locale(locale);
        for size in WindowSize::ALL {
            let label = value(
                Field::WindowSize,
                &Settings {
                    window_size: size,
                    ..Settings::default()
                },
            );
            assert!(
                !label.is_empty() && !label.starts_with("settings."),
                "{locale:?} {size:?}"
            );
        }
        let windowed = Field::WindowSize.describe(FontFacts {
            terminal: TerminalKind::ClassicConsole,
            owns_window: true,
            full_screen_active: false,
            ..FontFacts::default()
        });
        let full_screen = Field::WindowSize.describe(FontFacts {
            terminal: TerminalKind::ClassicConsole,
            owns_window: true,
            full_screen_active: true,
            ..FontFacts::default()
        });
        let not_owned = Field::WindowSize.describe(FontFacts {
            terminal: TerminalKind::ClassicConsole,
            owns_window: false,
            ..FontFacts::default()
        });
        let terminal = Field::WindowSize.describe(FontFacts {
            terminal: TerminalKind::WindowsTerminal,
            owns_window: true,
            ..FontFacts::default()
        });
        let all = [&windowed, &full_screen, &not_owned, &terminal];
        for (i, one) in all.iter().enumerate() {
            assert!(!one.starts_with("settings."), "{locale:?}: {one}");
            assert!(all[i + 1..].iter().all(|two| two != one), "{locale:?}");
        }
    }
    set_locale(Locale::EnUs);
}

#[test]
fn full_screen_active_explains_deferred_resize_in_settings() {
    set_locale(Locale::EnUs);
    let (mut h, _) = owned_classic(FullScreen::Auto);
    focus_in_tab(&mut h, Tab::AudioVisual, Field::WindowSize);
    assert!(
        h.text()
            .contains(&*t("settings.desc.window_size_full_screen")),
        "{}",
        h.text()
    );
}

#[test]
fn all_nine_locales_fit_window_size_choices_at_100_by_30() {
    assert_eq!(Locale::ALL.len(), 9);
    for locale in Locale::ALL {
        set_locale(locale);
        for size in WindowSize::ALL {
            let (mut h, _) = owned_classic(FullScreen::Off);
            h.resize(100, 30);
            h.app.apply(crate::screens::Effect::Apply(Settings {
                locale,
                full_screen: FullScreen::Off,
                window_size: size,
                ..Settings::default()
            }));
            switch_to_tab(&mut h, Tab::AudioVisual);
            focus_in_tab(&mut h, Tab::AudioVisual, Field::WindowSize);
            let lines = h.lines();
            assert!(lines.len() <= 30, "{locale:?} {size:?}");
            assert!(
                lines.iter().all(|line| line.chars().count() <= 100),
                "{locale:?} {size:?} overflow"
            );
            let expected_value = t(size.label_key()).into_owned();
            let row = line_of(&mut h, &t(Field::WindowSize.label_key()));
            assert!(
                bare(&row).ends_with(&expected_value),
                "{locale:?} {size:?}: {row}"
            );
        }
    }
    set_locale(Locale::EnUs);
}
