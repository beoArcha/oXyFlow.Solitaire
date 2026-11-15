//! Seeds of new games (decision R-18): fresh seeds without a new dependency, and the interim
//! `--seed` override.

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

/// The source for `Deps::seeds`: the `--seed` value for the first new game, fresh seeds afterwards.
pub fn source(first: Option<u64>) -> impl FnMut() -> u64 {
    let mut first = first;
    move || first.take().unwrap_or_else(fresh)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_command_line_seed_comes_first_then_fresh_seeds() {
        let mut next = source(Some(42));
        assert_eq!(next(), 42);
        let second = next();
        let third = next();
        assert_ne!(second, 42);
        assert_ne!(second, third);
    }

    #[test]
    fn without_a_command_line_seed_every_seed_is_fresh() {
        let mut next = source(None);
        assert_ne!(next(), next());
    }
}
