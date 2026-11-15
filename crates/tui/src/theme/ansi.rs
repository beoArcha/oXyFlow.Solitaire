//! ANSI color theme variant and color space conversions.

const CUBE_LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];

const fn squared_distance(r1: u8, g1: u8, b1: u8, r2: u8, g2: u8, b2: u8) -> u32 {
    let dr = r1 as i32 - r2 as i32;
    let dg = g1 as i32 - g2 as i32;
    let db = b1 as i32 - b2 as i32;
    (dr * dr + dg * dg + db * db) as u32
}

/// Converts an RGB color to the closest ANSI 256-color index.
///
/// Searches the 6 × 6 × 6 color cube (indices 16..=231) and the gray ramp
/// (indices 232..=255) by squared Euclidean distance. Never returns 0..=15.
#[must_use]
pub fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
    let mut best_index = 16;
    let mut best_dist = u32::MAX;

    for cube_idx in 0..216 {
        let ri = cube_idx / 36;
        let gi = (cube_idx / 6) % 6;
        let bi = cube_idx % 6;
        let cr = CUBE_LEVELS[ri];
        let cg = CUBE_LEVELS[gi];
        let cb = CUBE_LEVELS[bi];
        let dist = squared_distance(r, g, b, cr, cg, cb);
        if dist < best_dist {
            best_dist = dist;
            best_index = 16 + cube_idx as u8;
        }
    }

    for i in 0..24 {
        let v = 8 + 10 * i;
        let dist = squared_distance(r, g, b, v, v, v);
        if dist < best_dist {
            best_dist = dist;
            best_index = 232 + i;
        }
    }

    best_index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_to_ansi256_card_examples() {
        assert_eq!(rgb_to_ansi256(0, 0, 0), 16);
        assert_eq!(rgb_to_ansi256(255, 255, 255), 231);
        assert_eq!(rgb_to_ansi256(255, 0, 0), 196);
        assert_eq!(rgb_to_ansi256(128, 128, 128), 244);
        assert_eq!(rgb_to_ansi256(95, 135, 175), 67);
    }

    #[test]
    fn test_never_returns_system_colors() {
        for r in [0, 64, 128, 192, 255] {
            for g in [0, 64, 128, 192, 255] {
                for b in [0, 64, 128, 192, 255] {
                    assert!(rgb_to_ansi256(r, g, b) >= 16);
                }
            }
        }
    }
}
