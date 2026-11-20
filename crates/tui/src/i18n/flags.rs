//! The flag of each locale as block-art data: a grid of 4 × 4 pixels, drawn two pixels per cell
//! with half blocks (4 × 2 cells), and the language code shown beside it. The colors are the
//! colors of the flags themselves, not theme tokens (decision T-34). A new locale adds its flag
//! here and names it in the locale list (`i18n/locale.rs`).

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
const BLACK: Rgb = [0, 0, 0];
const GERMAN_RED: Rgb = [221, 0, 0];
const GOLD: Rgb = [255, 206, 0];
const FRENCH_BLUE: Rgb = [0, 85, 164];
const FRENCH_RED: Rgb = [239, 65, 53];
const ITALIAN_GREEN: Rgb = [0, 146, 70];
const ITALIAN_RED: Rgb = [206, 43, 55];
const BRAZILIAN_GREEN: Rgb = [0, 156, 59];
const BRAZILIAN_YELLOW: Rgb = [255, 223, 0];
const BRAZILIAN_BLUE: Rgb = [0, 39, 118];
const UKRAINIAN_BLUE: Rgb = [0, 87, 183];
const UKRAINIAN_YELLOW: Rgb = [255, 215, 0];
const DUTCH_RED: Rgb = [174, 28, 40];
const DUTCH_BLUE: Rgb = [33, 70, 139];
const SPANISH_RED: Rgb = [170, 21, 27];
const SPANISH_YELLOW: Rgb = [241, 191, 0];

const fn horizontal(code: &'static str, rows: [Rgb; PIXELS]) -> Flag {
    let [first, second, third, fourth] = rows;
    Flag {
        code,
        pixels: [
            [first; PIXELS],
            [second; PIXELS],
            [third; PIXELS],
            [fourth; PIXELS],
        ],
    }
}

const fn vertical(code: &'static str, columns: [Rgb; PIXELS]) -> Flag {
    Flag {
        code,
        pixels: [columns; PIXELS],
    }
}

/// The United States: a blue canton over alternating red and white stripes.
pub(crate) const EN_US: Flag = Flag {
    code: "EN",
    pixels: [
        [BLUE, BLUE, RED, RED],
        [BLUE, BLUE, WHITE, WHITE],
        [RED, RED, RED, RED],
        [WHITE, WHITE, WHITE, WHITE],
    ],
};

/// Poland: white over red.
pub(crate) const PL_PL: Flag = horizontal("PL", [WHITE, WHITE, POLISH_RED, POLISH_RED]);

/// Germany: black, red and gold; the middle band takes two rows.
pub(crate) const DE_DE: Flag = horizontal("DE", [BLACK, GERMAN_RED, GERMAN_RED, GOLD]);

/// France: blue, white and red, upright; the middle band takes two columns.
pub(crate) const FR_FR: Flag = vertical("FR", [FRENCH_BLUE, WHITE, WHITE, FRENCH_RED]);

/// Italy: green, white and red, upright; the middle band takes two columns.
pub(crate) const IT_IT: Flag = vertical("IT", [ITALIAN_GREEN, WHITE, WHITE, ITALIAN_RED]);

/// Brazil: a yellow rhombus on green around a blue disc, simplified to its corners and middle.
pub(crate) const PT_BR: Flag = Flag {
    code: "BR",
    pixels: [
        [
            BRAZILIAN_GREEN,
            BRAZILIAN_YELLOW,
            BRAZILIAN_YELLOW,
            BRAZILIAN_GREEN,
        ],
        [
            BRAZILIAN_YELLOW,
            BRAZILIAN_BLUE,
            BRAZILIAN_BLUE,
            BRAZILIAN_YELLOW,
        ],
        [
            BRAZILIAN_YELLOW,
            BRAZILIAN_BLUE,
            BRAZILIAN_BLUE,
            BRAZILIAN_YELLOW,
        ],
        [
            BRAZILIAN_GREEN,
            BRAZILIAN_YELLOW,
            BRAZILIAN_YELLOW,
            BRAZILIAN_GREEN,
        ],
    ],
};

/// Ukraine: blue over yellow. The code is the country's, `UA`, because `UK` reads as the United
/// Kingdom.
pub(crate) const UK_UA: Flag = horizontal(
    "UA",
    [
        UKRAINIAN_BLUE,
        UKRAINIAN_BLUE,
        UKRAINIAN_YELLOW,
        UKRAINIAN_YELLOW,
    ],
);

/// The Netherlands: red, white and blue; the middle band takes two rows.
pub(crate) const NL_NL: Flag = horizontal("NL", [DUTCH_RED, WHITE, WHITE, DUTCH_BLUE]);

/// Spain: red, yellow and red, the yellow band twice as tall.
pub(crate) const ES_ES: Flag = horizontal(
    "ES",
    [SPANISH_RED, SPANISH_YELLOW, SPANISH_YELLOW, SPANISH_RED],
);

/// The flag of `locale`.
pub const fn flag(locale: Locale) -> &'static Flag {
    locale.flag()
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
