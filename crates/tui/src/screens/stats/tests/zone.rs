use ratatui::crossterm::event::KeyCode;

use super::support::{STATISTICS_ITEM, three_games};
use crate::app::{Screen, Zone};
use crate::test_support::{Harness, TRUE_COLOR};

fn stats_in(zone: Zone) -> Harness {
    let mut h = Harness::with_zone(three_games(), TRUE_COLOR, 100, 30, zone);
    for _ in 0..STATISTICS_ITEM {
        h.press(KeyCode::Down);
    }
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::Stats);
    h
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
