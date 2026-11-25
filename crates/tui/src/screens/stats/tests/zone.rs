use ratatui::crossterm::event::KeyCode;

use super::support::{STATISTICS_ITEM, three_games};
use crate::app::{Screen, Zone};
use crate::i18n::{Locale, set_locale, t};
use crate::settings::Settings;
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, TRUE_COLOR};

fn open(storage: MemoryStorage, zone: Zone, height: u16) -> Harness {
    let mut h = Harness::with_zone(storage, TRUE_COLOR, 100, height, zone);
    for _ in 0..STATISTICS_ITEM {
        h.press(KeyCode::Down);
    }
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::Stats);
    h
}

fn stats_in(zone: Zone) -> Harness {
    open(three_games(), zone, 30)
}

#[test]
fn without_a_known_offset_the_history_is_in_utc_and_says_so() {
    let text = stats_in(Zone::Utc).text();
    assert!(text.contains("times in UTC"), "{text}");
    assert!(text.contains("1970-01-01 00:01"), "{text}");
}

#[test]
fn with_the_system_offset_the_history_is_in_local_time_and_says_so() {
    let text = stats_in(Zone::Local { minutes: 330 }).text();
    assert!(text.contains("local time"), "{text}");
    assert!(!text.contains("times in UTC"), "{text}");
    assert!(text.contains("1970-01-01 05:31"), "{text}");
    assert!(!text.contains("1970-01-01 00:01"), "{text}");
}

#[test]
fn the_local_time_heading_is_whole_in_every_locale_at_100_by_30_and_60() {
    let zone = Zone::Local { minutes: -300 };
    for locale in Locale::offered() {
        set_locale(locale);
        let heading = t("stats.history.title_local").into_owned();
        set_locale(Locale::EnUs);
        for height in [30, 60] {
            let mut storage = three_games();
            storage.settings = Some(Settings {
                locale,
                ..Settings::default()
            });
            let text = open(storage, zone, height).text();
            assert!(
                text.contains(&heading),
                "{locale:?} {height}: {heading:?} is not whole in {text}"
            );
        }
    }
}
