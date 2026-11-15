//! Seeds of new games (decision R-18): fresh seeds without a new dependency. A seed given on the
//! command line is not drawn here; it opens the game selection of the main menu.

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

/// A seed for a new deal, from process-random hasher state mixed with the wall clock.
pub fn fresh() -> u64 {
    let mut hasher = RandomState::new().build_hasher();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos());
    hasher.write_u128(nanos);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consecutive_fresh_seeds_differ() {
        assert_ne!(fresh(), fresh());
    }
}
