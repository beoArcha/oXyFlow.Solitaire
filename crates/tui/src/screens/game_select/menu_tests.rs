use oxysol_core::Variant;
use ratatui::crossterm::event::KeyCode;

use super::catalog::{Family, is_popular};
use super::rows::{self, Heading, Row, items};
use crate::app::Screen;
use crate::caps::TerminalCaps;
use crate::glyphs::Glyphs;
use crate::i18n::{Locale, set_locale, t};
use crate::screens::focus::marked_text;
use crate::screens::variant_label_key;
use crate::settings::{GlyphMode, Settings};
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, PLAIN, TRUE_COLOR};

fn label_of(variant: Variant) -> String {
    t(variant_label_key(variant)).into_owned()
}

fn select_with(favourites: Vec<Variant>, caps: TerminalCaps) -> Harness {
    let storage = MemoryStorage {
        settings: Some(Settings {
            favourites,
            ..Settings::for_caps(&caps)
        }),
        ..MemoryStorage::new()
    };
    let mut h = Harness::with(storage, caps, 120, 40);
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::GameSelect);
    h
}

fn focused_label(h: &mut Harness) -> Option<String> {
    marked_text(&h.lines(), "►").and_then(|text| {
        text.splitn(3, ' ')
            .nth(2)
            .map(|label| label.trim_start().to_owned())
    })
}

fn line_index(lines: &[String], needle: &str) -> Option<usize> {
    lines.iter().position(|line| line.contains(needle))
}

#[test]
fn the_groups_follow_the_fixed_family_order() {
    let rows = rows::build(&[], &[], "");
    let headings: Vec<Family> = rows
        .iter()
        .filter_map(|row| match row {
            Row::Heading {
                heading: Heading::Family(family),
                ..
            } => Some(*family),
            _ => None,
        })
        .collect();
    let expected: Vec<Family> = Family::ALL
        .into_iter()
        .filter(|family| Variant::ALL.iter().any(|v| Family::of(*v) == *family))
        .collect();
    assert_eq!(headings, expected);
    assert_eq!(items(&rows), Variant::ALL.to_vec());
}

#[test]
fn every_variant_sits_under_the_heading_of_its_family() {
    let rows = rows::build(&[], &[], "");
    let mut current = None;
    for row in rows {
        match row {
            Row::Heading {
                heading: Heading::Family(family),
                ..
            } => current = Some(family),
            Row::Heading {
                heading: Heading::Favourites,
                ..
            } => panic!("no favourites were set"),
            Row::Item(variant) => assert_eq!(current, Some(Family::of(variant))),
            Row::Gap => {}
        }
    }
}

#[test]
fn favourites_form_the_first_group_and_leave_their_family() {
    let last = Variant::ALL[Variant::ALL.len() - 1];
    let rows = rows::build(&[last], &[], "");
    assert_eq!(
        rows[0],
        Row::Heading {
            heading: Heading::Favourites,
            folded: false,
            count: 1,
        }
    );
    assert_eq!(rows[1], Row::Item(last));
    assert_eq!(items(&rows).len(), Variant::ALL.len());
    assert_eq!(items(&rows).iter().filter(|v| **v == last).count(), 1);
}

#[test]
fn the_headings_are_drawn_in_order() {
    let mut h = select_with(Vec::new(), TRUE_COLOR);
    let lines = h.lines();
    let mut previous = 0;
    for family in Family::ALL {
        let Some(at) = line_index(&lines, &t(family.title_key())) else {
            continue;
        };
        assert!(at >= previous, "{family:?}");
        previous = at;
    }
    assert!(previous > 0);
}

#[test]
fn popular_games_carry_the_star_in_unicode_and_ascii() {
    for (caps, mode) in [(TRUE_COLOR, GlyphMode::Unicode), (PLAIN, GlyphMode::Ascii)] {
        let star = Glyphs::new(mode).star;
        let mut h = select_with(Vec::new(), caps);
        let text = h.text();
        for variant in Variant::ALL {
            let starred = text.contains(&format!("{star} {}", label_of(variant)));
            assert_eq!(starred, is_popular(variant), "{variant:?} {mode:?}");
        }
    }
}

