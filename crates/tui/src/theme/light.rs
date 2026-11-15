//! The Light palette. Placeholder until micro task M-11 fills it: it aliases Dark so that the
//! theme is valid and every test already runs against it.

use super::palette::Palette;

/// The Light theme: dark text on a light surface. Text pairs must reach a ratio of at least 4.5.
pub fn palette() -> Palette {
    super::dark::palette()
}
