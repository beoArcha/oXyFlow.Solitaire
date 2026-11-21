use ratatui::crossterm::event::KeyCode;

use super::super::options::{FIELDS, Field, Step, back_styles, front_styles};
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
fn the_front_and_back_style_rows_list_every_style_found_and_cycle_both_ways() {
    assert_eq!(
        front_styles(),
        [
            "international",
            "french",
            "german",
            "italian",
            "spanish",
            "swiss"
        ]
    );
    assert_eq!(back_styles(), ["lattice", "ornament"]);
    let base = Settings::default();
    let french = changed(Field::FrontStyle, &base, Step::Next);
    assert_eq!(french.front_style, "french");
    assert_eq!(
        changed(Field::FrontStyle, &base, Step::Previous).front_style,
        "swiss"
    );
    assert_eq!(value(Field::FrontStyle, &french), "French");
    let swiss = Settings {
        front_style: "swiss".to_owned(),
        ..Settings::default()
    };
    assert_eq!(value(Field::FrontStyle, &swiss), "Swiss");
    let ornament = changed(Field::BackStyle, &base, Step::Next);
    assert_eq!(ornament.back_style, "ornament");
    assert_eq!(
        changed(Field::BackStyle, &ornament, Step::Next).back_style,
        "lattice"
    );
}

#[test]
fn the_style_rows_show_the_localized_style_names() {
    let base = Settings::default();
    assert_eq!(value(Field::FrontStyle, &base), "International");
    assert_eq!(value(Field::BackStyle, &base), "Lattice");
    let german = Settings {
        front_style: "german".to_owned(),
        back_style: "ornament".to_owned(),
        ..Settings::default()
    };
    assert_eq!(value(Field::FrontStyle, &german), "German");
    assert_eq!(value(Field::BackStyle, &german), "Ornament");
}

#[test]
fn a_saved_style_that_no_longer_exists_falls_back_to_the_default_style() {
    let mut storage = MemoryStorage::new();
    storage.settings = Some(Settings {
        front_style: "removed".to_owned(),
        back_style: "default".to_owned(),
        ..Settings::default()
    });
    let h = Harness::with(storage, TRUE_COLOR, 120, 40);
    assert_eq!(h.app.settings().front_style, "international");
    assert_eq!(h.app.settings().back_style, "lattice");
}

#[test]
fn a_saved_style_that_exists_is_kept() {
    let mut storage = MemoryStorage::new();
    storage.settings = Some(Settings {
        front_style: "italian".to_owned(),
        back_style: "ornament".to_owned(),
        ..Settings::default()
    });
    let h = Harness::with(storage, TRUE_COLOR, 120, 40);
    assert_eq!(h.app.settings().front_style, "italian");
    assert_eq!(h.app.settings().back_style, "ornament");
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
        front_styles()[0]
    );
    assert_eq!(
        changed(Field::BackStyle, &settings, Step::Next).back_style,
        back_styles()[0]
    );
    assert_eq!(
        changed(Field::SoundStyle, &settings, Step::Next).sound_style,
        "default"
    );
}

#[test]
fn the_default_styles_are_offered() {
    let base = Settings::default();
    assert!(front_styles().contains(&base.front_style.as_str()));
    assert!(back_styles().contains(&base.back_style.as_str()));
    assert!(
        sound_styles()
            .iter()
            .any(|style| style.id == base.sound_style)
    );
}
