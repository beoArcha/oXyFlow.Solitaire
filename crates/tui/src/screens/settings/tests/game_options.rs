//! The game option rows of the Gameplay tab (P6-06): every row fits at 100 by 30 in every offered
//! locale with its label and value, and a key on the focused row changes the option.

use ratatui::crossterm::event::KeyCode;

use super::super::options::Field;
use super::super::tabs::Tab;
use super::{bare, focus_in_tab, line_of, open_settings};
use crate::i18n::{Locale, set_locale, t};
use crate::screens::Effect;
use crate::settings::Settings;

const ROWS: [Field; 5] = [
    Field::KlondikePasses,
    Field::KlondikeEmptyColumn,
    Field::SpiderDeal,
    Field::GolfWrap,
    Field::GolfKingStops,
];

#[test]
fn every_game_option_row_shows_its_label_and_value_at_100_by_30_in_every_locale() {
    for locale in Locale::offered() {
        for field in ROWS {
            let mut h = open_settings();
            h.resize(100, 30);
            h.app.apply(Effect::Apply(Settings {
                locale,
                ..Settings::default()
            }));
            focus_in_tab(&mut h, Tab::Gameplay, field);
            let label = t(field.label_key()).into_owned();
            let line = line_of(&mut h, &label);
            assert!(!line.is_empty(), "{locale:?} {field:?}: row not shown");
            let value =
                super::super::game_option_rows::value(field, Settings::default().game_options);
            assert!(
                bare(&line).ends_with(&value),
                "{locale:?} {field:?}: {line}"
            );
            assert!(line.chars().count() <= 100, "{locale:?} {field:?}");
        }
    }
    set_locale(Locale::EnUs);
}

#[test]
fn right_on_a_game_option_row_changes_that_option_for_the_next_game() {
    let mut h = open_settings();
    focus_in_tab(&mut h, Tab::Gameplay, Field::GolfWrap);
    h.press(KeyCode::Right);
    assert!(h.app.settings().game_options.golf_wrap);
    let line = line_of(&mut h, &t(Field::GolfWrap.label_key()));
    assert!(
        line.contains(t("settings.game_option.adjacent").as_ref()),
        "{line}"
    );
    focus_in_tab(&mut h, Tab::Gameplay, Field::KlondikePasses);
    h.press(KeyCode::Right);
    assert_eq!(
        h.app.settings().game_options.klondike_passes,
        oxysol_core::StockPasses::Three
    );
}
