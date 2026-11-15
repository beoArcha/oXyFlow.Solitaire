//! Conversions between the engine's types and the SQLite column types.

use std::time::{SystemTime, UNIX_EPOCH};

use oxysol_core::Variant;
use oxysol_tui::Outcome;

use super::error::Fault;

/// A seed is a `u64` and SQLite integers are `i64`: the seed is stored as the same 64 bits
/// reinterpreted as `i64`, so every value round-trips and none is rejected.
pub(super) fn seed_to_column(seed: u64) -> i64 {
    i64::from_ne_bytes(seed.to_ne_bytes())
}

pub(super) fn seed_from_column(value: i64) -> u64 {
    u64::from_ne_bytes(value.to_ne_bytes())
}

pub(super) fn millis_to_column(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

pub(super) fn millis_from_column(value: i64) -> Result<u64, Fault> {
    u64::try_from(value).map_err(|_| Fault::invalid(format!("negative duration {value}")))
}

/// Current UTC time as Unix milliseconds.
pub(super) fn now_millis() -> i64 {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    i64::try_from(elapsed).unwrap_or(i64::MAX)
}

pub(super) fn variant_to_column(variant: Variant) -> Result<String, Fault> {
    Ok(serde_json::to_string(&variant)?)
}

pub(super) fn variant_from_column(text: &str) -> Result<Variant, Fault> {
    Ok(serde_json::from_str(text)?)
}

pub(super) fn outcome_to_column(outcome: Outcome) -> &'static str {
    match outcome {
        Outcome::Won => "won",
        Outcome::Abandoned => "abandoned",
    }
}

pub(super) fn outcome_from_column(text: &str) -> Result<Outcome, Fault> {
    match text {
        "won" => Ok(Outcome::Won),
        "abandoned" => Ok(Outcome::Abandoned),
        other => Err(Fault::invalid(format!("unknown outcome `{other}`"))),
    }
}
