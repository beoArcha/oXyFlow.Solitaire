//! Deterministic randomness: SplitMix64 over `u64` with explicit wrapping arithmetic.
//! Output must never depend on the platform, the pointer width or the app version.

/// SplitMix64 generator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    /// Creates a generator from a seed.
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// The next 64 random bits.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// An unbiased value in `0..n` by rejection sampling. Returns `0` when `n` is `0`.
    pub fn below(&mut self, n: u64) -> u64 {
        if n == 0 {
            return 0;
        }
        let threshold = n.wrapping_neg() % n;
        loop {
            let r = self.next_u64();
            if r >= threshold {
                return r % n;
            }
        }
    }

    /// Fisher–Yates shuffle, walking from the last index down.
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let Ok(bound) = u64::try_from(i + 1) else {
                return;
            };
            let Ok(j) = usize::try_from(self.below(bound)) else {
                return;
            };
            items.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splitmix_reference_values_for_seed_zero() {
        let mut rng = SplitMix64::new(0);
        assert_eq!(rng.next_u64(), 0xE220_A839_7B1D_CDAF);
        assert_eq!(rng.next_u64(), 0x6E78_9E6A_A1B9_65F4);
        assert_eq!(rng.next_u64(), 0x06C4_5D18_8009_454F);
    }

    #[test]
    fn below_stays_in_range_and_handles_edges() {
        let mut rng = SplitMix64::new(7);
        assert_eq!(rng.below(0), 0);
        assert_eq!(rng.below(1), 0);
        for n in [2_u64, 3, 10, 52, 104, u64::MAX] {
            for _ in 0..200 {
                assert!(rng.below(n) < n);
            }
        }
    }

    #[test]
    fn below_hits_every_value() {
        let mut rng = SplitMix64::new(1);
        let mut seen = [false; 6];
        for _ in 0..200 {
            seen[usize::try_from(rng.below(6)).unwrap_or(0)] = true;
        }
        assert!(seen.iter().all(|s| *s));
    }

    #[test]
    fn shuffle_is_a_permutation_and_deterministic() {
        let mut a: Vec<u16> = (0..52).collect();
        let mut b = a.clone();
        SplitMix64::new(9).shuffle(&mut a);
        SplitMix64::new(9).shuffle(&mut b);
        assert_eq!(a, b);
        let mut sorted = a.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, (0..52).collect::<Vec<u16>>());
        assert_ne!(a, (0..52).collect::<Vec<u16>>());
    }

    #[test]
    fn shuffle_handles_tiny_slices() {
        let mut empty: [u8; 0] = [];
        let mut one = [1_u8];
        SplitMix64::new(3).shuffle(&mut empty);
        SplitMix64::new(3).shuffle(&mut one);
        assert_eq!(one, [1]);
    }
}
