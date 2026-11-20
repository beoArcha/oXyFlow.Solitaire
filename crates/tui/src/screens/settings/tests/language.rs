//! The Language row offers only complete locales, and a saved unfinished one falls back to en-US.

use super::super::options::{Field, Step};
use super::changed;
use crate::i18n::{Locale, set_locale};
use crate::settings::Settings;
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, TRUE_COLOR};

fn cycle_from(start: Locale, step: Step) -> Vec<Locale> {
    let mut settings = Settings {
        locale: start,
        ..Settings::default()
    };
    let mut seen = Vec::new();
    for _ in 0..Locale::ALL.len() * 2 {
        settings = changed(Field::Language, &settings, step);
        seen.push(settings.locale);
    }
    seen
}

#[test]
fn the_language_row_steps_through_the_offered_locales_only() {
    let offered = Locale::offered();
    for step in [Step::Next, Step::Previous] {
        let seen = cycle_from(Locale::EnUs, step);
        assert!(
            seen.iter().all(|locale| offered.contains(locale)),
            "{seen:?}"
        );
        for locale in &offered {
            assert!(seen.contains(locale), "{locale:?} is never reached");
        }
    }
}

#[test]
fn a_saved_unfinished_locale_opens_in_english() {
    let Some(unfinished) = Locale::ALL.into_iter().find(|locale| !locale.is_complete()) else {
        return;
    };
    let storage = MemoryStorage {
        settings: Some(Settings {
            locale: unfinished,
            ..Settings::default()
        }),
        ..MemoryStorage::new()
    };
    let h = Harness::with(storage, TRUE_COLOR, 100, 30);
    assert_eq!(h.app.settings().locale, Locale::EnUs);
    assert_eq!(crate::i18n::locale(), Locale::EnUs);
    set_locale(Locale::EnUs);
}
