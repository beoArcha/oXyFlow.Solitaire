//! The flag of each locale as block-art data: a grid of 4 × 4 pixels, drawn two pixels per cell
//! with half blocks (4 × 2 cells), and the language code shown beside it. The colors are the
//! colors of the flags themselves, not theme tokens (decision T-34). A new locale adds its flag
//! here, in the same match.

use super::Locale;

/// One pixel of a flag.
pub type Rgb = [u8; 3];

/// Pixels per side of a flag.
pub const PIXELS: usize = 4;

/// A locale's flag and its language code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Flag {
    /// Two ASCII capitals, drawn next to the flag and instead of it where it cannot be drawn.
    pub code: &'static str,
    /// Rows from the top, left to right.
    pub pixels: [[Rgb; PIXELS]; PIXELS],
}

const WHITE: Rgb = [255, 255, 255];
const RED: Rgb = [200, 16, 46];
const POLISH_RED: Rgb = [220, 20, 60];
const BLUE: Rgb = [60, 59, 110];

/// The United States: a blue canton over alternating red and white stripes.
const EN_US: Flag = Flag {
    code: "EN",
    pixels: [
        [BLUE, BLUE, RED, RED],
        [BLUE, BLUE, WHITE, WHITE],
        [RED, RED, RED, RED],
        [WHITE, WHITE, WHITE, WHITE],
    ],
};

/// Poland: white over red.
const PL_PL: Flag = Flag {
    code: "PL",
    pixels: [
        [WHITE; PIXELS],
        [WHITE; PIXELS],
        [POLISH_RED; PIXELS],
        [POLISH_RED; PIXELS],
    ],
};

/// The flag of `locale`.
pub const fn flag(locale: Locale) -> &'static Flag {
    match locale {
        Locale::EnUs => &EN_US,
        Locale::PlPl => &PL_PL,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_locale_has_a_flag_with_a_distinct_two_letter_code() {
        let mut codes: Vec<&str> = Locale::ALL
            .iter()
            .map(|locale| flag(*locale).code)
            .collect();
        for code in &codes {
            assert_eq!(code.len(), 2);
            assert!(code.chars().all(|c| c.is_ascii_uppercase()));
        }
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), Locale::ALL.len());
    }

    #[test]
    fn every_flag_differs_from_every_other() {
        for (index, first) in Locale::ALL.iter().enumerate() {
            for second in &Locale::ALL[index + 1..] {
                assert_ne!(flag(*first).pixels, flag(*second).pixels);
            }
        }
    }

    #[test]
    fn a_flag_uses_more_than_one_color() {
        for locale in Locale::ALL {
            let pixels = flag(locale).pixels;
            assert!(pixels.iter().flatten().any(|pixel| *pixel != pixels[0][0]));
        }
    }
}
