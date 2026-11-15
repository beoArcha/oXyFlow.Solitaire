//! The game clock. It never reads the system time: every call is given the current `Instant`
//! (decision T-14), so tests control it. Pause and modals stop it.

use std::time::{Duration, Instant};

/// Elapsed playing time that can be paused and resumed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameClock {
    accumulated: Duration,
    running_since: Option<Instant>,
}

impl GameClock {
    /// A stopped clock showing `elapsed`.
    pub fn stopped(elapsed: Duration) -> Self {
        Self {
            accumulated: elapsed,
            running_since: None,
        }
    }

    /// Starts or continues counting from `now`. Does nothing when already running.
    pub fn resume(&mut self, now: Instant) {
        if self.running_since.is_none() {
            self.running_since = Some(now);
        }
    }

    /// Stops counting at `now`, keeping the time so far.
    pub fn pause(&mut self, now: Instant) {
        self.accumulated = self.elapsed(now);
        self.running_since = None;
    }

    /// Whether the clock is counting.
    pub fn is_running(&self) -> bool {
        self.running_since.is_some()
    }

    /// Playing time up to `now`.
    pub fn elapsed(&self, now: Instant) -> Duration {
        match self.running_since {
            Some(since) => self.accumulated + now.saturating_duration_since(since),
            None => self.accumulated,
        }
    }
}

/// Formats a duration as `m:ss`, or `h:mm:ss` from one hour on.
pub fn format_elapsed(elapsed: Duration) -> String {
    let total = elapsed.as_secs();
    let (hours, minutes, seconds) = (total / 3600, total / 60 % 60, total % 60);
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECOND: Duration = Duration::from_secs(1);

    #[test]
    fn a_stopped_clock_does_not_advance() {
        let clock = GameClock::stopped(SECOND);
        assert_eq!(clock.elapsed(Instant::now() + SECOND * 10), SECOND);
        assert!(!clock.is_running());
    }

    #[test]
    fn a_running_clock_counts_from_resume() {
        let start = Instant::now();
        let mut clock = GameClock::stopped(SECOND);
        clock.resume(start);
        assert_eq!(clock.elapsed(start + SECOND * 4), SECOND * 5);
        assert!(clock.is_running());
    }

    #[test]
    fn pause_freezes_and_resume_continues() {
        let start = Instant::now();
        let mut clock = GameClock::stopped(Duration::ZERO);
        clock.resume(start);
        clock.pause(start + SECOND * 3);
        assert_eq!(clock.elapsed(start + SECOND * 100), SECOND * 3);
        clock.resume(start + SECOND * 100);
        assert_eq!(clock.elapsed(start + SECOND * 102), SECOND * 5);
    }

    #[test]
    fn resume_while_running_keeps_the_original_start() {
        let start = Instant::now();
        let mut clock = GameClock::stopped(Duration::ZERO);
        clock.resume(start);
        clock.resume(start + SECOND * 2);
        assert_eq!(clock.elapsed(start + SECOND * 3), SECOND * 3);
    }

    #[test]
    fn a_time_before_the_start_counts_as_zero() {
        let start = Instant::now() + SECOND * 5;
        let mut clock = GameClock::stopped(Duration::ZERO);
        clock.resume(start);
        assert_eq!(clock.elapsed(start - SECOND), Duration::ZERO);
    }

    #[test]
    fn elapsed_time_is_formatted() {
        assert_eq!(format_elapsed(Duration::from_secs(0)), "0:00");
        assert_eq!(format_elapsed(Duration::from_secs(65)), "1:05");
        assert_eq!(format_elapsed(Duration::from_secs(3_725)), "1:02:05");
    }
}
