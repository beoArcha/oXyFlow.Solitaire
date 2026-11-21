use super::support::{open_stats, three_games};
use crate::app::Screen;
use crate::i18n::{Locale, set_locale, t};
use crate::settings::Settings;

const FOOTER_KEYS: [&str; 4] = [
    "stats.footer_list",
    "stats.footer_replay",
    "stats.footer_copy",
    "stats.footer_back",
];

fn footer_entries(locale: Locale) -> Vec<String> {
    set_locale(locale);
    let entries = FOOTER_KEYS.iter().map(|key| t(key).into_owned()).collect();
    set_locale(Locale::EnUs);
    entries
}

fn assert_spaced_footer(locale: Locale, width: u16, height: u16) {
    let entries = footer_entries(locale);
    let mut storage = three_games();
    storage.settings = Some(Settings {
        locale,
        ..Settings::default()
    });
    let mut h = open_stats(storage, width, height);
    let lines = h.lines();
    let footer = lines.last().cloned().unwrap_or_default();
    let mut end: Option<usize> = None;
    for entry in &entries {
        let from = end.unwrap_or(0);
        let Some(offset) = footer[from..].find(entry.as_str()) else {
            panic!("{locale:?} {width}x{height}: {entry:?} is not in the footer {footer:?}");
        };
        let start = from + offset;
        if let Some(previous) = end {
            let gap = &footer[previous..start];
            assert!(
                gap.contains(' ') && gap.trim().is_empty(),
                "{locale:?} {width}x{height}: no space between two footer entries in {footer:?}"
            );
        }
        end = Some(start + entry.len());
    }
    assert_eq!(h.app.screen(), Screen::Stats);
}

#[test]
fn the_statistics_footer_puts_a_space_between_every_pair_of_entries() {
    for locale in Locale::offered() {
        assert_spaced_footer(locale, 100, 30);
        assert_spaced_footer(locale, 200, 60);
    }
}
