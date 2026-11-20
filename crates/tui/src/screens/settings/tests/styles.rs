use ratatui::crossterm::event::KeyCode;

use super::super::options::{BACK_STYLES, FIELDS, FRONT_STYLES, Field, Step};
use super::super::values::unavailable_reason_key;
use super::{changed, move_down, open_settings, value};
use crate::audio::{AudioSettings, AudioStatus};
use crate::i18n::{Locale, set_locale, t};
use crate::settings::Settings;
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, TRUE_COLOR, sound_styles};

#[test]
fn the_sound_style_row_cycles_through_every_embedded_style_both_ways() {
    let base = Settings::default();
    let retro = changed(Field::SoundStyle, &base, Step::Next);
    assert_eq!(retro.sound_style, "retro");
    assert_eq!(
        changed(Field::SoundStyle, &retro, Step::Next).sound_style,
        "default"
    );
    assert_eq!(
        changed(Field::SoundStyle, &base, Step::Previous).sound_style,
        "retro"
    );
}

fn configured_style(h: &Harness) -> Option<String> {
    h.audio
        .borrow()
        .configured
        .last()
        .map(|settings: &AudioSettings| settings.style.clone())
}

#[test]
fn a_saved_sound_style_that_exists_is_kept() {
    let mut storage = MemoryStorage::new();
    storage.settings = Some(Settings {
        sound_style: "retro".to_owned(),
        ..Settings::default()
    });
    let h = Harness::with(storage, TRUE_COLOR, 120, 40);
    assert_eq!(h.app.settings().sound_style, "retro");
    assert_eq!(configured_style(&h).as_deref(), Some("retro"));
}

#[test]
fn a_saved_sound_style_that_no_longer_exists_falls_back_to_default() {
    let mut storage = MemoryStorage::new();
    storage.settings = Some(Settings {
        sound_style: "removed".to_owned(),
        ..Settings::default()
    });
    let h = Harness::with(storage, TRUE_COLOR, 120, 40);
    assert_eq!(h.app.settings().sound_style, "default");
    assert_eq!(configured_style(&h).as_deref(), Some("default"));
}

#[test]
fn choosing_retro_in_settings_reaches_the_audio_output() {
    let mut h = open_settings();
    let index = FIELDS
        .iter()
        .position(|field| *field == Field::SoundStyle)
        .unwrap_or_default();
    move_down(&mut h, index);
    h.press(KeyCode::Enter);
    assert_eq!(h.app.settings().sound_style, "retro");
    assert_eq!(configured_style(&h).as_deref(), Some("retro"));
}

#[test]
fn the_sound_style_row_shows_the_style_name() {
    let base = Settings::default();
    assert_eq!(value(Field::SoundStyle, &base), "Default");
    let retro = Settings {
        sound_style: "retro".to_owned(),
        ..Settings::default()
    };
    assert_eq!(value(Field::SoundStyle, &retro), "Retro");
}

#[test]
fn every_unavailable_status_has_its_own_reason_in_both_locales() {
    let statuses = [
        AudioStatus::RemoteSession,
        AudioStatus::NotBuilt,
        AudioStatus::NoDevice,
        AudioStatus::StyleFailed,
    ];
    for locale in Locale::offered() {
        set_locale(locale);
        let mut texts: Vec<String> = statuses
            .iter()
            .map(|status| {
                let key = unavailable_reason_key(*status).unwrap_or("missing");
                let text = t(key).into_owned();
                assert_ne!(text, key, "{status:?} lacks its text in {}", locale.id());
                text
            })
            .collect();
        texts.sort();
        texts.dedup();
        assert_eq!(texts.len(), statuses.len(), "{}", locale.id());
    }
    set_locale(Locale::EnUs);
    assert!(unavailable_reason_key(AudioStatus::Ready).is_none());
}

#[test]
fn front_and_back_style_rows_keep_their_only_choice_until_more_styles_exist() {
    let base = Settings::default();
    for field in [Field::FrontStyle, Field::BackStyle] {
        assert_eq!(changed(field, &base, Step::Next), base, "{field:?}");
        assert_eq!(changed(field, &base, Step::Previous), base, "{field:?}");
    }
}

#[test]
fn unknown_style_ids_move_to_the_first_offer() {
    let settings = Settings {
        front_style: "custom".to_owned(),
        back_style: "custom".to_owned(),
        sound_style: "custom".to_owned(),
        ..Settings::default()
    };
    assert_eq!(
        changed(Field::FrontStyle, &settings, Step::Next).front_style,
        FRONT_STYLES[0]
    );
    assert_eq!(
        changed(Field::BackStyle, &settings, Step::Next).back_style,
        BACK_STYLES[0]
    );
    assert_eq!(
        changed(Field::SoundStyle, &settings, Step::Next).sound_style,
        "default"
    );
}

#[test]
fn the_default_styles_are_offered() {
    let base = Settings::default();
    assert!(FRONT_STYLES.contains(&base.front_style.as_str()));
    assert!(BACK_STYLES.contains(&base.back_style.as_str()));
    assert!(
        sound_styles()
            .iter()
            .any(|style| style.id == base.sound_style)
    );
}
