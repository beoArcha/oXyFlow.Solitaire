//! Where the help window sits: centred over the screen, or docked to its right or left side.

use serde::{Deserialize, Serialize};

/// The placement of the help window. Centre is the overlay; right and left dock it beside the board.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DockSide {
    #[default]
    Centre,
    Right,
    Left,
}

impl DockSide {
    /// The placement after this one in the cycle: centre, right, left, and centre again.
    pub const fn next(self) -> Self {
        match self {
            Self::Centre => Self::Right,
            Self::Right => Self::Left,
            Self::Left => Self::Centre,
        }
    }

    /// Localization key of the placement's name.
    pub const fn label_key(self) -> &'static str {
        match self {
            Self::Centre => "help.dock.side_centre",
            Self::Right => "help.dock.side_right",
            Self::Left => "help.dock.side_left",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cycle_is_centre_right_left() {
        assert_eq!(DockSide::Centre.next(), DockSide::Right);
        assert_eq!(DockSide::Right.next(), DockSide::Left);
        assert_eq!(DockSide::Left.next(), DockSide::Centre);
    }
}
