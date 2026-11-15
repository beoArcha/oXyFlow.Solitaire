//! The border of a card: one edge set per highlight, in Unicode (WGL4) and in ASCII.

use ratatui::layout::Rect;
use ratatui::style::Style;

use super::Mark;
use super::paint::Painter;
use crate::glyphs::Glyphs;

pub(super) struct Edges {
    corners: [&'static str; 4],
    horizontal: &'static str,
    vertical: &'static str,
}

const fn edges(
    corners: [&'static str; 4],
    horizontal: &'static str,
    vertical: &'static str,
) -> Edges {
    Edges {
        corners,
        horizontal,
        vertical,
    }
}

struct EdgeSets {
    normal: Edges,
    selected: Edges,
    focus: Edges,
    target: Edges,
    hint: Edges,
}

const UNICODE_EDGES: EdgeSets = EdgeSets {
    normal: edges(["┌", "┐", "└", "┘"], "─", "│"),
    selected: edges(["╒", "╕", "╘", "╛"], "═", "│"),
    focus: edges(["╔", "╗", "╚", "╝"], "═", "║"),
    target: edges(["┌", "┐", "└", "┘"], "─", "│"),
    hint: edges(["┌", "┐", "└", "┘"], "─", "│"),
};

const ASCII_EDGES: EdgeSets = EdgeSets {
    normal: edges(["+", "+", "+", "+"], "-", "|"),
    selected: edges(["*", "*", "*", "*"], "*", "*"),
    focus: edges(["@", "@", "@", "@"], "=", "@"),
    target: edges(["+", "+", "+", "+"], "~", "!"),
    hint: edges(["+", "+", "+", "+"], ".", ":"),
};

pub(super) fn edges_for(glyphs: &Glyphs, mark: Mark) -> &'static Edges {
    let sets = if glyphs.unicode {
        &UNICODE_EDGES
    } else {
        &ASCII_EDGES
    };
    if mark.selected {
        &sets.selected
    } else if mark.focus {
        &sets.focus
    } else if mark.target {
        &sets.target
    } else if mark.hint {
        &sets.hint
    } else {
        &sets.normal
    }
}

pub(super) fn paint_frame(painter: &mut Painter<'_>, area: Rect, edges: &Edges, border: Style) {
    let (left, right) = (area.left(), area.right() - 1);
    let (top, bottom) = (area.top(), area.bottom() - 1);
    for x in left + 1..right {
        painter.put(x, top, edges.horizontal, border);
        painter.put(x, bottom, edges.horizontal, border);
    }
    for y in top + 1..bottom {
        painter.put(left, y, edges.vertical, border);
        painter.put(right, y, edges.vertical, border);
    }
    let [top_left, top_right, bottom_left, bottom_right] = edges.corners;
    painter.put(left, top, top_left, border);
    painter.put(right, top, top_right, border);
    painter.put(left, bottom, bottom_left, border);
    painter.put(right, bottom, bottom_right, border);
}
