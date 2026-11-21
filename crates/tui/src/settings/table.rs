//! The table colour and feel: the felt the cards lie on is set apart from the theme. The colour is a
//! preset (or the theme's own felt); the feel is a plain felt, a sparse texture or ruled lines.

use serde::{Deserialize, Serialize};

/// The colour of the table. `Theme` keeps the felt of the theme; the others bring their own felt,
/// which the theme module shades when a preset fails the contrast tests on that table.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TableColour {
    #[default]
    Theme,
    Green,
    Blue,
    Red,
    Grey,
    Brown,
    Black,
}

impl TableColour {
    /// Every colour, in the order of the settings screen.
    pub const ALL: [TableColour; 7] = [
        TableColour::Theme,
        TableColour::Green,
        TableColour::Blue,
        TableColour::Red,
        TableColour::Grey,
        TableColour::Brown,
        TableColour::Black,
    ];

    /// Localization key of the colour name.
    pub const fn label_key(self) -> &'static str {
        match self {
            TableColour::Theme => "settings.table_colour.theme",
            TableColour::Green => "settings.table_colour.green",
            TableColour::Blue => "settings.table_colour.blue",
            TableColour::Red => "settings.table_colour.red",
            TableColour::Grey => "settings.table_colour.grey",
            TableColour::Brown => "settings.table_colour.brown",
            TableColour::Black => "settings.table_colour.black",
        }
    }
}

/// The texture of the felt: none, a sparse dot or shade texture, or faint ruled lines. The texture
/// is drawn only on empty felt, so it never sits under a card, a frame or text.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TableFeel {
    #[default]
    Plain,
    Felt,
    Lined,
}

impl TableFeel {
    /// Every feel, in the order of the settings screen.
    pub const ALL: [TableFeel; 3] = [TableFeel::Plain, TableFeel::Felt, TableFeel::Lined];

    /// Localization key of the feel name.
    pub const fn label_key(self) -> &'static str {
        match self {
            TableFeel::Plain => "settings.table_feel.plain",
            TableFeel::Felt => "settings.table_feel.felt",
            TableFeel::Lined => "settings.table_feel.lined",
        }
    }
}
