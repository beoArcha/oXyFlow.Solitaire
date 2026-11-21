//! The Table colour and Table feel rows (P3-32): the presets and feels in order, their localized names,
//! and the low-contrast marker on a colour that fails the table tests in its theme.

use super::super::options::{Field, Group, Step};
use super::{changed, value};
use crate::i18n::{t, tf};
use crate::settings::{Settings, TableColour, TableFeel};
use crate::theme::ThemeId;
use crate::theme::table::felt_for;

#[test]
fn table_colours_and_feels_cycle_both_ways_in_order() {
    let base = Settings::default();
    assert_eq!(base.table_colour, TableColour::Theme);
    assert_eq!(base.table_feel, TableFeel::Plain);
    for (index, colour) in TableColour::ALL.into_iter().enumerate() {
        let from = Settings {
            table_colour: colour,
            ..base.clone()
        };
        let next = TableColour::ALL[(index + 1) % TableColour::ALL.len()];
        assert_eq!(
            changed(Field::TableColour, &from, Step::Next).table_colour,
            next
        );
    }
    for (index, feel) in TableFeel::ALL.into_iter().enumerate() {
        let from = Settings {
            table_feel: feel,
            ..base.clone()
        };
        let next = TableFeel::ALL[(index + 1) % TableFeel::ALL.len()];
        assert_eq!(
            changed(Field::TableFeel, &from, Step::Next).table_feel,
            next
        );
    }
    assert_eq!(
        changed(Field::TableColour, &base, Step::Previous).table_colour,
        TableColour::Black
    );
    assert_eq!(
        changed(Field::TableFeel, &base, Step::Previous).table_feel,
        TableFeel::Lined
    );
}

#[test]
fn both_rows_sit_in_the_interface_group() {
    assert_eq!(Field::TableColour.group(), Group::Interface);
    assert_eq!(Field::TableFeel.group(), Group::Interface);
}

#[test]
fn each_colour_and_feel_shows_its_localized_name() {
    let base = Settings::default();
    for feel in TableFeel::ALL {
        let settings = Settings {
            table_feel: feel,
            ..base.clone()
        };
        assert_eq!(
            value(Field::TableFeel, &settings),
            t(feel.label_key()).into_owned()
        );
    }
    for colour in TableColour::ALL {
        let settings = Settings {
            table_colour: colour,
            theme: ThemeId::Dark,
            ..base.clone()
        };
        let text = value(Field::TableColour, &settings);
        assert!(
            text.starts_with(&*t(colour.label_key())),
            "{colour:?}: {text}"
        );
    }
}

#[test]
fn a_colour_that_fails_the_table_tests_is_marked_low_contrast_in_its_value() {
    let mut marked = 0;
    for theme in ThemeId::ALL {
        for colour in TableColour::ALL {
            let settings = Settings {
                theme,
                table_colour: colour,
                ..Settings::default()
            };
            let name = t(colour.label_key()).into_owned();
            let text = value(Field::TableColour, &settings);
            if felt_for(theme, colour).shifted {
                marked += 1;
                assert_eq!(
                    text,
                    tf("settings.table.low_contrast", &[("name", &name)]),
                    "{theme:?} {colour:?}"
                );
            } else {
                assert_eq!(text, name, "{theme:?} {colour:?}");
            }
        }
    }
    assert!(marked > 0);
}