#[test]
fn the_f_key_toggles_the_favourite_and_saves_it() {
    let mut h = select_with(Vec::new(), TRUE_COLOR);
    h.press(KeyCode::Down);
    let second = Variant::ALL[1];
    h.type_char('f');
    assert_eq!(h.app.settings().favourites, vec![second]);
    let saved = h.storage.borrow().settings.clone();
    assert!(saved.is_some_and(|s| s.favourites == vec![second]));
    let lines = h.lines();
    let heading = line_index(&lines, &t("menu.favourites"));
    let entry = line_index(&lines, &label_of(second));
    assert!(heading.is_some() && heading < entry);
    assert_eq!(focused_label(&mut h), Some(label_of(second)));
    h.type_char('f');
    assert!(h.app.settings().favourites.is_empty());
    assert!(!h.text().contains(&t("menu.favourites").into_owned()));
}

#[test]
fn a_favourite_is_still_there_after_a_restart() {
    let target = Variant::ALL[Variant::ALL.len() - 1];
    let mut h = select_with(vec![target], TRUE_COLOR);
    assert_eq!(focused_label(&mut h), Some(label_of(target)));
    let lines = h.lines();
    assert!(line_index(&lines, &t("menu.favourites")).is_some());
}

#[test]
fn a_favourite_is_marked_without_colour() {
    let mut h = select_with(vec![Variant::ALL[0]], PLAIN);
    let glyphs = Glyphs::new(GlyphMode::Ascii);
    let text = h.text();
    assert!(text.contains(&format!("{} ", glyphs.favourite)));
    assert!(text.contains(&format!("{} ", glyphs.not_favourite)));
}

#[test]
fn a_click_on_the_marks_toggles_the_favourite_without_starting_the_game() {
    let mut h = select_with(Vec::new(), TRUE_COLOR);
    let variant = Variant::ALL[2];
    let (column, row) = h
        .find(&format!("★ {}", label_of(variant)))
        .unwrap_or_default();
    h.click(column, row);
    assert_eq!(h.app.screen(), Screen::GameSelect);
    assert_eq!(h.app.settings().favourites, vec![variant]);
    let (column, row) = h
        .find(&format!("★ {}", label_of(variant)))
        .unwrap_or_default();
    h.click(column, row);
    assert!(h.app.settings().favourites.is_empty());
}

#[test]
fn a_click_on_the_name_starts_the_game() {
    let mut h = select_with(Vec::new(), TRUE_COLOR);
    let variant = Variant::ALL[2];
    let (column, row) = h
        .find(&format!("★ {}", label_of(variant)))
        .unwrap_or_default();
    h.click(column + 2, row);
    assert_eq!(h.app.screen(), Screen::Board);
    assert_eq!(h.app.board().session().map(|s| s.variant()), Some(variant));
}

#[test]
fn every_favourite_round_trips_through_the_settings_file() {
    let settings = Settings {
        favourites: Variant::ALL.to_vec(),
        ..Settings::default()
    };
    let text = toml::to_string(&settings);
    assert!(text.is_ok());
    let back = text.map(|text| toml::from_str::<Settings>(&text));
    assert!(matches!(back, Ok(Ok(ref s)) if *s == settings));
    let old = toml::from_str::<Settings>("volume = 10");
    assert!(matches!(old, Ok(ref s) if s.favourites.is_empty()));
}

#[test]
fn the_menu_fits_at_100_by_30_in_every_offered_locale() {
    for locale in Locale::offered() {
        for (caps, mode) in [(TRUE_COLOR, GlyphMode::Unicode), (PLAIN, GlyphMode::Ascii)] {
            let storage = MemoryStorage {
                settings: Some(Settings {
                    locale,
                    favourites: vec![Variant::ALL[0]],
                    ..Settings::for_caps(&caps)
                }),
                ..MemoryStorage::new()
            };
            let mut h = Harness::with(storage, caps, 100, 30);
            set_locale(locale);
            h.press(KeyCode::Enter);
            let text = h.text();
            let glyphs = Glyphs::new(mode);
            let context = format!("{locale:?} {mode:?}");
            assert!(
                text.contains(&t("menu.favourites").into_owned()),
                "{context}"
            );
            let mut seen = text.clone();
            for _ in 0..Variant::ALL.len() {
                h.press(KeyCode::Down);
                seen.push_str(&h.text());
            }
            for family in Family::ALL {
                assert!(
                    seen.contains(&t(family.title_key()).into_owned()),
                    "{context}: {} is not reachable",
                    family.title_key()
                );
            }
            assert!(
                text.contains(&format!("[ / {} ]", t("menu.search.label"))),
                "{context}"
            );
            assert!(text.contains(glyphs.star), "{context}");
            assert!(
                text.contains(&t("menu.footer.play").into_owned()),
                "{context}"
            );
            assert!(
                text.contains(&t("help.action.back").into_owned()),
                "{context}"
            );
        }
    }
    set_locale(Locale::EnUs);
}
