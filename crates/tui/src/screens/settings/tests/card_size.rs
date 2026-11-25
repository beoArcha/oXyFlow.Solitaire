//! The `Card size` row: its place in the Interface group, its values in both locales and how it steps.

use super::{changed, row, value};
use crate::i18n::{Locale, set_locale};
use crate::screens::settings::options::{Field, Step};
use crate::screens::settings::tabs::Tab;
use crate::settings::{CardSizeSetting, Settings};

#[test]
fn the_row_cycles_every_value_in_both_directions() {
    let mut settings = Settings::default();
    assert_eq!(settings.card_size, CardSizeSetting::Auto);
    for expected in [
        CardSizeSetting::Small,
        CardSizeSetting::Medium,
        CardSizeSetting::Large,
        CardSizeSetting::Huge,
        CardSizeSetting::Auto,
    ] {
        settings = changed(Field::CardSize, &settings, Step::Next);
        assert_eq!(settings.card_size, expected);
    }
    assert_eq!(
        changed(Field::CardSize, &Settings::default(), Step::Previous).card_size,
        CardSizeSetting::Huge
    );
    assert!(Tab::AudioVisual.fields().contains(&Field::CardSize));
}

#[test]
fn the_row_names_its_values_in_english_and_polish() {
    let english = ["Auto", "Small", "Medium", "Large", "Huge"];
    let polish = ["Auto", "Małe", "Średnie", "Duże", "Ogromne"];
    for (locale, names, label) in [
        (Locale::EnUs, english, "Card size"),
        (Locale::PlPl, polish, "Rozmiar kart"),
    ] {
        set_locale(locale);
        for (card_size, name) in CardSizeSetting::ALL.into_iter().zip(names) {
            let settings = Settings {
                card_size,
                ..Settings::default()
            };
            assert_eq!(value(Field::CardSize, &settings), name);
            assert!(row(Field::CardSize, &settings, true).starts_with(label));
        }
    }
    set_locale(Locale::EnUs);
}
