//! Dates and times of the history in the zone the binary gave the TUI. The standard library has no
//! time zones, so the offset is added to the Unix milliseconds here, and the heading names the zone.

use crate::app::Zone;

const MILLIS_PER_DAY: i64 = 86_400_000;
const MILLIS_PER_MINUTE: i64 = 60_000;

/// Year, month and day of a day count since 1970-01-01 in the proleptic Gregorian calendar.
fn civil_date(days: i64) -> (i64, i64, i64) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let doe = shifted.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mi = (5 * doy + 2) / 153;
    let day = doy - (153 * mi + 2) / 5 + 1;
    let mo = if mi < 10 { mi + 3 } else { mi - 9 };
    (yoe + era * 400 + i64::from(mo <= 2), mo, day)
}

/// `YYYY-MM-DD HH:MM` in `zone` for Unix milliseconds.
pub fn format_date_time(unix_ms: i64, zone: Zone) -> String {
    let offset_ms = i64::from(zone.minutes()) * MILLIS_PER_MINUTE;
    let local_ms = unix_ms.saturating_add(offset_ms);
    let days = local_ms.div_euclid(MILLIS_PER_DAY);
    let minute = local_ms.rem_euclid(MILLIS_PER_DAY) / MILLIS_PER_MINUTE;
    let (year, month, day) = civil_date(days);
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}",
        minute / 60,
        minute % 60
    )
}

/// The key of the history heading: it names local time only when the system offset is known.
pub const fn heading_key(zone: Zone) -> &'static str {
    match zone {
        Zone::Local { .. } => "stats.history.title_local",
        Zone::Utc => "stats.history.title_utc",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const UTC: Zone = Zone::Utc;

    /// 2023-11-14 22:13:20 UTC.
    const NOV_2023: i64 = 1_700_000_000_000;

    #[test]
    fn utc_dates_are_formatted_as_before() {
        assert_eq!(format_date_time(0, UTC), "1970-01-01 00:00");
        assert_eq!(format_date_time(NOV_2023, UTC), "2023-11-14 22:13");
        assert_eq!(format_date_time(951_782_400_000, UTC), "2000-02-29 00:00");
        assert_eq!(format_date_time(-1, UTC), "1969-12-31 23:59");
        assert_eq!(format_date_time(4_102_444_799_000, UTC), "2099-12-31 23:59");
    }

    #[test]
    fn a_zero_offset_local_zone_reads_like_utc() {
        let zero = Zone::Local { minutes: 0 };
        assert_eq!(format_date_time(NOV_2023, zero), "2023-11-14 22:13");
    }

    #[test]
    fn plus_sixty_minutes_moves_the_clock_by_an_hour() {
        let zone = Zone::Local { minutes: 60 };
        assert_eq!(format_date_time(NOV_2023, zone), "2023-11-14 23:13");
    }

    #[test]
    fn minus_three_hundred_minutes_moves_the_clock_back() {
        let zone = Zone::Local { minutes: -300 };
        assert_eq!(format_date_time(NOV_2023, zone), "2023-11-14 17:13");
    }

    #[test]
    fn a_half_hour_zone_is_applied_to_the_minutes() {
        let zone = Zone::Local { minutes: 330 };
        assert_eq!(format_date_time(NOV_2023, zone), "2023-11-15 03:43");
    }

    #[test]
    fn midnight_rolls_the_date_over_forward_and_backward() {
        let midnight_utc = 1_700_006_400_000;
        let east = Zone::Local { minutes: 120 };
        let west = Zone::Local { minutes: -120 };
        assert_eq!(format_date_time(midnight_utc, UTC), "2023-11-15 00:00");
        assert_eq!(format_date_time(midnight_utc, east), "2023-11-15 02:00");
        assert_eq!(format_date_time(midnight_utc - 1, west), "2023-11-14 21:59");
        assert_eq!(format_date_time(midnight_utc + 1, west), "2023-11-14 22:00");
    }

    #[test]
    fn the_heading_names_local_time_only_when_the_offset_is_known() {
        assert_eq!(heading_key(Zone::Utc), "stats.history.title_utc");
        assert_eq!(
            heading_key(Zone::Local { minutes: 0 }),
            "stats.history.title_local"
        );
        assert_eq!(
            heading_key(Zone::Local { minutes: -300 }),
            "stats.history.title_local"
        );
    }
}
