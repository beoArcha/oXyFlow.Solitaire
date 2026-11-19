//! One frame of the terminal as plain cells with resolved colors, independent of Ratatui.

use ratatui::buffer::Buffer;
use ratatui::style::{Color, Modifier};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    pub fn hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub symbol: String,
    pub fg: Rgb,
    pub bg: Rgb,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub dim: bool,
}

impl Cell {
    pub fn is_blank(&self) -> bool {
        self.symbol.trim().is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid {
    pub width: u16,
    pub height: u16,
    pub background: Rgb,
    cells: Vec<Cell>,
}

impl Grid {
    pub fn from_buffer(buffer: &Buffer, default_fg: Rgb, default_bg: Rgb) -> Self {
        let area = buffer.area;
        let mut cells = Vec::with_capacity(usize::from(area.width) * usize::from(area.height));
        for y in 0..area.height {
            for x in 0..area.width {
                cells.push(convert(&buffer[(x, y)], default_fg, default_bg));
            }
        }
        Self {
            width: area.width,
            height: area.height,
            background: default_bg,
            cells,
        }
    }

    pub fn row(&self, y: u16) -> &[Cell] {
        let width = usize::from(self.width);
        let start = usize::from(y) * width;
        &self.cells[start..start + width]
    }
}

fn convert(cell: &ratatui::buffer::Cell, default_fg: Rgb, default_bg: Rgb) -> Cell {
    let fg = resolve(cell.fg, default_fg);
    let bg = resolve(cell.bg, default_bg);
    let modifier = cell.modifier;
    let (fg, bg) = if modifier.contains(Modifier::REVERSED) {
        (bg, fg)
    } else {
        (fg, bg)
    };
    Cell {
        symbol: cell.symbol().to_owned(),
        fg,
        bg,
        bold: modifier.contains(Modifier::BOLD),
        italic: modifier.contains(Modifier::ITALIC),
        underline: modifier.contains(Modifier::UNDERLINED),
        dim: modifier.contains(Modifier::DIM),
    }
}

fn resolve(color: Color, default: Rgb) -> Rgb {
    match color {
        Color::Rgb(r, g, b) => Rgb(r, g, b),
        Color::Reset => default,
        Color::Indexed(index) => indexed(index),
        Color::Black => indexed(0),
        Color::Red => indexed(1),
        Color::Green => indexed(2),
        Color::Yellow => indexed(3),
        Color::Blue => indexed(4),
        Color::Magenta => indexed(5),
        Color::Cyan => indexed(6),
        Color::Gray => indexed(7),
        Color::DarkGray => indexed(8),
        Color::LightRed => indexed(9),
        Color::LightGreen => indexed(10),
        Color::LightYellow => indexed(11),
        Color::LightBlue => indexed(12),
        Color::LightMagenta => indexed(13),
        Color::LightCyan => indexed(14),
        Color::White => indexed(15),
    }
}

const SYSTEM_COLORS: [Rgb; 16] = [
    Rgb(0, 0, 0),
    Rgb(205, 49, 49),
    Rgb(13, 188, 121),
    Rgb(229, 229, 16),
    Rgb(36, 114, 200),
    Rgb(188, 63, 188),
    Rgb(17, 168, 205),
    Rgb(229, 229, 229),
    Rgb(102, 102, 102),
    Rgb(241, 76, 76),
    Rgb(35, 209, 139),
    Rgb(245, 245, 67),
    Rgb(59, 142, 234),
    Rgb(214, 112, 214),
    Rgb(41, 184, 219),
    Rgb(255, 255, 255),
];

fn indexed(index: u8) -> Rgb {
    match index {
        0..=15 => SYSTEM_COLORS[usize::from(index)],
        16..=231 => {
            let cube = index - 16;
            let level = |value: u8| if value == 0 { 0 } else { 55 + value * 40 };
            Rgb(level(cube / 36), level(cube / 6 % 6), level(cube % 6))
        }
        232..=255 => {
            let gray = 8 + (index - 232) * 10;
            Rgb(gray, gray, gray)
        }
    }
}
