//! Writes frames as SVG: one `<rect>` per background run, one `<text>` per run of glyphs with an
//! explicit x for every glyph, so the grid holds in any monospace font.

use std::collections::BTreeMap;

use super::grid::{Cell, Grid};
use super::shapes;

pub const CELL_WIDTH: u32 = 9;
pub const CELL_HEIGHT: u32 = 19;
const BASELINE: u32 = 14;
const FONT_SIZE: u32 = 15;
const FONT_STACK: &str =
    "'Cascadia Mono','DejaVu Sans Mono','Menlo','Consolas','Liberation Mono',monospace";
const MERGE_GAP: usize = 6;

pub struct Frame {
    pub grid: Grid,
    pub millis: u32,
}

pub fn still(grid: &Grid, label: &str) -> String {
    let mut body = String::new();
    for y in 0..grid.height {
        body.push_str(&span_markup(grid, y, 0, usize::from(grid.width), false));
    }
    document(grid, label, "", &body)
}

pub fn animation(frames: &[Frame], label: &str) -> Option<String> {
    let first = frames.first()?;
    let total: u32 = frames.iter().map(|frame| frame.millis).sum();
    let mut body = String::new();
    for y in 0..first.grid.height {
        body.push_str(&span_markup(
            &first.grid,
            y,
            0,
            usize::from(first.grid.width),
            false,
        ));
    }
    let mut css = String::new();
    let mut elapsed = first.millis;
    for (index, pair) in frames.windows(2).enumerate() {
        let number = index + 1;
        let [before, after] = pair else { continue };
        let spans = changed_spans(&before.grid, &after.grid);
        if !spans.is_empty() {
            body.push_str(&format!(
                "<g class=\"d\" style=\"animation-name:k{number}\">"
            ));
            for (y, start, end) in spans {
                body.push_str(&span_markup(&after.grid, y, start, end, true));
            }
            body.push_str("</g>");
            css.push_str(&keyframes(number, elapsed, total));
        }
        elapsed += after.millis;
    }
    let style = format!(
        ".d{{visibility:hidden;animation-duration:{total}ms;animation-iteration-count:infinite;animation-timing-function:step-end}}@media (prefers-reduced-motion:reduce){{.d{{animation:none;visibility:visible}}}}{css}"
    );
    Some(document(&first.grid, label, &style, &body))
}

fn keyframes(number: usize, start: u32, total: u32) -> String {
    let at = f64::from(start) * 100.0 / f64::from(total);
    format!(
        "@keyframes k{number}{{0%,{before:.3}%{{visibility:hidden}}{at:.3}%,100%{{visibility:visible}}}}",
        before = (at - 0.001).max(0.0),
    )
}

fn document(grid: &Grid, label: &str, style: &str, body: &str) -> String {
    let width = u32::from(grid.width) * CELL_WIDTH;
    let height = u32::from(grid.height) * CELL_HEIGHT;
    let label = escape(label);
    let background = grid.background.hex();
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width} {height}\" width=\"{width}\" height=\"{height}\" role=\"img\" aria-label=\"{label}\">\n<title>{label}</title>\n<style>{style}</style>\n<rect width=\"{width}\" height=\"{height}\" fill=\"{background}\"/>\n<g shape-rendering=\"crispEdges\" font-family=\"{FONT_STACK}\" font-size=\"{FONT_SIZE}\">\n{body}</g>\n</svg>\n"
    )
}

fn changed_spans(before: &Grid, after: &Grid) -> Vec<(u16, usize, usize)> {
    let mut spans = Vec::new();
    for y in 0..after.height {
        let (old, new) = (before.row(y), after.row(y));
        let mut current: Option<(usize, usize)> = None;
        for x in 0..new.len() {
            if old.get(x) == new.get(x) {
                continue;
            }
            current = match current {
                Some((start, end)) if x < end + MERGE_GAP => Some((start, x + 1)),
                Some(done) => {
                    spans.push((y, done.0, done.1));
                    Some((x, x + 1))
                }
                None => Some((x, x + 1)),
            };
        }
        if let Some((start, end)) = current {
            spans.push((y, start, end));
        }
    }
    spans
}

fn span_markup(grid: &Grid, y: u16, start: usize, end: usize, paint_all: bool) -> String {
    let cells = &grid.row(y)[start..end];
    let top = u32::from(y) * CELL_HEIGHT;
    let mut out = String::new();
    for (from, to, cell) in runs(cells, |a, b| a.bg == b.bg) {
        if paint_all || cell.bg != grid.background {
            let x = (start + from) as u32 * CELL_WIDTH;
            let width = (to - from) as u32 * CELL_WIDTH;
            out.push_str(&format!(
                "<rect x=\"{x}\" y=\"{top}\" width=\"{width}\" height=\"{CELL_HEIGHT}\" fill=\"{}\"/>
",
                cell.bg.hex()
            ));
        }
    }
    let mut paths: BTreeMap<String, String> = BTreeMap::new();
    for (offset, cell) in cells.iter().enumerate() {
        if shapes::is_drawn(&cell.symbol) {
            for (fill, data) in shapes::shapes(cell, start + offset, top) {
                paths.entry(fill).or_default().push_str(&data);
            }
        }
    }
    for (fill, data) in paths {
        out.push_str(&format!(
            "<path fill=\"{fill}\" d=\"{data}\"/>
"
        ));
    }
    for (from, to, cell) in runs(cells, same_text_style) {
        out.push_str(&text_markup(&cells[from..to], start + from, top, cell));
    }
    out
}

fn skipped(cell: &Cell) -> bool {
    cell.is_blank() || shapes::is_drawn(&cell.symbol)
}

fn same_text_style(a: &Cell, b: &Cell) -> bool {
    skipped(a) == skipped(b)
        && a.fg == b.fg
        && a.bold == b.bold
        && a.italic == b.italic
        && a.underline == b.underline
        && a.dim == b.dim
}

fn runs(cells: &[Cell], same: impl Fn(&Cell, &Cell) -> bool) -> Vec<(usize, usize, &Cell)> {
    let mut result: Vec<(usize, usize, &Cell)> = Vec::new();
    for (index, cell) in cells.iter().enumerate() {
        match result.last_mut() {
            Some((_, end, first)) if same(first, cell) => *end = index + 1,
            _ => result.push((index, index + 1, cell)),
        }
    }
    result
}

fn text_markup(cells: &[Cell], first_column: usize, top: u32, style: &Cell) -> String {
    if skipped(style) {
        return String::new();
    }
    let xs: Vec<String> = (0..cells.len())
        .map(|offset| ((first_column + offset) as u32 * CELL_WIDTH).to_string())
        .collect();
    let glyphs: String = cells.iter().map(|cell| escape(&cell.symbol)).collect();
    let mut attributes = format!("fill=\"{}\"", style.fg.hex());
    if style.bold {
        attributes.push_str(" font-weight=\"bold\"");
    }
    if style.italic {
        attributes.push_str(" font-style=\"italic\"");
    }
    if style.underline {
        attributes.push_str(" text-decoration=\"underline\"");
    }
    if style.dim {
        attributes.push_str(" fill-opacity=\"0.6\"");
    }
    format!(
        "<text x=\"{}\" y=\"{}\" {attributes}>{glyphs}</text>\n",
        xs.join(" "),
        top + BASELINE
    )
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
