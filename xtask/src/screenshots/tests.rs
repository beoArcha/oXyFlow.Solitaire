use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};

use super::grid::{Grid, Rgb};
use super::shapes;
use super::svg::{self, Frame};

const FG: Rgb = Rgb(200, 200, 200);
const BG: Rgb = Rgb(10, 20, 30);

fn grid_of(rows: &[&str]) -> Grid {
    let width = rows
        .iter()
        .map(|row| row.chars().count())
        .max()
        .unwrap_or(0);
    let mut buffer = Buffer::empty(Rect::new(0, 0, width as u16, rows.len() as u16));
    for (y, row) in rows.iter().enumerate() {
        buffer.set_string(0, y as u16, row, Style::new().fg(Color::Rgb(1, 2, 3)));
    }
    Grid::from_buffer(&buffer, FG, BG)
}

#[test]
fn reset_colors_resolve_to_the_theme_defaults() {
    let grid = grid_of(&["a"]);
    let cell = &grid.row(0)[0];
    assert_eq!(cell.fg, Rgb(1, 2, 3));
    assert_eq!(cell.bg, BG);
}

#[test]
fn a_still_positions_every_glyph_and_escapes_markup() {
    let svg = svg::still(&grid_of(&["a<b"]), "label & more");
    assert!(svg.contains("x=\"0 9 18\""));
    assert!(svg.contains(">a&lt;b</text>"));
    assert!(svg.contains("aria-label=\"label &amp; more\""));
}

#[test]
fn blanks_are_not_written_as_text() {
    let svg = svg::still(&grid_of(&["a  b"]), "x");
    assert!(svg.contains("x=\"0\""));
    assert!(svg.contains("x=\"27\""));
    assert!(!svg.contains(">a  b<"));
}

#[test]
fn box_and_block_characters_become_paths_not_text() {
    assert!(shapes::is_drawn("█"));
    assert!(shapes::is_drawn("─"));
    assert!(shapes::is_drawn("╔"));
    assert!(!shapes::is_drawn("A"));
    assert!(!shapes::is_drawn("♠"));
    let svg = svg::still(&grid_of(&["┌─┐"]), "x");
    assert!(svg.contains("<path fill=\"#010203\""));
    assert!(!svg.contains(">┌─┐<"));
}

#[test]
fn an_animation_draws_only_what_changed_and_loops() {
    let frames = vec![
        Frame {
            grid: grid_of(&["abc", "def"]),
            millis: 100,
        },
        Frame {
            grid: grid_of(&["abc", "dXf"]),
            millis: 300,
        },
    ];
    let svg = svg::animation(&frames, "x").unwrap_or_default();
    assert!(svg.contains("@keyframes k1"));
    assert!(svg.contains("animation-duration:400ms"));
    assert_eq!(svg.matches("class=\"d\"").count(), 1);
    assert!(svg.contains(">X</text>"));
    assert!(svg.contains("prefers-reduced-motion"));
}

#[test]
fn identical_frames_add_no_group_and_no_frames_give_nothing() {
    let frame = || Frame {
        grid: grid_of(&["abc"]),
        millis: 100,
    };
    let svg = svg::animation(&[frame(), frame()], "x").unwrap_or_default();
    assert_eq!(svg.matches("class=\"d\"").count(), 0);
    assert!(svg::animation(&[], "x").is_none());
}
