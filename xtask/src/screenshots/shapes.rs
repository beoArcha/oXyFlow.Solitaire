//! Box-drawing and block characters drawn as rectangles, so that cards and frames join without
//! gaps whatever font the viewer's browser picks.

use super::grid::{Cell, Rgb};
use super::svg::{CELL_HEIGHT, CELL_WIDTH};

/// Up, down, left and right weight of a box-drawing character: 1 light, 2 double.
const LINES: [(char, [u8; 4]); 35] = [
    ('─', [0, 0, 1, 1]),
    ('│', [1, 1, 0, 0]),
    ('┌', [0, 1, 0, 1]),
    ('┐', [0, 1, 1, 0]),
    ('└', [1, 0, 0, 1]),
    ('┘', [1, 0, 1, 0]),
    ('├', [1, 1, 0, 1]),
    ('┤', [1, 1, 1, 0]),
    ('┬', [0, 1, 1, 1]),
    ('┴', [1, 0, 1, 1]),
    ('┼', [1, 1, 1, 1]),
    ('═', [0, 0, 2, 2]),
    ('║', [2, 2, 0, 0]),
    ('╔', [0, 2, 0, 2]),
    ('╗', [0, 2, 2, 0]),
    ('╚', [2, 0, 0, 2]),
    ('╝', [2, 0, 2, 0]),
    ('╠', [2, 2, 0, 2]),
    ('╣', [2, 2, 2, 0]),
    ('╦', [0, 2, 2, 2]),
    ('╩', [2, 0, 2, 2]),
    ('╬', [2, 2, 2, 2]),
    ('╒', [0, 1, 0, 2]),
    ('╓', [0, 2, 0, 1]),
    ('╕', [0, 1, 2, 0]),
    ('╖', [0, 2, 1, 0]),
    ('╘', [1, 0, 0, 2]),
    ('╙', [2, 0, 0, 1]),
    ('╛', [1, 0, 2, 0]),
    ('╜', [2, 0, 1, 0]),
    ('╞', [1, 1, 0, 2]),
    ('╟', [2, 2, 0, 1]),
    ('╡', [1, 1, 2, 0]),
    ('╢', [2, 2, 1, 0]),
    ('╪', [1, 1, 2, 2]),
];

/// Share of the foreground over the background, in quarters, of the shade characters.
const SHADES: [(char, u16); 3] = [('░', 1), ('▒', 2), ('▓', 3)];

const CENTER_X: f64 = 4.0;
const CENTER_Y: f64 = 9.0;

pub fn is_drawn(symbol: &str) -> bool {
    let mut chars = symbol.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) => {
            LINES.iter().any(|(known, _)| *known == c)
                || SHADES.iter().any(|(known, _)| *known == c)
                || matches!(c, '█' | '▀' | '▄' | '▌' | '▐')
        }
        _ => false,
    }
}

/// The shapes of one cell as `(fill, path data)` pairs.
pub fn shapes(cell: &Cell, column: usize, top: u32) -> Vec<(String, String)> {
    let Some(c) = cell.symbol.chars().next() else {
        return Vec::new();
    };
    let x = column as f64 * f64::from(CELL_WIDTH);
    let y = f64::from(top);
    let fg = cell.fg.hex();
    let width = f64::from(CELL_WIDTH);
    let height = f64::from(CELL_HEIGHT);
    if let Some((_, quarters)) = SHADES.iter().find(|(known, _)| *known == c) {
        let color = blend(cell.fg, cell.bg, *quarters).hex();
        return vec![(color, rect(x, y, width, height))];
    }
    let path = match c {
        '█' => rect(x, y, width, height),
        '▀' => rect(x, y, width, height / 2.0),
        '▄' => rect(x, y + height / 2.0, width, height / 2.0),
        '▌' => rect(x, y, width / 2.0, height),
        '▐' => rect(x + width / 2.0, y, width / 2.0, height),
        _ => LINES
            .iter()
            .find(|(known, _)| *known == c)
            .map_or_else(String::new, |(_, weights)| lines(x, y, *weights)),
    };
    vec![(fg, path)]
}

fn blend(fg: Rgb, bg: Rgb, quarters: u16) -> Rgb {
    let mix = |a: u8, b: u8| ((u16::from(a) * quarters + u16::from(b) * (4 - quarters)) / 4) as u8;
    Rgb(mix(fg.0, bg.0), mix(fg.1, bg.1), mix(fg.2, bg.2))
}

fn lines(x: f64, y: f64, [up, down, left, right]: [u8; 4]) -> String {
    let width = f64::from(CELL_WIDTH);
    let height = f64::from(CELL_HEIGHT);
    let mut out = String::new();
    for (weight, toward_start) in [(up, true), (down, false)] {
        for offset in offsets(weight) {
            let (from, length) = if toward_start {
                (y, CENTER_Y + 1.0 + overshoot(weight))
            } else {
                (
                    y + CENTER_Y - overshoot(weight),
                    height - CENTER_Y + overshoot(weight),
                )
            };
            out.push_str(&rect(x + CENTER_X + offset, from, 1.0, length));
        }
    }
    for (weight, toward_start) in [(left, true), (right, false)] {
        for offset in offsets(weight) {
            let (from, length) = if toward_start {
                (x, CENTER_X + 1.0 + overshoot(weight))
            } else {
                (
                    x + CENTER_X - overshoot(weight),
                    width - CENTER_X + overshoot(weight),
                )
            };
            out.push_str(&rect(from, y + CENTER_Y + offset, length, 1.0));
        }
    }
    out
}

fn offsets(weight: u8) -> &'static [f64] {
    match weight {
        0 => &[],
        1 => &[0.0],
        _ => &[-1.0, 1.0],
    }
}

fn overshoot(weight: u8) -> f64 {
    if weight > 1 { 1.0 } else { 0.0 }
}

fn rect(x: f64, y: f64, width: f64, height: f64) -> String {
    format!("M{x} {y}h{width}v{height}h-{width}z")
}
