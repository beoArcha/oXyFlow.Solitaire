use ratatui::crossterm::event::KeyCode;

use super::super::options::{Field, Step};
use super::super::tabs::Tab;
use super::super::values::changed_with_custom;
use super::{changed, focus_in_tab, line_of};
use crate::app::Screen;
use crate::i18n::t;
use crate::screens::Effect;
use crate::settings::{Settings, TableColour};
use crate::storage::memory::MemoryStorage;
use crate::test_support::Harness;
use crate::theme::{ThemeId, UserTheme};

fn custom_palette() -> crate::theme::Palette {
    let mut palette = ThemeId::Dark.palette();
    palette.background.rgb = [14, 18, 28];
    palette
}

#[test]
fn custom_appears_at_the_end_only_when_a_valid_user_theme_is_loaded() {
    let base = Settings::default();
    let dracula = Settings {
        theme: ThemeId::Dracula,
        ..base.clone()
    };
    assert_eq!(
        changed(Field::Theme, &dracula, Step::Next).theme,
        ThemeId::Dark
    );
    assert_eq!(
        changed_with_custom(
            Field::Theme,
            &dracula,
            Step::Next,
            &crate::test_support::sound_styles()
        )
        .theme,
        ThemeId::Custom
    );
    let custom = Settings {
        theme: ThemeId::Custom,
        ..base
    };
    assert_eq!(
        changed_with_custom(
            Field::Theme,
            &custom,
            Step::Next,
            &crate::test_support::sound_styles()
        )
        .theme,
        ThemeId::Dark
    );
    assert_eq!(
        changed_with_custom(
            Field::Theme,
            &Settings::default(),
            Step::Previous,
            &crate::test_support::sound_styles()
        )
        .theme,
        ThemeId::Custom
    );
}

#[test]
fn settings_screen_steps_to_custom_and_saves_custom_when_confirmed() {
    let mut h = Harness::with_user_theme(MemoryStorage::new(), UserTheme::Valid(custom_palette()));
    h.app.apply(Effect::Goto(Screen::Settings));
    focus_in_tab(&mut h, Tab::Popular, Field::Theme);
    h.press(KeyCode::Left);
    assert_eq!(h.app.settings().theme, ThemeId::Custom);
    let row = line_of(&mut h, &t(Field::Theme.label_key()));
    assert!(row.contains(&*t("settings.theme.custom")), "{row}");

    h.press(KeyCode::Esc);
    h.press(KeyCode::Tab);
    h.press(KeyCode::Enter);
    assert_eq!(
        h.storage.borrow().settings.as_ref().map(|s| s.theme),
        Some(ThemeId::Custom)
    );
}

#[test]
fn table_colour_checks_contrast_against_the_loaded_custom_palette() {
    let mut palette = custom_palette();
    palette.table_text.rgb = [255, 255, 255];
    let storage = MemoryStorage {
        settings: Some(Settings {
            theme: ThemeId::Custom,
            table_colour: TableColour::Black,
            ..Settings::default()
        }),
        ..MemoryStorage::new()
    };
    let mut h = Harness::with_user_theme(storage, UserTheme::Valid(palette));
    h.app.apply(Effect::Goto(Screen::Settings));
    focus_in_tab(&mut h, Tab::AudioVisual, Field::TableColour);
    let text = h.text();
    assert!(text.contains(&*t("settings.table_colour.black")), "{text}");

    focus_in_tab(&mut h, Tab::Popular, Field::Theme);
    h.press(KeyCode::Right);
    assert_eq!(h.app.settings().theme, ThemeId::Dark);
    let dark_row = line_of(&mut h, &t(Field::Theme.label_key()));
    assert!(dark_row.contains(&*t("settings.theme.dark")), "{dark_row}");
    assert_eq!(
        h.terminal.backend().buffer()[(0, 0)].bg,
        ratatui::style::Color::Rgb(24, 28, 38)
    );
    h.press(KeyCode::Left);
    assert_eq!(h.app.settings().theme, ThemeId::Custom);
    let custom_row = line_of(&mut h, &t(Field::Theme.label_key()));
    assert!(
        custom_row.contains(&*t("settings.theme.custom")),
        "{custom_row}"
    );
    assert_eq!(
        h.terminal.backend().buffer()[(0, 0)].bg,
        ratatui::style::Color::Rgb(14, 18, 28)
    );

    let mut plain = Harness::with_user_theme(MemoryStorage::new(), UserTheme::Missing);
    plain.app.apply(Effect::Goto(Screen::Settings));
    focus_in_tab(&mut plain, Tab::Popular, Field::Theme);
    for _ in 0..ThemeId::ALL_WITH_CUSTOM.len() {
        assert_ne!(plain.app.settings().theme, ThemeId::Custom);
        plain.press(KeyCode::Right);
    }
}
