//! The High Contrast palette. Placeholder until micro task M-12 fills it: it aliases Dark, whose
//! text pairs already reach 7.0, so that the theme is valid and every test already runs against it.

use super::palette::Palette;

/// The High Contrast theme: maximum separation, no mid-tones for text. Text pairs must reach a
/// ratio of at least 7.0.
pub fn palette() -> Palette {
    super::dark::palette()
}
