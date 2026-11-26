use ratatui::crossterm::event::KeyCode;
use ratatui::style::Color;

use super::*;
use crate::i18n::{Locale, set_locale, t};
use crate::screens::settings::options::Field;
use crate::screens::settings::tabs::Tab;
use crate::settings::LowVisionBackup;
use crate::storage::memory::MemoryStorage;
use crate::test_support::Harness;
use crate::theme::{Palette, ThemeId, Token, UserTheme};

fn custom_palette() -> Palette {
    let mut palette = ThemeId::Dark.palette();
    palette.background.rgb = [14, 18, 28];
    palette
}

fn modal_text(h: &mut Harness) -> String {
    h.lines()
        .iter()
        .filter_map(|line| {
            let (_, after) = line.split_once('║')?;
            let (inner, _) = after.rsplit_once('║')?;
            Some(inner)
        })
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalized_text(h: &mut Harness) -> String {
    h.text()
        .replace(
            ['│', '║', '┌', '┐', '└', '┘', '╔', '╗', '╚', '╝', '─', '═'],
            " ",
        )
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn valid_user_theme_does_not_replace_active_theme_until_chosen_and_renders_with_its_colors() {
    let palette = custom_palette();
    let mut h = Harness::with_user_theme(MemoryStorage::new(), UserTheme::Valid(palette));
    assert!(h.app.modal().is_none());
    assert_eq!(h.app.settings().theme, ThemeId::Dark);
    assert_eq!(h.app.theme.color(Token::Background), Color::Rgb(24, 28, 38));

    h.app.apply(Effect::Apply(Settings {
        theme: ThemeId::Custom,
        ..Settings::default()
    }));
    assert_eq!(h.app.settings().theme, ThemeId::Custom);
    assert_eq!(h.app.theme.id(), ThemeId::Custom);
    assert_eq!(h.app.theme.color(Token::Background), Color::Rgb(14, 18, 28));
    h.lines();
    let cell = &h.terminal.backend().buffer()[(0, 0)];
    assert_eq!(cell.bg, Color::Rgb(14, 18, 28));
}

#[test]
fn unreadable_user_theme_alerts_and_keeps_running_without_custom_option() {
    let user = UserTheme::from_toml("C:/data/theme.toml", "broken = [");
    let mut h = Harness::with_user_theme(MemoryStorage::new(), user);
    assert_eq!(
        h.app.modal(),
        Some(&Modal::Alert {
            message: "settings.theme.user_unreadable",
        })
    );
    assert!(normalized_text(&mut h).contains(&*t("settings.theme.user_unreadable")));
    assert_eq!(h.app.settings().theme, ThemeId::Dark);

    h.press(KeyCode::Enter);
    assert!(h.app.modal().is_none());
    assert!(h.app.user_theme_palette().is_none());
}

#[test]
fn low_contrast_user_theme_alerts_and_keeps_running_without_custom_option() {
    let bad = include_str!("../../../../assets/themes/dark.toml").replace(
        "text = { rgb = [232, 236, 244], ansi16 = \"white\" }",
        "text = { rgb = [40, 44, 54], ansi16 = \"gray\" }",
    );
    let user = UserTheme::from_toml("C:/data/theme.toml", &bad);
    let mut h = Harness::with_user_theme(MemoryStorage::new(), user);
    assert_eq!(
        h.app.modal(),
        Some(&Modal::Alert {
            message: "settings.theme.user_low_contrast",
        })
    );
    assert!(normalized_text(&mut h).contains(&*t("settings.theme.user_low_contrast")));
    assert_eq!(h.app.settings().theme, ThemeId::Dark);

    h.press(KeyCode::Enter);
    assert!(h.app.modal().is_none());
}

#[test]
fn saved_custom_theme_with_missing_or_invalid_file_falls_back_to_default_and_alerts() {
    let saved = Settings {
        theme: ThemeId::Custom,
        low_vision_saved: Some(LowVisionBackup {
            theme: ThemeId::Custom,
            card_size: crate::settings::CardSizeSetting::Auto,
            aura_size: crate::settings::AuraSize::Thin,
            console_font_size: 0,
        }),
        ..Settings::default()
    };
    let storage = MemoryStorage {
        settings: Some(saved.clone()),
        ..MemoryStorage::new()
    };
    let mut missing = Harness::with_user_theme(storage, UserTheme::Missing);
    assert_eq!(missing.app.settings().theme, ThemeId::Dark);
    assert_eq!(
        missing.app.settings().low_vision_saved.map(|b| b.theme),
        Some(ThemeId::Dark)
    );
    assert_eq!(missing.app.theme.id(), ThemeId::Dark);
    assert_eq!(
        missing.app.modal(),
        Some(&Modal::Alert {
            message: "settings.theme.user_missing",
        })
    );
    assert!(normalized_text(&mut missing).contains(&*t("settings.theme.user_missing")));
    missing.press(KeyCode::Enter);
    assert!(missing.app.modal().is_none());

    let storage = MemoryStorage {
        settings: Some(saved),
        ..MemoryStorage::new()
    };
    let invalid = Harness::with_user_theme(
        storage,
        UserTheme::Unreadable {
            path: "theme.toml".to_owned(),
            detail: "bad toml".to_owned(),
        },
    );
    assert_eq!(invalid.app.settings().theme, ThemeId::Dark);
    assert_eq!(
        invalid.app.modal(),
        Some(&Modal::Alert {
            message: "settings.theme.user_unreadable",
        })
    );
}

#[test]
fn old_settings_file_without_custom_still_loads_and_custom_round_trips() {
    let old = toml::from_str::<Settings>("theme = \"dracula\"\nvolume = 40");
    assert!(matches!(
        old,
        Ok(ref s) if s.theme == ThemeId::Dracula && s.volume == 40
    ));

    let custom = Settings {
        theme: ThemeId::Custom,
        ..Settings::default()
    };
    let serialized = toml::to_string(&custom).unwrap_or_default();
    assert!(serialized.contains("theme = \"custom\""));
    let back = toml::from_str::<Settings>(&serialized);
    assert!(matches!(back, Ok(ref s) if s.theme == ThemeId::Custom));
}

#[test]
fn user_theme_alerts_and_custom_label_exist_and_fit_in_every_offered_locale() {
    let keys = [
        "settings.theme.custom",
        "settings.theme.user_unreadable",
        "settings.theme.user_low_contrast",
        "settings.theme.user_missing",
    ];
    for locale in Locale::offered() {
        set_locale(locale);
        for key in keys {
            assert_ne!(t(key), key, "{locale:?} missing {key}");
        }
        let palette = custom_palette();
        let storage = MemoryStorage {
            settings: Some(Settings {
                locale,
                theme: ThemeId::Custom,
                ..Settings::default()
            }),
            ..MemoryStorage::new()
        };
        let mut h = Harness::build(
            storage,
            crate::test_support::TRUE_COLOR,
            100,
            30,
            None,
            None,
            Zone::Utc,
            UserTheme::Valid(palette),
        );
        h.app.apply(Effect::Goto(Screen::Settings));
        let lines = h.lines();
        assert!(lines.len() <= 30, "{locale:?}");
        assert!(lines.iter().all(|l| l.chars().count() <= 100), "{locale:?}");
        assert!(
            lines
                .iter()
                .any(|l| l.contains(&*t("settings.theme.custom"))),
            "{locale:?} {} {:?}",
            Tab::Popular.fields().contains(&Field::Theme),
            lines
        );
        for alert_key in [
            "settings.theme.user_unreadable",
            "settings.theme.user_low_contrast",
            "settings.theme.user_missing",
        ] {
            h.app.apply(Effect::Open(Modal::alert(alert_key)));
            let box_text = modal_text(&mut h);
            assert!(
                box_text.contains(&*t(alert_key)),
                "{locale:?} {alert_key}: {box_text:?}"
            );
            h.press(KeyCode::Enter);
            assert!(h.app.modal().is_none(), "{locale:?} {alert_key}");
        }
    }
    set_locale(Locale::EnUs);
}
