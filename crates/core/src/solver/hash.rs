//! A 128-bit position hash for the visited-position sets. Two independent 64-bit lanes make an
//! accidental collision between two different positions negligible for the node budgets used here
//! (below 2^-80 for a million positions), which is what lets an exhausted search be reported as
//! "unwinnable".

use std::collections::HashSet;
use std::hash::{BuildHasherDefault, Hasher};

const GOLDEN: u64 = 0x9E37_79B9_7F4A_7C15;

fn mix(value: u64) -> u64 {
    let mut z = value;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Accumulates numbers and byte runs into a 128-bit key.
#[derive(Clone, Copy)]
pub(super) struct PositionHash {
    a: u64,
    b: u64,
}

impl PositionHash {
    pub(super) fn new() -> Self {
        Self {
            a: GOLDEN,
            b: !GOLDEN,
        }
    }

    pub(super) fn feed(&mut self, value: u64) {
        self.a = mix(self.a ^ value);
        self.b = mix(self.b.wrapping_add(value).wrapping_add(GOLDEN));
    }

    pub(super) fn feed_bytes(&mut self, bytes: &[u8]) {
        self.feed(u64::try_from(bytes.len()).unwrap_or(u64::MAX));
        for chunk in bytes.chunks(8) {
            let packed = chunk
                .iter()
                .fold(0u64, |packed, &byte| (packed << 8) | u64::from(byte));
            self.feed(packed);
        }
    }

    pub(super) fn key(self) -> u128 {
        (u128::from(self.a) << 64) | u128::from(self.b)
    }

    pub(super) fn short(self) -> u64 {
        self.a ^ self.b.rotate_left(32)
    }
}

/// The 64-bit hash of a column: its face-down count and its cards, bottom to top.
pub(super) fn column_hash(hidden: u8, cards: &[u8]) -> u64 {
    let mut hash = PositionHash::new();
    hash.feed(u64::from(hidden));
    hash.feed_bytes(cards);
    hash.short()
}

/// Feeds `hashes` in sorted order, so that columns that differ only in position hash alike.
pub(super) fn feed_unordered(hash: &mut PositionHash, mut hashes: Vec<u64>) {
    hashes.sort_unstable();
    for value in hashes {
        hash.feed(value);
    }
}

/// Passes an already well mixed 128-bit key through as the 64-bit table hash.
#[derive(Default)]
pub(super) struct KeyHasher(u64);

impl Hasher for KeyHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.0 = mix(self.0 ^ u64::from(byte));
        }
    }

    fn write_u128(&mut self, value: u128) {
        let low = u64::try_from(value & u128::from(u64::MAX)).unwrap_or(0);
        let high = u64::try_from(value >> 64).unwrap_or(0);
        self.0 = low ^ high;
    }
}

/// The set of positions already generated.
pub(super) type Seen = HashSet<u128, BuildHasherDefault<KeyHasher>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn order_of_columns_does_not_change_the_key() {
        let key = |columns: Vec<u64>| {
            let mut hash = PositionHash::new();
            feed_unordered(&mut hash, columns);
            hash.key()
        };
        assert_eq!(key(vec![3, 1, 2]), key(vec![2, 3, 1]));
        assert_ne!(key(vec![3, 1, 2]), key(vec![3, 1, 4]));
    }

    #[test]
    fn columns_with_other_cards_or_face_down_counts_differ() {
        assert_ne!(column_hash(0, &[1, 2]), column_hash(0, &[2, 1]));
        assert_ne!(column_hash(0, &[1, 2]), column_hash(1, &[1, 2]));
        assert_ne!(column_hash(0, &[1]), column_hash(0, &[1, 0]));
    }
}
