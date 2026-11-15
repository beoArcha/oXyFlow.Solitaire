//! A pixel font for rank labels: five pixel rows, drawn two pixel rows per cell with half blocks
//! (Unicode, WGL4) or with `^`, `_` and `#` (ASCII), so a rank is three cells tall.

type Bitmap = [&'static str; 5];

const SPACING: usize = 1;

const ONE: Bitmap = [".#", "##", ".#", ".#", ".#"];
const ZERO: Bitmap = ["###", "#.#", "#.#", "#.#", "###"];
const TWO: Bitmap = ["###", "..#", "###", "#..", "###"];
const THREE: Bitmap = ["###", "..#", "###", "..#", "###"];
const FOUR: Bitmap = ["#.#", "#.#", "###", "..#", "..#"];
const FIVE: Bitmap = ["###", "#..", "###", "..#", "###"];
const SIX: Bitmap = ["###", "#..", "###", "#.#", "###"];
const SEVEN: Bitmap = ["###", "..#", ".#.", ".#.", ".#."];
const EIGHT: Bitmap = ["###", "#.#", "###", "#.#", "###"];
const NINE: Bitmap = ["###", "#.#", "###", "..#", "###"];
const LETTER_A: Bitmap = [".#.", "#.#", "###", "#.#", "#.#"];
const LETTER_D: Bitmap = ["##.", "#.#", "#.#", "#.#", "##."];
const LETTER_J: Bitmap = ["..#", "..#", "..#", "#.#", ".#."];
const LETTER_K: Bitmap = ["#.#", "#.#", "##.", "#.#", "#.#"];
const LETTER_Q: Bitmap = ["###", "#.#", "#.#", "###", "..#"];
const LETTER_W: Bitmap = ["#.#", "#.#", "#.#", "###", "#.#"];

/// Rows per cell column of a drawn rank.
pub(super) const HEIGHT: u16 = 3;

const UNICODE_HALVES: [&str; 4] = [" ", "▀", "▄", "█"];
const ASCII_HALVES: [&str; 4] = [" ", "^", "_", "#"];

fn bitmap(symbol: char) -> Option<&'static Bitmap> {
    Some(match symbol {
        '0' => &ZERO,
        '1' => &ONE,
        '2' => &TWO,
        '3' => &THREE,
        '4' => &FOUR,
        '5' => &FIVE,
        '6' => &SIX,
        '7' => &SEVEN,
        '8' => &EIGHT,
        '9' => &NINE,
        'A' => &LETTER_A,
        'D' => &LETTER_D,
        'J' => &LETTER_J,
        'K' => &LETTER_K,
        'Q' => &LETTER_Q,
        'W' => &LETTER_W,
        _ => return None,
    })
}

fn lit(bitmap: &Bitmap, row: usize, column: usize) -> bool {
    bitmap
        .get(row)
        .is_some_and(|pixels| pixels.as_bytes().get(column) == Some(&b'#'))
}

/// The drawn rows of `text`, or `None` when a character has no pixel shape.
pub(super) fn render(text: &str, unicode: bool) -> Option<Vec<String>> {
    let shapes: Vec<&Bitmap> = text.chars().map(bitmap).collect::<Option<_>>()?;
    let halves = if unicode {
        &UNICODE_HALVES
    } else {
        &ASCII_HALVES
    };
    let rows = (0..usize::from(HEIGHT))
        .map(|cell_row| {
            let mut line = String::new();
            for (index, shape) in shapes.iter().enumerate() {
                if index > 0 {
                    line.push_str(&" ".repeat(SPACING));
                }
                for column in 0..shape[0].len() {
                    let top = lit(shape, cell_row * 2, column);
                    let bottom = lit(shape, cell_row * 2 + 1, column);
                    line.push_str(halves[usize::from(top) + 2 * usize::from(bottom)]);
                }
            }
            line
        })
        .collect();
    Some(rows)
}
