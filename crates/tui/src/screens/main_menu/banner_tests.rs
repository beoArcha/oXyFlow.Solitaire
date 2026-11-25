//! First-start low-vision banner on the main menu: shown once when not yet offered, both answers
//! saved, keyboard and mouse navigation, non-colour focus, and 9-locale fit at 100 x 30 / 130 x 40.

use ratatui::crossterm::event::KeyCode;
use ratatui::style::Modifier;

use crate::i18n::{Locale, set_locale, t};
use crate::settings::Settings;
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, PLAIN, TRUE_COLOR};
use crate::theme::ThemeId;

#[test]
fn banner_is_shown_once_with_no_modal_and_old_settings_show_it() {
    let mut fresh = Harness::new();
    assert!(fresh.app.modal().is_none());
    let prompt = t("menu.low_vision.prompt").into_owned();
    assert!(fresh.text().contains(&prompt));
    assert!(fresh.text().contains(&format!("[ {} ]", t("menu.low_vision.yes"))));
    assert!(fresh.text().contains(&format!("[ {} ]", t("menu.low_vision.not_now"))));

    let old: Settings = toml::from_str("volume = 40").unwrap_or_default();
    assert!(!old.low_vision_offered);
    let mut from_old = Harness::with(
        MemoryStorage {
            settings: Some(old),
            ..MemoryStorage::default()
        },
        TRUE_COLOR,
        120,
        40,
    );
    assert!(from_old.app.modal().is_none());
    assert!(from_old.text().contains(&prompt));
}

#[test]
fn keyboard_focuses_banner_without_colour_and_yes_turns_low_vision_on_and_saves() {
    let mut h = Harness::new();
    h.press(KeyCode::Right);
    let yes_text = format!("► [ {} ]", t("menu.low_vision.yes"));
    let (col, row) = h.find(&yes_text).expect("focused Yes button has ►");
    let cell = &h.terminal.backend().buffer()[(col + 2, row)];
    assert!(cell.modifier.contains(Modifier::REVERSED));
    assert!(cell.modifier.contains(Modifier::BOLD));

    h.press(KeyCode::Right);
    let not_now_text = format!("► [ {} ]", t("menu.low_vision.not_now"));
    assert!(h.text().contains(&not_now_text));

    h.press(KeyCode::Left);
    assert!(h.text().contains(&yes_text));
    h.press(KeyCode::Enter);

    assert!(h.app.settings().low_vision);
    assert!(h.app.settings().low_vision_offered);
    assert_eq!(h.app.settings().theme, ThemeId::HighContrast);
    let saved = h.storage.borrow().settings.clone().expect("settings saved");
    assert!(saved.low_vision);
    assert!(saved.low_vision_offered);
    assert!(!h.text().contains(&*t("menu.low_vision.prompt")));
}

#[test]
fn keyboard_not_now_keeps_low_vision_off_and_saves() {
    let mut h = Harness::new();
    h.press(KeyCode::Right);
    h.press(KeyCode::Tab);
    h.type_char(' ');

    assert!(!h.app.settings().low_vision);
    assert!(h.app.settings().low_vision_offered);
    let saved = h.storage.borrow().settings.clone().expect("settings saved");
    assert!(!saved.low_vision);
    assert!(saved.low_vision_offered);
    assert!(!h.text().contains(&*t("menu.low_vision.prompt")));
}

#[test]
fn mouse_clicks_on_yes_and_not_now_save_and_dismiss_the_banner() {
    let mut yes = Harness::new();
    let (col, row) = yes
        .find(&format!("[ {} ]", t("menu.low_vision.yes")))
        .expect("Yes button");
    yes.click(col + 1, row);
    assert!(yes.app.settings().low_vision);
    assert!(yes.app.settings().low_vision_offered);
    assert!(yes.storage.borrow().settings.as_ref().is_some_and(|s| s.low_vision && s.low_vision_offered));
    assert!(!yes.text().contains(&*t("menu.low_vision.prompt")));

    let mut no = Harness::new();
    let (col, row) = no
        .find(&format!("[ {} ]", t("menu.low_vision.not_now")))
        .expect("Not now button");
    no.click(col + 1, row);
    assert!(!no.app.settings().low_vision);
    assert!(no.app.settings().low_vision_offered);
    assert!(no.storage.borrow().settings.as_ref().is_some_and(|s| !s.low_vision && s.low_vision_offered));
    assert!(!no.text().contains(&*t("menu.low_vision.prompt")));
}

#[test]
fn banner_fits_at_both_reference_sizes_across_all_nine_locales_and_ascii() {
    for locale in Locale::ALL {
        set_locale(locale);
        for (caps, width, height) in [
            (TRUE_COLOR, 100, 30),
            (TRUE_COLOR, 130, 40),
            (PLAIN, 100, 30),
        ] {
            let mut h = Harness::with(
                MemoryStorage {
                    settings: Some(Settings {
                        locale,
                        ..Settings::for_caps(&caps)
                    }),
                    ..MemoryStorage::default()
                },
                caps,
                width,
                height,
            );
            let text = h.text();
            if !caps.unicode {
                assert!(text.is_ascii(), "{locale:?} ASCII");
            }
            assert!(
                text.contains(&*t("menu.low_vision.prompt")),
                "{locale:?} {width}x{height}: missing prompt"
            );
            assert!(
                text.contains(&*t("menu.low_vision.yes")),
                "{locale:?} {width}x{height}: missing yes"
            );
            assert!(
                text.contains(&*t("menu.low_vision.not_now")),
                "{locale:?} {width}x{height}: missing not_now"
            );
        }
    }
    set_locale(Locale::EnUs);
}
